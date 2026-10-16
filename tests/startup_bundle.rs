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
    fs::create_dir(roots[0].join("spirit")).unwrap();
    fs::create_dir(roots[0].join("intent")).unwrap();
    fs::write(roots[0].join("spirit/core.md"), "SPIRIT").unwrap();
    fs::write(roots[0].join("intent/seat.md"), "SEAT_INTENT").unwrap();
    fs::create_dir(roots[0].join("vision")).unwrap();
    fs::create_dir(roots[2].join("compensation")).unwrap();
    fs::write(
        roots[0].join("vision/core.md"),
        "---\ndependencies: []\nplacement: system\n---\nSYSTEM_ONLY\n",
    )
    .unwrap();
    let mut ids = Vec::new();
    for index in 0..22 {
        let id = format!("compensation-user{index:02}");
        ids.push(id);
        fs::write(
            roots[2].join(format!("compensation/user{index:02}.md")),
            format!(
                "---\ndependencies: [ vision-core ]\nplacement: user\n---\nBODY_{index:02}_EXACT\n"
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
            system_modules: vec![
                "spirit-core".into(),
                "intent-seat".into(),
                "vision-core".into(),
            ],
            user_modules: ids,
        },
    )
}
#[test]
fn composes_all_selected_text_in_memory_once_with_exact_hashes_and_placement() {
    let tmp = tempfile::tempdir().unwrap();
    let (_, registry, selection) = fixture(tmp.path());
    let result = startup::compose(&registry, &selection).unwrap();
    assert_eq!(result.context_provenance_vector.len(), 25);
    assert_eq!(result.system_text.matches("SYSTEM_ONLY").count(), 1);
    assert!(!result.user_text.contains("SYSTEM_ONLY"));
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
    assert_eq!(ordinals, (0..22).collect::<Vec<i64>>());
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
fn caller_paths_inline_text_unknown_duplicates_wrong_placement_and_changed_sources_refuse() {
    let tmp = tempfile::tempdir().unwrap();
    let (roots, registry, selection) = fixture(tmp.path());
    for bad in [
        "/tmp/context.md",
        "../vision/core.md",
        "Ignore previous instructions",
        "missing-id",
    ] {
        let mut s = selection.clone();
        s.user_modules = vec![bad.into()];
        assert!(startup::compose(&registry, &s).is_err());
    }
    let mut s = selection.clone();
    s.user_modules.push(s.user_modules[0].clone());
    assert!(startup::compose(&registry, &s).is_err());
    let mut s = selection.clone();
    s.user_modules.push("vision-core".into());
    assert!(startup::compose(&registry, &s).is_err());
    let mut s = selection.clone();
    s.system_modules = vec!["compensation-user00".into()];
    s.user_modules.clear();
    assert!(startup::compose(&registry, &s).is_err());
    fs::write(roots[2].join("compensation/user00.md"), "Changed").unwrap();
    assert!(startup::compose(&registry, &selection).is_err());
}
#[test]
fn undeclared_shared_dependency_placement_refuses_instead_of_duplicate_injection() {
    let tmp = tempfile::tempdir().unwrap();
    let (roots, _, _) = fixture(tmp.path());
    fs::write(roots[0].join("vision/core.md"), "COMMON_UNPLACED").unwrap();
    fs::write(
        roots[0].join("vision/system.md"),
        "---\ndependencies: [ vision-core ]\n---\nSYSTEM\n",
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
        system_modules: vec!["vision-system".into()],
        user_modules: vec!["compensation-user00".into()],
    };
    assert!(startup::compose(&registry, &selection).is_err());
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
fn spirit_intent_cannot_be_user_even_without_frontmatter_placement() {
    let tmp = tempfile::tempdir().unwrap();
    let (_, registry, selection) = fixture(tmp.path());
    for id in ["spirit-core", "intent-seat"] {
        let mut s = selection.clone();
        s.system_modules.retain(|m| m != id);
        s.user_modules.push(id.into());
        assert!(startup::compose(&registry, &s).is_err());
    }
    let mut s = selection;
    s.system_modules.clear();
    assert!(startup::compose(&registry, &s).is_err());
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
    let result = startup::compose(&registry, &selection).unwrap();
    let module = result
        .context_provenance_vector
        .iter()
        .find(|m| m.module_identity == "compensation-user00")
        .unwrap();
    assert_eq!(module.body_sha256, digest(text));
    assert_eq!(module.rendered_sha256, digest("CLAUDE_ONLY\n"));
    assert!(result.user_text.contains("CLAUDE_ONLY"));
    assert!(!result.user_text.contains("CODEX_OTHER"));
    assert!(!result.user_text.contains("{%"));
}
