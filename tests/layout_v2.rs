use curriculum::registry::IdentityText;
mod support;
use curriculum::registry::ExpandsSkills;
use std::{fs, path::PathBuf};

fn roots(base: &std::path::Path) -> [PathBuf; 3] {
    let roots = [base.join("psyche"), base.join("mind"), base.join("field")];
    for root in &roots {
        fs::create_dir_all(root).unwrap();
    }
    roots
}
fn revisions() -> [String; 3] {
    ["a".repeat(40), "b".repeat(40), "c".repeat(40)]
}
fn source(root: &std::path::Path, relative: &str, body: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

#[test]
fn source_api_exposes_registry_address_and_declared_placement() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    source(&r[0], "vision/book/0.md", "Book vision.\n");
    source(
        &r[2],
        "compensation/illustration/0.md",
        "---\ndependencies: [ vision:book:0 ]\nplacement: system\n---\nExact approved text.\n",
    );
    let registry = support::registry(&r, &revisions()).unwrap();
    let ids = vec!["compensation:illustration:0".into()];
    let resolved = registry.resolved_sources(&ids).unwrap();
    assert_eq!(resolved.len(), 2);
    assert_eq!(resolved[1].repository_name.0, "field-belief");
    assert_eq!(
        resolved[1].source_relative_path.0,
        "compensation/illustration/0.md"
    );
    assert_eq!(resolved[1].source_revision.0, "c".repeat(40));
    assert_eq!(resolved[1].declared_placement.0, "system");
    assert_eq!(resolved[1].string, "Exact approved text.\n");
    assert_eq!(registry.resolve_exact(&ids).unwrap().len(), 1);
}

#[test]
fn dependencies_require_explicit_three_field_ids_and_reject_duplicates() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    source(&r[0], "vision/book-making/0.md", "Book making.\n");
    source(&r[0], "spirit/core/0.md", "Core.\n");
    source(
        &r[2],
        "compensation/illustration/0.md",
        "---\ndependencies: [vision:book-making:0, spirit:core:0]\n---\nIllustrate.\n",
    );
    let registry = support::registry(&r, &revisions()).unwrap();
    let resolved = registry
        .resolved_sources(&["compensation:illustration:0".into()])
        .unwrap();
    let ids: Vec<String> = resolved
        .iter()
        .map(|module| module.module_identity.identity_text().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "vision:book-making:0",
            "spirit:core:0",
            "compensation:illustration:0"
        ]
    );

    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    source(&r[0], "spirit/core/0.md", "Core.\n");
    source(
        &r[2],
        "compensation/twice/0.md",
        "---\ndependencies: [ spirit:core:0 spirit:core:0 ]\n---\nTwice.\n",
    );
    assert!(support::registry(&r, &revisions()).is_err());
}

#[test]
fn a_missing_dependency_still_refuses() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    source(
        &r[1],
        "knowledge/missing/0.md",
        "---\ndependencies: [ knowledge:absent:0 ]\n---\nMissing.\n",
    );
    assert!(support::registry(&r, &revisions()).is_err());
}

#[test]
fn same_subject_different_types_are_distinct() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    source(&r[1], "knowledge/topic/0.md", "Knowledge.");
    source(&r[1], "inquiry/topic/0.md", "Inquiry.");
    let registry = support::registry(&r, &revisions()).unwrap();
    assert_eq!(
        registry.skill_names().unwrap(),
        vec!["inquiry:topic:0", "knowledge:topic:0"]
    );
    assert!(registry.resolve_exact(&["topic".into()]).is_err());
}

