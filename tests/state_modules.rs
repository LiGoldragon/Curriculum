use curriculum::{
    registry::{ModuleIdentityConversion, SkillRegistry},
    startup,
};
use signal_curriculum_nexus::{
    CallerMetaflow, ModuleKind, StartupSelection, SystemModules, Target, UserModules,
};
use std::{fs, path::Path};

const DESCRIPTION: &str = "Which of his hosts are away, since when and until when.";
const STATE_BODY: &str = "STATE_BODY_AVAILABLE_ON_REQUEST";

fn write(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

#[test]
fn field_state_is_listed_and_resolved_without_entering_startup() {
    let temporary = tempfile::tempdir().unwrap();
    let roots = ["psyche", "mind", "field"].map(|name| temporary.path().join(name));
    write(
        &roots[0],
        "modules.datom",
        "[ { { Spirit core Single.0 } spirit/core/0.md } ]",
    );
    write(&roots[0], "spirit/core/0.md", "SYSTEM_SPIRIT");
    write(
        &roots[1],
        "modules.datom",
        "[ { { Intent main-flow Single.0 } intent/main-flow/0.md } ]",
    );
    write(&roots[1], "intent/main-flow/0.md", "SYSTEM_INTENT");
    write(
        &roots[2],
        "modules.datom",
        "[ { { State hosts Single.0 } state/hosts/0.md } ]",
    );
    write(
        &roots[2],
        "state/hosts/0.md",
        &format!("---\ndescription: {DESCRIPTION}\ndependencies: []\n---\n{STATE_BODY}\n"),
    );
    let registry = SkillRegistry::from_context_repositories(
        &roots,
        &["a".repeat(40), "b".repeat(40), "c".repeat(40)],
    )
    .unwrap();
    let name = "state:hosts:0";
    let identity = name.module_identity().unwrap();
    assert_eq!(identity.module_kind, ModuleKind::State);
    let listed = registry.listed_modules_verbose().unwrap();
    let listed_state = listed
        .iter()
        .find(|row| row.module.module_identity == identity)
        .unwrap();
    assert_eq!(listed_state.description_text.0, DESCRIPTION);
    let resolved = registry.resolved_sources(&[name.into()]).unwrap();
    let [found] = resolved.as_slice() else {
        panic!("exactly one State module must resolve");
    };
    assert_eq!(found.repository_name.0, "field-belief");
    assert_eq!(found.source_relative_path.0, "state/hosts/0.md");
    assert!(found.string.contains(STATE_BODY));

    let mut selection = StartupSelection {
        target: Target::Codex,
        caller_metaflow: CallerMetaflow {
            first_string: "Field".into(),
            second_string: "core".into(),
            third_string: "Sol".into(),
        },
        system_modules: SystemModules(vec![
            "spirit:core:0".module_identity().unwrap(),
            "intent:main-flow:0".module_identity().unwrap(),
        ]),
        user_modules: UserModules(vec![]),
        flow_id_vector: vec![],
    };
    let composed = startup::compose(&registry, temporary.path(), &selection).unwrap();
    assert!(!composed.system_text.0.contains(name));
    assert!(!composed.system_text.0.contains(STATE_BODY));
    assert!(!composed.user_text.0.contains(STATE_BODY));
    assert!(
        composed
            .context_provenance_vector
            .iter()
            .all(|row| row.module_identity != identity)
    );

    selection.system_modules.0.push(identity);
    assert_eq!(
        startup::compose(&registry, temporary.path(), &selection).unwrap_err(),
        "module has the wrong startup placement: state:hosts:0"
    );
}
