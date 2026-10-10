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
    let psyche_skill_sources = psyche.join("skills");
    let psyche_vision_sources = psyche.join("vision");
    let mind = temporary.path().join("mind-skills");
    let field = temporary.path().join("field-skills");
    let workspace = temporary.path().join("workspace");
    let roles_file = temporary.path().join("roles.datom");
    for directory in [
        &runtime_directory,
        &psyche,
        &psyche_skill_sources,
        &psyche_vision_sources,
        &mind,
        &field,
        &workspace,
    ] {
        fs::create_dir_all(directory).expect("fixture directory");
    }
    fs::write(psyche.join("README.md"), "Repository notes, not a skill.\n")
        .expect("repository README");
    fs::write(&roles_file, "Roles.{ [] [] [] [] [] [] [] [] [] }")
        .expect("empty role configuration");
    let old_ethos = psyche_skill_sources.join("vision-ethos.md");
    let ethos = psyche_vision_sources.join("ethos.md");
    let changed = mind.join("knowledge-changed.md");
    let root = mind.join("operation-root.md");
    let removed = field.join("compensation-removed.md");
    let behavior = field.join("compensation-behavior.md");
    let added = mind.join("operation-added.md");
    fs::write(
        &old_ethos,
        "---\ndependencies: [ compensation-behavior ]\n---\nOld ethos body.\n",
    )
    .expect("legacy Psyche source");
    fs::write(
        &changed,
        "---\ndependencies: [ compensation-behavior ]\n---\nOld content.\n",
    )
    .expect("initial source");
    fs::write(
        &root,
        "---\ndependencies: [ vision-ethos, knowledge-changed ]\n---\nRoot content.\n",
    )
    .expect("root source");
    fs::write(&removed, "To be removed.\n").expect("removed source");
    fs::write(&behavior, "Behavior instructions.\n").expect("transitive source");

    let _nexus = RunningNexus(
        Command::new(env!("CARGO_BIN_EXE_curriculum-nexus"))
            .env("XDG_RUNTIME_DIR", &runtime)
            .env("CURRICULUM_PSYCHES_REPOSITORY_DIR", &psyche)
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
    assert!(!rebuilt.contains("README"), "{rebuilt}");
    assert_eq!(
        invoke(&runtime, "ResolveSkills.[ operation-root ]"),
        "ResolvedSkills.[ compensation-behavior vision-ethos knowledge-changed operation-root ]"
    );
    assert_eq!(
        invoke(&runtime, "ResolveSkills.[ vision-ethos ]"),
        "ResolvedSkills.[ compensation-behavior vision-ethos ]"
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
        assert!(
            workspace
                .join(surface)
                .join("vision-ethos/SKILL.md")
                .is_file()
        );
        assert!(!workspace.join(surface).join("ethos/SKILL.md").exists());
    }

    fs::rename(&old_ethos, &ethos).expect("move Psyche source into its category");
    fs::write(
        &ethos,
        "---\ndependencies: [ compensation-behavior ]\n---\nUpdated ethos body.\n",
    )
    .expect("update moved Psyche source");
    let stale_paths = invoke_rejected(&runtime, "CheckSkills");
    assert!(
        stale_paths
            .stderr
            .contains("authored skill sources changed without an EditSkills signal"),
        "{}",
        stale_paths.stderr
    );
    fs::write(&changed, "Updated content.\n").expect("edit source");
    fs::write(&added, "New content.\n").expect("add source");
    fs::remove_file(&removed).expect("remove source");
    let request = format!(
        "EditSkills.{{ [ «{}» «{}» ] [ «{}» ] [ «{}» «{}» ] }}",
        added.display(),
        ethos.display(),
        changed.display(),
        removed.display(),
        old_ethos.display()
    );
    let response = invoke(&runtime, &request);
    assert!(response.contains("knowledge-changed"), "{response}");
    assert!(response.contains("vision-ethos"), "{response}");
    assert!(response.contains("operation-added"), "{response}");
    assert!(response.contains("compensation-removed"), "{response}");

    for surface in surfaces() {
        let root = workspace.join(surface);
        let edited =
            fs::read_to_string(root.join("knowledge-changed/SKILL.md")).expect("edited projection");
        let added_body =
            fs::read_to_string(root.join("operation-added/SKILL.md")).expect("new projection");
        let ethos_body =
            fs::read_to_string(root.join("vision-ethos/SKILL.md")).expect("ethos projection");
        assert!(edited.contains("Updated content."), "{surface}: {edited}");
        assert!(
            added_body.contains("New content."),
            "{surface}: {added_body}"
        );
        assert!(
            ethos_body.contains("Updated ethos body."),
            "{surface}: {ethos_body}"
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
        "ResolvedSkills.[ compensation-behavior vision-ethos knowledge-changed operation-root ]"
    );
    assert!(invoke(&runtime, "CheckSkills").starts_with("SkillsChecked.["));
    assert!(invoke(&runtime, "RebuildSkills").starts_with("SkillsRebuilt.["));
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
fn reopening_with_a_distinct_psyche_root_preserves_skill_and_role_projections() {
    let temporary = tempdir().expect("temporary root");
    let runtime = temporary.path().join("runtime");
    let runtime_directory = runtime.join("curriculum");
    let old_psyche = temporary.path().join("psyche-old");
    let old_skill_sources = old_psyche.join("skills");
    let new_psyche = temporary.path().join("psyche-new");
    let new_skill_sources = new_psyche.join("skills");
    let new_vision_sources = new_psyche.join("vision");
    let mind = temporary.path().join("mind-skills");
    let field = temporary.path().join("field-skills");
    let workspace = temporary.path().join("workspace");
    let roles_file = temporary.path().join("roles.datom");
    for directory in [
        &runtime_directory,
        &old_skill_sources,
        &new_skill_sources,
        &new_vision_sources,
        &mind,
        &field,
        &workspace,
    ] {
        fs::create_dir_all(directory).expect("fixture directory");
    }

    let roles_source = include_str!("../roles.datom");
    fs::write(&roles_file, roles_source).expect("real role configuration");
    let roles = parse_roles(roles_source);
    let mut markers = roles
        .second_string_vector
        .iter()
        .map(|name| (name.clone(), format!("STANDING_BODY<{name}>")))
        .collect::<Vec<_>>();
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
        (
            "vision-ethos".to_owned(),
            "VISION_ETHOS_BODY".to_owned(),
        ),
    ]);
    let ethos = "---\ndependencies: [ compensation-behavior ]\n---\n\nVISION_ETHOS_BODY\n";
    fs::write(old_skill_sources.join("vision-ethos.md"), ethos)
        .expect("old flat Psyche source");
    fs::write(new_vision_sources.join("ethos.md"), ethos)
        .expect("new categorized Psyche source");

    for (index, (name, marker)) in markers.iter().enumerate() {
        if name == "vision-ethos" {
            continue;
        }
        let root = match index % 3 {
            0 => &old_skill_sources,
            1 => &mind,
            _ => &field,
        };
        let dependencies = if name == "spirit" {
            "dependencies: [ compensation-behavior, compensation-correction, knowledge-vocabulary, compensation-book-distillation, vision-ethos ]\n"
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
    fs::create_dir_all(workspace.join("subagents")).expect("procedure directory");
    fs::write(workspace.join("subagents/book.md"), procedure).expect("Book procedure");

    let start_nexus = |psyche: &Path| {
        RunningNexus(
            Command::new(env!("CARGO_BIN_EXE_curriculum-nexus"))
                .env("XDG_RUNTIME_DIR", &runtime)
                .env("CURRICULUM_PSYCHES_REPOSITORY_DIR", psyche)
                .env("CURRICULUM_MIND_SKILLS_DIR", &mind)
                .env("CURRICULUM_FIELD_SKILLS_DIR", &field)
                .env("CURRICULUM_WORKSPACE", &workspace)
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Curriculum Nexus starts"),
        )
    };
    let first_nexus = start_nexus(&old_psyche);
    let socket = runtime.join("curriculum/curriculum.sock");
    wait_for_socket(&socket);
    assert!(invoke(&runtime, "RebuildSkills").starts_with("SkillsRebuilt.["));
    assert!(invoke(&runtime, "CheckSkills").starts_with("SkillsChecked.["));

    let resolved_before = invoke(&runtime, "ResolveSkills.[ spirit ]");
    assert!(
        resolved_before.split_whitespace().any(|name| name == "vision-ethos"),
        "{resolved_before}"
    );
    let names_before = resolved_before
        .strip_prefix("ResolvedSkills.[")
        .expect("typed resolver reply")
        .strip_suffix(']')
        .expect("typed resolver reply")
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let capture_skills = |names: &[String]| {
        let mut bodies = Vec::new();
        for surface in surfaces() {
            for name in names {
                let relative = format!("{surface}/{name}/SKILL.md");
                let body = fs::read_to_string(workspace.join(&relative))
                    .expect("projected resolved skill body");
                bodies.push((relative, body));
            }
        }
        bodies
    };
    let skills_before = capture_skills(&names_before);
    for (surface, body) in surfaces()
        .iter()
        .map(|surface| (*surface, fs::read_to_string(workspace.join(surface).join("vision-ethos/SKILL.md")).expect("projected ethos body")))
    {
        assert!(body.contains("VISION_ETHOS_BODY"), "{surface}: {body}");
    }
    let capture_packets = || {
        let plan = curriculum::roles::PlansRoles::role_plan(&roles, &workspace)
            .expect("role packet plan");
        plan.role_packet_plan_vector
            .iter()
            .map(|packet| {
                let name = packet.first_string.clone();
                let body = fs::read_to_string(workspace.join(&name)).expect("projected role packet");
                assert_eq!(body.matches("VISION_ETHOS_BODY").count(), 1, "{name}");
                (name, body)
            })
            .collect::<Vec<_>>()
    };
    let packets_before = capture_packets();
    assert_eq!(packets_before.len(), 24);

    drop(first_nexus);
    fs::remove_file(&socket).expect("remove stopped Nexus socket before restart");
    fs::write(
        old_skill_sources.join("vision-ethos.md"),
        "---\ndependencies: [ compensation-behavior ]\n---\n\nOLD_ROOT_BODY\n",
    )
    .expect("distinguish the stopped old-root source");
    assert_eq!(
        fs::read_to_string(new_vision_sources.join("ethos.md"))
            .expect("new categorized Psyche source remains"),
        ethos
    );
    let second_nexus = start_nexus(&new_psyche);
    wait_for_socket(&socket);
    assert!(invoke(&runtime, "RebuildSkills").starts_with("SkillsRebuilt.["));
    assert!(invoke(&runtime, "CheckSkills").starts_with("SkillsChecked.["));

    let resolved_after = invoke(&runtime, "ResolveSkills.[ spirit ]");
    assert_eq!(resolved_after, resolved_before);
    let skills_after = capture_skills(&names_before);
    assert_eq!(skills_after, skills_before);
    assert_eq!(capture_packets(), packets_before);
    drop(second_nexus);
}

#[test]
fn rebuild_projects_every_role_with_each_standing_skill_and_keeps_book_verbatim() {
    let temporary = tempdir().expect("temporary root");
    let runtime = temporary.path().join("runtime");
    let runtime_directory = runtime.join("curriculum");
    let psyche = temporary.path().join("psyche-skills");
    let psyche_skill_sources = psyche.join("skills");
    let mind = temporary.path().join("mind-skills");
    let field = temporary.path().join("field-skills");
    let workspace = temporary.path().join("workspace");
    let roles_file = temporary.path().join("roles.datom");
    for directory in [
        &runtime_directory,
        &psyche,
        &psyche_skill_sources,
        &mind,
        &field,
        &workspace,
    ] {
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
            0 => &psyche_skill_sources,
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
            .env("CURRICULUM_PSYCHES_REPOSITORY_DIR", &psyche)
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