#[test]
fn registry_workspace_projection_updates_plain_target_directories() {
    use curriculum::registry::workspace::WorkspaceProjector;

    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    let source_path = r[1].join("knowledge/topic/0.md");
    source(
        &r[1],
        "knowledge/topic/0.md",
        "---\ndescription: topic\nuser-only: true\n---\nBase.\n{% if claude %}\nClaude-only.\n{% else %}\nOther.\n{% endif %}\n{% if pi %}\nPi-only.\n{% endif %}\n{% raw %}\n{% if claude %}\n{% endraw %}\n",
    );
    let workspace = temp.path().join("workspace");
    let projector = WorkspaceProjector::new(&workspace);

    let first = support::registry(&r, &revisions()).unwrap();
    let published = projector.project(&first).unwrap();
    assert_eq!(
        published.deployment_destination.0,
        fs::canonicalize(&workspace).unwrap().to_string_lossy()
    );
    assert_eq!(published.repository_revision_vector.len(), 3);
    assert_eq!(
        published.repository_revision_vector[0].repository_name.0,
        "psyche-context"
    );
    assert_eq!(
        published.repository_revision_vector[0].source_revision.0,
        revisions()[0]
    );
    assert_eq!(
        published.renderer_version.0,
        "curriculum-workspace-projector-v1"
    );
    assert_eq!(published.published_inventory_digest.0.len(), 64);
    assert_eq!(published.published_content_digest.0.len(), 64);

    assert!(
        fs::read_to_string(workspace.join(".agents/skills/knowledge:topic:0/SKILL.md"))
            .unwrap()
            .contains("Other.\n")
    );
    assert!(
        fs::read_to_string(workspace.join(".claude/skills/knowledge:topic:0/SKILL.md"))
            .unwrap()
            .contains("disable-model-invocation: true\n")
    );
    assert_eq!(
        fs::read_to_string(workspace.join(".agents/skills/knowledge:topic:0/agents/openai.yaml"))
            .unwrap(),
        "policy:\n  allow_implicit_invocation: false\n"
    );
    assert_eq!(
        fs::read_to_string(workspace.join(".codex/skills/knowledge:topic:0/agents/openai.yaml"))
            .unwrap(),
        "policy:\n  allow_implicit_invocation: false\n"
    );
    assert!(
        fs::read_to_string(workspace.join(".opencode/skills/knowledge:topic:0/SKILL.md"))
            .unwrap()
            .starts_with("---\nname: knowledge:topic:0\n")
    );
    for surface in [".codex/skills", ".pi/skills"] {
        assert!(
            workspace
                .join(surface)
                .join("knowledge:topic:0/SKILL.md")
                .is_file()
        );
    }
    assert!(
        fs::read_to_string(workspace.join(".pi/skills/knowledge:topic:0/SKILL.md"))
            .unwrap()
            .contains("Pi-only.\n")
    );
    assert!(
        fs::read_to_string(workspace.join(".pi/skills/knowledge:topic:0/SKILL.md"))
            .unwrap()
            .contains("{% if claude %}\n")
    );
    assert_eq!(
        fs::read_to_string(workspace.join("modules")).unwrap(),
        format!(
            "knowledge:topic:0 {} knowledge/topic/0.md\n",
            revisions()[1]
        )
    );

    let stale = workspace.join(".agents/skills/retired/SKILL.md");
    fs::create_dir_all(stale.parent().unwrap()).unwrap();
    fs::write(&stale, "stale").unwrap();
    let unrelated = workspace.join(".agents/skills/local/notes.txt");
    fs::create_dir_all(unrelated.parent().unwrap()).unwrap();
    fs::write(&unrelated, "preserve").unwrap();

    fs::write(&source_path, "Latest module text.\n").unwrap();
    let latest = support::registry(&r, &revisions()).unwrap();
    let refreshed = projector.project(&latest).unwrap();
    assert_eq!(
        refreshed.published_inventory_digest,
        published.published_inventory_digest
    );
    assert_ne!(
        refreshed.published_content_digest,
        published.published_content_digest
    );
    assert_eq!(
        fs::read_to_string(workspace.join(".agents/skills/knowledge:topic:0/SKILL.md")).unwrap(),
        "Latest module text.\n"
    );
    // A directory not named in the previous generated inventory is user-owned.
    assert_eq!(fs::read_to_string(&stale).unwrap(), "stale");
    assert_eq!(fs::read_to_string(unrelated).unwrap(), "preserve");
    assert!(
        fs::read_dir(workspace.join(".agents/skills/knowledge:topic:0"))
            .unwrap()
            .all(|entry| !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp"))
    );

    let outside = temp.path().join("outside");
    fs::create_dir_all(&outside).unwrap();
    let linked_workspace = temp.path().join("linked-workspace");
    fs::create_dir_all(&linked_workspace).unwrap();
    std::os::unix::fs::symlink(&outside, linked_workspace.join(".agents")).unwrap();
    assert!(
        WorkspaceProjector::new(&linked_workspace)
            .write(&latest)
            .is_err()
    );
    assert_eq!(fs::read_dir(outside).unwrap().count(), 0);
}

#[test]
fn registry_projection_covers_all_module_kinds_and_harness_surfaces() {
    use curriculum::registry::{MODULE_KINDS, Worded, workspace::WorkspaceProjector};
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    for (index, kind) in MODULE_KINDS.iter().enumerate() {
        source(
            &r[index % 3],
            &format!("{}/item-{index}/0.md", kind.word()),
            "---\ndescription: fixture\n  user-only: true  \n---\n{% if claude %}\nclaude\n{% endif %}\n{% if codex %}\ncodex\n{% endif %}\n{% if pi %}\npi\n{% endif %}\n{% if opencode %}\nopencode\n{% endif %}\n",
        );
    }
    let registry = support::registry(&r, &revisions()).unwrap();
    let ids = registry.skill_names().unwrap();
    assert_eq!(ids.len(), 9);
    assert!(ids.iter().any(|id| id == "notion:item-8:0"));
    let workspace = temp.path().join("workspace");
    WorkspaceProjector::new(&workspace)
        .write(&registry)
        .unwrap();
    let inventory = fs::read_to_string(workspace.join("modules")).unwrap();
    let lines = inventory.lines().collect::<Vec<_>>();
    let mut sorted = lines.clone();
    sorted.sort();
    assert_eq!(lines, sorted);
    for id in ids {
        for (surface, marker) in [
            (".claude/skills", "claude"),
            (".agents/skills", "codex"),
            (".codex/skills", "codex"),
            (".pi/skills", "pi"),
            (".opencode/skills", "opencode"),
        ] {
            let rendered =
                fs::read_to_string(workspace.join(surface).join(&id).join("SKILL.md")).unwrap();
            assert!(rendered.contains(marker), "{surface}/{id}");
            assert!(
                !rendered.contains("{% if "),
                "unrendered directive in {surface}/{id}"
            );
        }
        for surface in [".agents/skills", ".codex/skills"] {
            assert!(
                workspace
                    .join(surface)
                    .join(&id)
                    .join("agents/openai.yaml")
                    .is_file()
            );
        }
        assert!(
            fs::read_to_string(workspace.join(".claude/skills").join(&id).join("SKILL.md"))
                .unwrap()
                .contains("disable-model-invocation: true")
        );
    }
}

#[test]
fn explicit_versions_are_distinct_and_dependencies_select_exactly_one() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    source(&r[0], "vision/topic/0.md", "Version zero.\n");
    source(&r[0], "vision/topic/12.md", "Version twelve.\n");
    source(
        &r[1],
        "knowledge/consumer/0.md",
        "---\ndependencies: [ vision:topic:0 ]\n---\nConsumer.\n",
    );
    let registry = support::registry(&r, &revisions()).unwrap();
    let selected = registry
        .resolved_sources(&["knowledge:consumer:0".into()])
        .unwrap();
    assert_eq!(
        selected
            .iter()
            .map(|m| m.module_identity.identity_text().unwrap())
            .collect::<Vec<_>>(),
        ["vision:topic:0", "knowledge:consumer:0"]
    );
    assert_eq!(selected[0].source_relative_path.0, "vision/topic/0.md");
    assert_eq!(selected[0].module_identity.module_subject.0, "topic");
    assert_eq!(
        registry.resolve_exact(&["vision:topic:12".into()]).unwrap()[0].body,
        "Version twelve.\n"
    );
    for id in [
        "vision:topic",
        "vision-topic",
        "vision:topic:1",
        "vision:topic:latest",
    ] {
        assert!(registry.resolve_exact(&[id.into()]).is_err(), "{id}");
    }
}

