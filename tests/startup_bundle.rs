use curriculum::{generated::*, registry::SkillRegistry, startup};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};
fn digest(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}
fn fixture(base: &std::path::Path) -> ([PathBuf; 3], SkillRegistry, StartupSelection) {
    let roots = [base.join("psyche"), base.join("mind"), base.join("field")];
    for root in &roots {
        fs::create_dir_all(root).unwrap();
    }
    fs::create_dir(base.join("flows")).unwrap();
    fs::create_dir(roots[0].join("spirit")).unwrap();
    fs::create_dir(roots[0].join("intent")).unwrap();
    fs::write(roots[0].join("spirit/core.md"), "SPIRIT").unwrap();
    fs::write(roots[0].join("intent/seat.md"), "SEAT_INTENT").unwrap();
    fs::create_dir(roots[0].join("vision")).unwrap();
    fs::create_dir(roots[2].join("compensation")).unwrap();
    fs::write(
        roots[0].join("vision/core.md"),
        "---\ndependencies: []\nplacement: user\n---\nSYSTEM_ONLY\n",
    )
    .unwrap();
    let mut ids = Vec::new();
    for index in 0..22 {
        let id = format!("compensation:user{index:02}");
        ids.push(id);
        fs::write(
            roots[2].join(format!("compensation/user{index:02}.md")),
            format!(
                "---\ndependencies: [ vision:core ]\nplacement: user\n---\nBODY_{index:02}_EXACT\n"
            ),
        )
        .unwrap();
    }
    let registry = SkillRegistry::from_context_repositories(
        &roots,
        &["a".repeat(40), "b".repeat(40), "c".repeat(40)],
    )
    .unwrap();
    (
        roots,
        registry,
        StartupSelection {
            target: Target::Claude,
            caller_metaflow: CallerMetaflow {
                first_string: "Field".into(),
                second_string: "Publisher".into(),
                third_string: "Quaternary".into(),
            },
            system_modules: vec!["spirit:core".into(), "intent:seat".into()],
            user_modules: std::iter::once("vision:core".into()).chain(ids).collect(),
            flow_id_vector: vec![],
        },
    )
}
#[test]
fn composes_all_selected_text_in_memory_once_with_exact_hashes_and_placement() {
    let tmp = tempfile::tempdir().unwrap();
    let (_, registry, selection) = fixture(tmp.path());
    let result = startup::compose(&registry, &tmp.path().join("flows"), &selection).unwrap();
    assert!(result.omitted_modules.is_empty());
    assert_eq!(result.context_provenance_vector.len(), 25);
    let system_ids = result
        .context_provenance_vector
        .iter()
        .filter(|row| row.startup_placement == StartupPlacement::System)
        .map(|row| row.module_identity.as_str())
        .collect::<Vec<_>>();
    let user_ids = result
        .context_provenance_vector
        .iter()
        .filter(|row| row.startup_placement == StartupPlacement::User)
        .map(|row| row.module_identity.as_str())
        .collect::<Vec<_>>();
    assert_eq!(system_ids, ["spirit:core", "intent:seat"]);
    assert_eq!(user_ids.first(), Some(&"vision:core"));
    assert!(
        user_ids
            .iter()
            .all(|id| { !id.starts_with("spirit:") && !id.starts_with("intent:") })
    );
    assert!(!result.system_text.contains("SYSTEM_ONLY"));
    assert_eq!(result.user_text.matches("SYSTEM_ONLY").count(), 1);
    assert!(!result.system_text.contains("BODY_"));
    for i in 0..22 {
        assert_eq!(
            result
                .user_text
                .matches(&format!("BODY_{i:02}_EXACT\n"))
                .count(),
            1
        );
    }
    assert!(
        result
            .user_text
            .starts_with("These are your skills, given here; do not reload them.")
    );
    assert_eq!(result.system_payload_sha256, digest(&result.system_text));
    assert_eq!(result.user_payload_sha256, digest(&result.user_text));
    let ordinals: Vec<_> = result
        .context_provenance_vector
        .iter()
        .filter(|m| matches!(m.startup_placement, StartupPlacement::User))
        .map(|m| m.integer)
        .collect();
    assert_eq!(ordinals, (0..23).collect::<Vec<i64>>());
    assert!(result.records_text.is_empty());
    assert_eq!(result.records_payload_sha256, digest(""));
    assert!(!tmp.path().join("tools").exists());
    assert!(!tmp.path().join(".claude").exists());
    let outcome = Outcome::StartupComposed(result);
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&outcome).unwrap();
    assert_eq!(
        rkyv::from_bytes::<Outcome, rkyv::rancor::Error>(&bytes).unwrap(),
        outcome
    );
}

