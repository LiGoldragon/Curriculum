use curriculum::{
    generated::*,
    registry::{IdentityText, ModuleIdentityConversion, SkillRegistry},
    roles, startup,
};
use sha2::{Digest, Sha256};
use signal_curriculum_nexus::*;
use std::{fs, path::Path};

fn identity(text: &str) -> ModuleIdentity {
    text.module_identity().unwrap()
}
fn write(root: &Path, file: &str, body: &str) {
    let path = root.join(file);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

#[test]
fn notions_resolve_with_spans_and_compose_as_knowledge_with_actual_provenance() {
    let temp = tempfile::tempdir().unwrap();
    let roots = [
        temp.path().join("psyche"),
        temp.path().join("mind"),
        temp.path().join("field"),
    ];
    for root in &roots {
        fs::create_dir_all(root).unwrap();
        fs::write(root.join("modules.datom"), "[]").unwrap();
    }
    let sources = [
        ("Spirit", "core", "Single.0", "spirit/core/0.md", "SPIRIT"),
        (
            "Intent",
            "main-flow",
            "Single.0",
            "intent/main-flow/0.md",
            "INTENT",
        ),
        (
            "Vision",
            "flow-nexus",
            "Single.0",
            "vision/flow-nexus/0.md",
            "SINGLETON",
        ),
        (
            "Vision",
            "flow-nexus",
            "Span.{ 0 1 }",
            "vision/flow-nexus/0-1.md",
            "SPAN",
        ),
        (
            "Notion",
            "source-nexus",
            "Single.0",
            "notion/source-nexus/0.md",
            "SOURCE_NOTION",
        ),
        (
            "Notion",
            "harness-nexus",
            "Single.0",
            "notion/harness-nexus/0.md",
            "---\ndependencies: [ notion:source-nexus:0 ]\n---\nHARNESS_NOTION",
        ),
    ];
    let mut declarations = String::from("[");
    for (kind, topic, versions, path, body) in sources {
        write(&roots[1], path, body);
        declarations.push_str(&format!(" {{ {{ {kind} {topic} {versions} }} «{path}» }}"));
    }
    declarations.push_str(" ]");
    fs::write(roots[1].join("modules.datom"), declarations).unwrap();
    let registry = SkillRegistry::from_context_repositories(
        &roots,
        &["a".repeat(40), "b".repeat(40), "c".repeat(40)],
    )
    .unwrap();
    let numeric = registry
        .resolved_sources(&["vision:flow-nexus:0".into()])
        .unwrap();
    assert_eq!(numeric.len(), 2);
    let exact = registry
        .resolved_sources(&["vision:flow-nexus:0-1".into()])
        .unwrap();
    assert_eq!(exact.len(), 1);
    assert_eq!(
        exact[0].module_identity.identity_text().unwrap(),
        "vision:flow-nexus:0-1"
    );
    assert!(
        registry
            .resolved_sources(&["vision:flow-nexus:0-2".into()])
            .is_err()
    );
    let notion = registry
        .resolved_sources(&["notion:harness-nexus:0".into()])
        .unwrap();
    assert_eq!(notion.len(), 2);
    assert_eq!(notion[0].module_identity.module_kind, ModuleKind::Notion);
    assert_eq!(notion[0].source_relative_path.0, "notion/source-nexus/0.md");
    assert_eq!(
        notion[1].source_relative_path.0,
        "notion/harness-nexus/0.md"
    );
    fs::create_dir(temp.path().join("flows")).unwrap();
    let selection = StartupSelection {
        target: Target::Codex,
        caller_metaflow: CallerMetaflow {
            first_string: "Mind".into(),
            second_string: "flowNexus".into(),
            third_string: "Opus".into(),
        },
        system_modules: SystemModules(vec![
            identity("spirit:core:0"),
            identity("intent:main-flow:0"),
        ]),
        user_modules: UserModules(vec![
            identity("vision:flow-nexus:0"),
            identity("vision:flow-nexus:0-1"),
            identity("notion:harness-nexus:0"),
        ]),
        flow_id_vector: vec![],
    };
    let composed = startup::compose(&registry, &temp.path().join("flows"), &selection).unwrap();
    assert!(!composed.system_text.0.contains("NOTION"));
    for body in ["SINGLETON", "SPAN", "SOURCE_NOTION", "HARNESS_NOTION"] {
        assert_eq!(composed.user_text.0.matches(body).count(), 1);
    }
    for row in composed
        .context_provenance_vector
        .iter()
        .filter(|row| row.module_identity.module_kind == ModuleKind::Notion)
    {
        assert_eq!(row.startup_placement, StartupPlacement::User);
        assert_eq!(row.repository_name.0, "mind-belief");
        assert_eq!(row.source_revision.0, "b".repeat(40));
        assert_eq!(
            row.source_sha256.0,
            format!(
                "{:x}",
                Sha256::digest(fs::read(roots[1].join(&row.source_relative_path.0)).unwrap())
            )
        );
    }
    let data = Roles {
        role_definition_vector: vec![RoleDefinition {
            role_name: RoleName("witness".into()),
            surface: Surface::CodexAgent,
            model_id: ModelId("test".into()),
            effort: Effort::Medium,
            permission: Permission::Unrestricted,
            description_module: DescriptionModule(identity("vision:flow-nexus:0-1")),
            context_modules: ContextModules(vec![
                identity("notion:harness-nexus:0"),
                identity("notion:source-nexus:0"),
            ]),
            role_tools: RoleTools::Explicit(vec![]),
            role_callers: RoleCallers::Any,
        }],
    };
    let role = roles::compose(
        &registry,
        &data,
        &RoleSelection {
            role_name: RoleName("witness".into()),
            surface: Surface::CodexAgent,
        },
    )
    .unwrap();
    assert_eq!(role.context_text.0.matches("HARNESS_NOTION").count(), 1);
    assert!(role.context_text.0.contains("SOURCE_NOTION"));
}

#[test]
fn exact_authored_notions_and_flow_span_resolve_from_frozen_source_cohort() {
    let base =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/notions-404d40");
    let roots = ["psyche-belief", "mind-belief", "field-belief"].map(|name| base.join(name));
    let registry = SkillRegistry::from_context_repositories(
        &roots,
        &[
            "e9fcd7253eb4a26edaa6cccd621f769c8817787d".into(),
            "2fed6dfee98b88f0e00b463917f6c15f5e669560".into(),
            "c".repeat(40),
        ],
    )
    .unwrap();
    let oracle = include_str!("fixtures/notions-404d40/source-oracle.tsv");
    assert_eq!(
        registry.listed_modules_verbose().unwrap().len(),
        oracle.lines().count()
    );
    for line in oracle.lines() {
        let columns: Vec<_> = line.split('\t').collect();
        let actual = registry.resolved_sources(&[columns[2].into()]).unwrap();
        let found = actual
            .iter()
            .find(|item| item.module_identity.identity_text().unwrap() == columns[2])
            .unwrap();
        assert_eq!(found.repository_name.0, columns[0]);
        assert_eq!(found.source_revision.0, columns[1]);
        assert_eq!(found.source_relative_path.0, columns[3]);
        assert_eq!(found.source_sha256.0, columns[4]);
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(fs::read(base.join(columns[0]).join(columns[3])).unwrap())
            ),
            columns[4]
        );
    }
    for name in [
        "vision:flow-nexus:0-1",
        "vision:flow-nexus:0",
        "notion:harness-nexus:0",
        "notion:source-nexus:0",
    ] {
        assert!(
            registry
                .resolved_sources(&[name.into()])
                .unwrap()
                .iter()
                .any(|module| module.module_identity.identity_text().unwrap() == name)
        );
    }
}