#[test]
fn versioned_identity_validation_has_no_implicit_or_alias_form() {
    use curriculum::registry::valid_id;
    for id in [
        "vision:topic:0",
        "knowledge:a-b:42",
        "spirit:core:1",
        "vision:topic:0-1",
        "knowledge:a-b:2-42",
    ] {
        assert!(valid_id(id), "{id}");
    }
    for id in [
        "vision:topic",
        "vision-topic-0",
        "vision-topic:0",
        "vision:topic:",
        "vision:topic:00",
        "vision:topic:01",
        "vision:topic:0-0",
        "vision:topic:2-1",
        "vision:topic:00-1",
        "vision:topic:0-01",
        "vision:topic:0-1-2",
        "vision:topic:-1",
        "vision:topic:+1",
        "vision:topic:latest",
        "vision:topic:0:1",
        "vision:Topic:0",
        "vision::0",
        "unknown:topic:0",
        "vision:../topic:0",
        "vision:topic:١",
    ] {
        assert!(!valid_id(id), "{id}");
    }
}

#[test]
fn dependencies_refuse_old_aliases_and_missing_versions() {
    for dependency in [
        "vision:topic",
        "vision-topic",
        "vision-topic-0",
        "vision:topic:1",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let r = roots(temp.path());
        source(&r[0], "vision/topic/0.md", "Topic.\n");
        source(
            &r[1],
            "knowledge/consumer/0.md",
            &format!("---\ndependencies: [ {dependency} ]\n---\nConsumer.\n"),
        );
        assert!(support::registry(&r, &revisions()).is_err(), "{dependency}");
    }
}

