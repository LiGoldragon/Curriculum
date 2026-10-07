use std::{
    fs,
    path::Path,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use datom_codec::{Actualizing, Budget, Potential};
use protos::ReaderBudget;
use tempfile::tempdir;

use curriculum::generated::RolesDocument;

struct RunningNexus(Child);

impl Drop for RunningNexus {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn path_edits_reread_sources_and_regenerate_all_five_skill_trees() {
    let temporary = tempdir().expect("temporary root");
    let runtime = temporary.path().join("runtime");
    let runtime_directory = runtime.join("curriculum");
    let psyche = temporary.path().join("psyche-skills");
    let mind = temporary.path().join("mind-skills");
    let field = temporary.path().join("field-skills");
    let workspace = temporary.path().join("workspace");
    let roles_file = temporary.path().join("roles.datom");
    for directory in [&runtime_directory, &psyche, &mind, &field, &workspace] {
        fs::create_dir_all(directory).expect("fixture directory");
    }
    fs::write(&roles_file, "Roles.{ [] [] [] [] [] [] [] [] [] }")
        .expect("empty role configuration");
    let changed = mind.join("knowledge-changed.md");
    let root = mind.join("operation-root.md");
    let removed = field.join("compensation-removed.md");
    let behavior = field.join("compensation-behavior.md");
    let added = mind.join("operation-added.md");
    fs::write(
        &changed,
        "---\ndependencies: [ compensation-behavior ]\n---\nOld content.\n",
    )
    .expect("initial source");
    fs::write(
        &root,
        "---\ndependencies: [ knowledge-changed ]\n---\nRoot content.\n",
    )
    .expect("root source");
    fs::write(&removed, "To be removed.\n").expect("removed source");
    fs::write(&behavior, "Behavior instructions.\n").expect("transitive source");

    let _nexus = RunningNexus(
        Command::new(env!("CARGO_BIN_EXE_curriculum-nexus"))
            .env("XDG_RUNTIME_DIR", &runtime)
            .env("CURRICULUM_PSYCHES_SKILLS_DIR", &psyche)
            .env("CURRICULUM_MIND_SKILLS_DIR", &mind)
            .env("CURRICULUM_FIELD_SKILLS_DIR", &field)
            .env("CURRICULUM_WORKSPACE", &workspace)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Curriculum Nexus starts"),
    );
    wait_for_socket(&runtime.join("curriculum/curriculum.sock"));
    let empty_check = invoke_rejected(&runtime, "CheckSkills");
    assert!(
        empty_check
            .stderr
            .contains("generated skill projection differs"),
        "{}",
        empty_check.stderr
    );
    assert!(!workspace.join(".agents/skills").exists());
    let rebuilt = invoke(&runtime, "RebuildSkills");
    assert!(rebuilt.starts_with("SkillsRebuilt.["), "{rebuilt}");
    assert_eq!(
        invoke(&runtime, "ResolveSkills.[ operation-root ]"),
        "ResolvedSkills.[ compensation-behavior knowledge-changed operation-root ]"
    );
    assert!(invoke(&runtime, "CheckSkills").starts_with("SkillsChecked.["));

    for surface in surfaces() {
        assert!(
            workspace
                .join(surface)
                .join("knowledge-changed/SKILL.md")
                .is_file()
        );
        assert!(
            workspace
                .join(surface)
                .join("compensation-removed/SKILL.md")
                .is_file()
        );
    }

    fs::write(&changed, "Updated content.\n").expect("edit source");
    fs::write(&added, "New content.\n").expect("add source");
    fs::remove_file(&removed).expect("remove source");
    let request = format!(
        "EditSkills.{{ [ «{}» ] [ «{}» ] [ «{}» ] }}",
        added.display(),
        changed.display(),
        removed.display()
    );
    let response = invoke(&runtime, &request);
    assert!(response.contains("knowledge-changed"), "{response}");
    assert!(response.contains("operation-added"), "{response}");
    assert!(response.contains("compensation-removed"), "{response}");

    for surface in surfaces() {
        let root = workspace.join(surface);
        let edited =
            fs::read_to_string(root.join("knowledge-changed/SKILL.md")).expect("edited projection");
        let added_body =
            fs::read_to_string(root.join("operation-added/SKILL.md")).expect("new projection");
        assert!(edited.contains("Updated content."), "{surface}: {edited}");
        assert!(
            added_body.contains("New content."),
            "{surface}: {added_body}"
        );
        if surface == ".opencode/skills" {
            assert!(edited.contains("name: knowledge-changed"));
            assert!(added_body.contains("name: operation-added"));
        }
        assert!(!root.join("compensation-removed").exists());
    }
    assert_eq!(
        invoke(&runtime, "ResolveSkills.[ operation-added ]"),
        "ResolvedSkills.[ operation-added ]"
    );
    assert_eq!(
        invoke(&runtime, "ResolveSkills.[ operation-root ]"),
        "ResolvedSkills.[ knowledge-changed operation-root ]"
    );
    assert!(invoke(&runtime, "CheckSkills").starts_with("SkillsChecked.["));

    fs::write(&changed, "Rejected update.\n").expect("write rejected update");
    let saved_workspace = temporary.path().join("workspace-saved");
    fs::rename(&workspace, &saved_workspace).expect("hide workspace to reject projection");
    fs::write(&workspace, "projection unavailable").expect("block projection workspace");
    let rejected = invoke_rejected(
        &runtime,
        &format!("EditSkills.{{ [] [ «{}» ] [] }}", changed.display()),
    );
    assert!(rejected.stdout.is_empty(), "{}", rejected.stdout);
    assert!(
        rejected
            .stderr
            .contains("cannot write generated skill path"),
        "{}",
        rejected.stderr
    );
    fs::remove_file(&workspace).expect("remove blocking file");
    fs::rename(&saved_workspace, &workspace).expect("restore workspace");
    for surface in surfaces() {
        let projected =
            fs::read_to_string(workspace.join(surface).join("knowledge-changed/SKILL.md"))
                .expect("previous projection remains");
        assert!(
            projected.contains("Updated content."),
            "{surface}: {projected}"
        );
        assert!(
            !projected.contains("Rejected update."),
            "{surface}: {projected}"
        );
    }
    assert_eq!(
        invoke(&runtime, "ResolveSkills.[ knowledge-changed ]"),
        "ResolvedSkills.[ knowledge-changed ]"
    );
    let stale_source = invoke_rejected(&runtime, "CheckSkills");
    assert!(
        stale_source
            .stderr
            .contains("authored skill sources changed without an EditSkills signal"),
        "{}",
        stale_source.stderr
    );
    fs::write(&changed, "Updated content.\n").expect("restore accepted source");
    assert!(invoke(&runtime, "CheckSkills").starts_with("SkillsChecked.["));
}

#[test]
fn rebuild_projects_every_role_with_each_standing_skill_and_keeps_book_verbatim() {
    let temporary = tempdir().expect("temporary root");
    let runtime = temporary.path().join("runtime");
    let runtime_directory = runtime.join("curriculum");
    let psyche = temporary.path().join("psyche-skills");
    let mind = temporary.path().join("mind-skills");
    let field = temporary.path().join("field-skills");
    let workspace = temporary.path().join("workspace");
    let roles_file = temporary.path().join("roles.datom");
    for directory in [&runtime_directory, &psyche, &mind, &field, &workspace] {
        fs::create_dir_all(directory).expect("fixture directory");
    }
    let roles_source = include_str!("../roles.datom");
    fs::write(&roles_file, roles_source).expect("real role configuration");
    let roles = parse_roles(roles_source);
    let direct_markers = roles
        .second_string_vector
        .iter()
        .map(|name| (name.clone(), format!("STANDING_BODY<{name}>")))
        .collect::<Vec<_>>();
    let mut markers = direct_markers.clone();
    markers.extend([
        (
            "compensation-behavior".to_owned(),
            "TRANSITIVE_BODY<compensation-behavior>".to_owned(),
        ),
        (
            "compensation-correction".to_owned(),
            "TRANSITIVE_BODY<compensation-correction>".to_owned(),
        ),
        (
            "knowledge-vocabulary".to_owned(),
            "TRANSITIVE_BODY<knowledge-vocabulary>".to_owned(),
        ),
    ]);
    for (index, (name, marker)) in markers.iter().enumerate() {
        let root = match index % 3 {
            0 => &psyche,
            1 => &mind,
            _ => &field,
        };
        let dependencies = if name == "spirit" {
            "dependencies: [ compensation-behavior, compensation-correction, knowledge-vocabulary, compensation-book-distillation ]\n"
        } else {
            ""
        };
        fs::write(
            root.join(format!("{name}.md")),
            format!("---\ndescription: test\n{dependencies}---\n\n{marker}\n"),
        )
        .expect("standing skill source");
    }
    let procedure = "# Book procedure: preserve every byte.\n\nLine  2 stays as authored.\n";
    fs::create_dir_all(workspace.join("subagents")).expect("authored procedure directory");
    fs::write(workspace.join("subagents/book.md"), procedure).expect("authored Book procedure");

    let _nexus = RunningNexus(
        Command::new(env!("CARGO_BIN_EXE_curriculum-nexus"))
            .env("XDG_RUNTIME_DIR", &runtime)
            .env("CURRICULUM_PSYCHES_SKILLS_DIR", &psyche)
            .env("CURRICULUM_MIND_SKILLS_DIR", &mind)
            .env("CURRICULUM_FIELD_SKILLS_DIR", &field)
            .env("CURRICULUM_WORKSPACE", &workspace)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Curriculum Nexus starts"),
    );
    wait_for_socket(&runtime.join("curriculum/curriculum.sock"));
    let rebuilt = invoke(&runtime, "RebuildSkills");
    assert!(rebuilt.starts_with("SkillsRebuilt.["), "{rebuilt}");
    assert!(invoke(&runtime, "CheckSkills").starts_with("SkillsChecked.["));

    let packet_plan =
        curriculum::roles::PlansRoles::role_plan(&roles, &workspace).expect("role packet plan");
    assert_eq!(packet_plan.role_packet_plan_vector.len(), 24);
    assert_eq!(packet_plan.first_string_vector.len(), 10);
    assert_eq!(markers.len(), 13);
    for packet in &packet_plan.role_packet_plan_vector {
        let projected = fs::read_to_string(workspace.join(&packet.first_string))
            .expect("projected role packet");
        for (_, marker) in &markers {
            assert_eq!(
                projected.matches(marker).count(),
                1,
                "{}",
                packet.first_string
            );
        }
    }
    let book = fs::read_to_string(workspace.join(".claude/agents/book.md"))
        .expect("projected Book packet");
    assert!(
        book.ends_with(procedure),
        "Book procedure changed: {book:?}"
    );
}

fn parse_roles(source: &str) -> curriculum::generated::Roles {
    let mut potential = Potential::<RolesDocument>::from(source);
    let RolesDocument::Roles(roles) = potential
        .actualize(&mut Budget {
            remaining: 16_384,
            reader: ReaderBudget { remaining: 16_384 },
            depth: 0,
            maximum_depth: 16_384,
        })
        .expect("parse role configuration");
    roles
}

fn surfaces() -> [&'static str; 5] {
    [
        ".agents/skills",
        ".claude/skills",
        ".codex/skills",
        ".pi/skills",
        ".opencode/skills",
    ]
}

fn wait_for_socket(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if path.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("Curriculum Nexus did not create {}", path.display());
}

fn invoke(runtime: &Path, request: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_curriculum"))
        .arg(request)
        .env("XDG_RUNTIME_DIR", runtime)
        .env(
            "CURRICULUM_ROLES_FILE",
            runtime.parent().unwrap().join("roles.datom"),
        )
        .env(
            "CURRICULUM_WORKSPACE",
            runtime.parent().unwrap().join("workspace"),
        )
        .output()
        .expect("Curriculum CLI starts");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("CLI output is UTF-8")
        .trim()
        .to_owned()
}

struct RejectedOutput {
    stdout: String,
    stderr: String,
}

fn invoke_rejected(runtime: &Path, request: &str) -> RejectedOutput {
    let output = Command::new(env!("CARGO_BIN_EXE_curriculum"))
        .arg(request)
        .env("XDG_RUNTIME_DIR", runtime)
        .env(
            "CURRICULUM_ROLES_FILE",
            runtime.parent().unwrap().join("roles.datom"),
        )
        .env(
            "CURRICULUM_WORKSPACE",
            runtime.parent().unwrap().join("workspace"),
        )
        .output()
        .expect("Curriculum CLI starts");
    assert!(!output.status.success(), "CLI unexpectedly succeeded");
    RejectedOutput {
        stdout: String::from_utf8(output.stdout).expect("stdout is UTF-8"),
        stderr: String::from_utf8(output.stderr).expect("stderr is UTF-8"),
    }
}