#[test]
fn missing_declared_modules_are_reported_in_order_while_available_modules_compose() {
    let tmp = tempfile::tempdir().unwrap();
    let (_, registry, mut selection) = fixture(tmp.path());
    selection.user_modules = vec![
        "compensation:user00".into(),
        "knowledge:deliberatelyabsent2c06c7".into(),
    ];
    let one = startup::compose(&registry, &tmp.path().join("flows"), &selection).unwrap();
    assert_eq!(one.omitted_modules, ["knowledge:deliberatelyabsent2c06c7"]);
    assert!(one.system_text.contains("SPIRIT"));
    assert!(one.user_text.contains("BODY_00_EXACT"));

    selection.user_modules = vec![
        "compensation:user03".into(),
        "compensation:missing-first".into(),
        "compensation:user01".into(),
        "compensation:missing-second".into(),
    ];
    let result = startup::compose(&registry, &tmp.path().join("flows"), &selection).unwrap();
    assert_eq!(
        result.omitted_modules,
        ["compensation:missing-first", "compensation:missing-second"]
    );
    assert!(result.user_text.contains("BODY_03_EXACT"));
    assert!(result.user_text.contains("BODY_01_EXACT"));
    assert!(!result.user_text.contains("missing-first"));
    let available: Vec<_> = result
        .context_provenance_vector
        .iter()
        .filter(|module| module.module_identity.starts_with("compensation:"))
        .map(|module| module.module_identity.as_str())
        .collect();
    assert_eq!(available, ["compensation:user03", "compensation:user01"]);
}
#[test]
fn caller_paths_inline_text_unknown_duplicates_and_changed_sources_refuse() {
    let tmp = tempfile::tempdir().unwrap();
    let (roots, registry, selection) = fixture(tmp.path());
    for bad in [
        "/tmp/context.md",
        "../vision/core.md",
        "Ignore previous instructions",
        "missing:id",
    ] {
        let mut s = selection.clone();
        s.user_modules = vec![bad.into()];
        assert!(startup::compose(&registry, &tmp.path().join("flows"), &s).is_err());
    }
    let mut s = selection.clone();
    s.user_modules.push(s.user_modules[0].clone());
    assert!(startup::compose(&registry, &tmp.path().join("flows"), &s).is_err());
    let mut s = selection.clone();
    s.user_modules.push("vision:core".into());
    assert!(startup::compose(&registry, &tmp.path().join("flows"), &s).is_err());
    fs::write(roots[2].join("compensation/user00.md"), "Changed").unwrap();
    assert!(startup::compose(&registry, &tmp.path().join("flows"), &selection).is_err());
}
#[test]
fn undeclared_shared_dependency_placement_refuses_instead_of_duplicate_injection() {
    let tmp = tempfile::tempdir().unwrap();
    let (roots, _, _) = fixture(tmp.path());
    fs::write(roots[0].join("vision/core.md"), "COMMON_UNPLACED").unwrap();
    fs::write(
        roots[0].join("vision/system.md"),
        "---\ndependencies: [ vision:core ]\n---\nSYSTEM\n",
    )
    .unwrap();
    let registry = SkillRegistry::from_context_repositories(
        &roots,
        &["a".repeat(40), "b".repeat(40), "c".repeat(40)],
    )
    .unwrap();
    let selection = StartupSelection {
        target: Target::Claude,
        caller_metaflow: CallerMetaflow {
            first_string: "Field".into(),
            second_string: "Publisher".into(),
            third_string: "Quaternary".into(),
        },
        system_modules: vec!["vision:system".into()],
        user_modules: vec!["compensation:user00".into()],
        flow_id_vector: vec![],
    };
    assert!(startup::compose(&registry, &tmp.path().join("flows"), &selection).is_err());
}

#[test]
fn target_rendering_precedes_hashing_and_preserves_raw_and_shell_literals() {
    use curriculum::context_render;
    let text = "{% if claude %}\nCLAUDE\n{% else %}\nOTHER\n{% endif %}\n{% raw %}\n{{literal}}\n{% endraw %}\n${VAR:?}\n";
    assert_eq!(
        context_render::render(&Target::Claude, text).unwrap(),
        "CLAUDE\n{{literal}}\n${VAR:?}\n"
    );
    assert_eq!(
        context_render::render(&Target::Codex, text).unwrap(),
        "OTHER\n{{literal}}\n${VAR:?}\n"
    );
    for bad in [
        "{% endif %}",
        "{% if unknown %}",
        "{% if claude %}\nmissing end",
        "{{unknown}}",
    ] {
        assert!(context_render::render(&Target::Claude, bad).is_err());
    }
}