#[test]
fn a_specialist_is_typed_but_not_disclosed_in_a_public_projection_or_source_query() {
    use curriculum::{
        SkillVisibility, generated::ModulePlacement, module_placement,
        registry::workspace::WorkspaceProjector,
    };
    use signal_curriculum_nexus::RoleName;

    let private = "---\ndescription: Private fixture\nplacement: Specialist.[ FlowDriver SecondRole ]\n---\nThe protected fixture body.\n";
    let placed = module_placement(private).unwrap();
    assert_eq!(
        placed,
        ModulePlacement::Specialist(vec![
            RoleName("FlowDriver".into()),
            RoleName("SecondRole".into())
        ])
    );
    assert!(!placed.is_public_skill());
    for malformed in [
        "placement: Specialist.[ ]",
        "placement: Specialist.[ One One ]",
        " placement: Specialist.[ One ]",
        "placement: Specialist.[ One ]\nplacement: system",
        "placement: Specialist.[ One, Two ]",
    ] {
        assert!(module_placement(&format!("---\n{malformed}\n---\nBody.\n")).is_err());
    }

    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    source(
        &r[0],
        "knowledge/public/0.md",
        "---\ndescription: Public fixture\n---\nA normal skill.\n",
    );
    source(&r[1], "knowledge/private/0.md", private);
    source(
        &r[2],
        "knowledge/dependency/0.md",
        "---\ndescription: Public dependency fixture\ndependencies: [ knowledge:private:0 ]\n---\nDepends on a specialist.\n",
    );
    let registry = support::registry(&r, &revisions()).unwrap();
    let private = ["knowledge:private:0".into()];
    assert!(registry.resolve_exact(&private).is_err());
    assert!(registry.resolved_exact_sources(&private).is_err());
    assert!(registry.resolved_sources(&private).is_err());
    assert!(registry.expand_skills(&private).is_err());
    assert!(
        registry
            .resolved_sources(&["knowledge:dependency:0".into()])
            .is_err()
    );
    assert_eq!(
        registry.skill_names().unwrap(),
        ["knowledge:dependency:0", "knowledge:public:0"]
    );
    assert_eq!(registry.listed_modules().unwrap().len(), 2);
    assert_eq!(registry.listed_modules_verbose().unwrap().len(), 2);
    assert_eq!(
        registry
            .resolved_sources(&["knowledge:public:0".into()])
            .unwrap()[0]
            .string,
        "A normal skill.\n"
    );

    let workspace = temp.path().join("published");
    fs::create_dir_all(&workspace).unwrap();
    fs::write(
        workspace.join("modules"),
        "knowledge:private:0 old k/private/0.md\n",
    )
    .unwrap();
    let targets = [".claude", ".agents", ".codex", ".pi", ".opencode"];
    for target in targets {
        let old = workspace.join(target).join("skills/knowledge:private:0");
        fs::create_dir_all(&old).unwrap();
        fs::write(old.join("SKILL.md"), "Previously published fixture").unwrap();
    }
    WorkspaceProjector::new(&workspace)
        .write(&registry)
        .unwrap();
    for target in targets {
        let skills = workspace.join(target).join("skills");
        assert!(!skills.join("knowledge:private:0").exists());
        assert!(skills.join("knowledge:public:0/SKILL.md").is_file());
    }
    assert!(
        !fs::read_to_string(workspace.join("modules"))
            .unwrap()
            .contains("knowledge:private:0")
    );
}

#[test]
fn public_startup_composition_does_not_grant_a_claimed_specialist_role() {
    use curriculum::registry::ModuleIdentityConversion;
    use signal_curriculum_nexus::{
        CallerMetaflow, StartupSelection, SystemModules, Target, UserModules,
    };
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    source(&r[0], "spirit/core/0.md", "Public Spirit.\n");
    source(&r[1], "intent/test/0.md", "Public seat intent.\n");
    source(
        &r[1],
        "knowledge/private/0.md",
        "---\nplacement: Specialist.[ FlowDriver ]\n---\nSynthetic marker.\n",
    );
    let registry = support::registry(&r, &revisions()).unwrap();
    let mut selection = StartupSelection {
        caller_metaflow: CallerMetaflow {
            first_string: "Mind".into(),
            second_string: "FlowDriver".into(),
            third_string: "Sol".into(),
        },
        target: Target::Codex,
        system_modules: SystemModules(vec![
            "spirit:core:0".module_identity().unwrap(),
            "intent:test:0".module_identity().unwrap(),
        ]),
        user_modules: UserModules(vec![]),
        flow_id_vector: vec![],
    };
    assert!(curriculum::startup::compose(&registry, temp.path(), &selection).is_ok());
    selection
        .user_modules
        .0
        .push("knowledge:private:0".module_identity().unwrap());
    assert!(curriculum::startup::compose(&registry, temp.path(), &selection).is_err());
}