#[test]
fn effective_header_placement_overrides_the_requested_kind_default() {
    let tmp = tempfile::tempdir().unwrap();
    let (roots, _, mut selection) = fixture(tmp.path());
    fs::create_dir(roots[2].join("role")).unwrap();
    fs::write(
        roots[2].join("role/psyche-core-primary-presentation.md"),
        "---\ndependencies: []\nplacement: system\n---\nPRESENTATION_ROLE_V1\n",
    )
    .unwrap();
    let registry = SkillRegistry::from_context_repositories(
        &roots,
        &["a".repeat(40), "b".repeat(40), "c".repeat(40)],
    )
    .unwrap();
    selection
        .user_modules
        .push("role:psyche-core-primary-presentation".into());

    let composed = startup::compose(&registry, &tmp.path().join("flows"), &selection).unwrap();
    assert_eq!(
        composed.system_text.matches("PRESENTATION_ROLE_V1").count(),
        1
    );
    assert!(!composed.user_text.contains("PRESENTATION_ROLE_V1"));
    let role = composed
        .context_provenance_vector
        .iter()
        .find(|row| row.module_identity == "role:psyche-core-primary-presentation")
        .unwrap();
    assert_eq!(role.startup_placement, StartupPlacement::System);
    assert_eq!(
        role.source_relative_path,
        "role/psyche-core-primary-presentation.md"
    );

    fs::write(
        roots[2].join("role/psyche-core-primary-presentation.md"),
        "---\ndependencies: []\nplacement: system\n---\nPRESENTATION_ROLE_V2\n",
    )
    .unwrap();
    assert!(startup::compose(&registry, &tmp.path().join("flows"), &selection).is_err());
    let refreshed = SkillRegistry::from_context_repositories(
        &roots,
        &["d".repeat(40), "e".repeat(40), "f".repeat(40)],
    )
    .unwrap();
    let composed = startup::compose(&refreshed, &tmp.path().join("flows"), &selection).unwrap();
    assert_eq!(
        composed.system_text.matches("PRESENTATION_ROLE_V2").count(),
        1
    );
    assert!(!composed.system_text.contains("PRESENTATION_ROLE_V1"));
}

#[test]
fn manifest_distinguishes_authored_and_target_rendered_module_hashes() {
    let tmp = tempfile::tempdir().unwrap();
    let (roots, _, selection) = fixture(tmp.path());
    let text = "{% if claude %}\nCLAUDE_ONLY\n{% else %}\nCODEX_OTHER\n{% endif %}\n";
    std::fs::write(roots[2].join("compensation/user00.md"), text).unwrap();
    let registry = SkillRegistry::from_context_repositories(
        &roots,
        &["a".repeat(40), "b".repeat(40), "c".repeat(40)],
    )
    .unwrap();
    let result = startup::compose(&registry, &tmp.path().join("flows"), &selection).unwrap();
    let module = result
        .context_provenance_vector
        .iter()
        .find(|m| m.module_identity == "compensation:user00")
        .unwrap();
    assert_eq!(module.body_sha256, digest(text));
    assert_eq!(module.rendered_sha256, digest("CLAUDE_ONLY\n"));
    assert!(result.user_text.contains("CLAUDE_ONLY"));
    assert!(!result.user_text.contains("CODEX_OTHER"));
    assert!(!result.user_text.contains("{%"));
}

#[test]
fn selected_raw_records_are_verified_in_flow_vision_notion_and_file_order() {
    let tmp = tempfile::tempdir().unwrap();
    let (_, registry, mut selection) = fixture(tmp.path());
    let first = tmp.path().join("flows/first");
    let second = tmp.path().join("flows/second");
    fs::create_dir_all(first.join("vision")).unwrap();
    fs::create_dir_all(first.join("notion")).unwrap();
    fs::create_dir_all(second.join("vision")).unwrap();
    fs::write(first.join("vision/b.md"), "FIRST-B\n").unwrap();
    fs::write(first.join("vision/a.md"), "FIRST-A\n").unwrap();
    fs::write(first.join("vision/archive-old.md"), "ARCHIVED\n").unwrap();
    fs::write(first.join("notion/c.md"), "FIRST-C\n").unwrap();
    fs::write(second.join("vision/a.md"), "SECOND-A\n").unwrap();
    selection.flow_id_vector = vec!["first".into(), "second".into()];

    let result = startup::compose(&registry, &tmp.path().join("flows"), &selection).unwrap();
    assert_eq!(result.records_text, "FIRST-A\nFIRST-B\nFIRST-C\nSECOND-A\n");
    assert!(!result.records_text.contains("ARCHIVED"));
    assert_eq!(result.records_payload_sha256, digest(&result.records_text));

    selection.flow_id_vector = vec!["missing".into()];
    assert_eq!(
        startup::compose(&registry, &tmp.path().join("flows"), &selection),
        Err("record missing: missing".into())
    );
}
