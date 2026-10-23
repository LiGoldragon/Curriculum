use curriculum::{generated::*, registry::SkillRegistry, roles};
use std::fs;
fn fixture(base: &std::path::Path) -> (SkillRegistry, Roles, RoleSelection) {
    let roots = [base.join("psyche"), base.join("mind"), base.join("field")];
    for root in &roots {
        fs::create_dir_all(root).unwrap();
    }
    fs::create_dir(roots[2].join("role")).unwrap();
    fs::create_dir(roots[2].join("compensation")).unwrap();
    fs::write(
        roots[2].join("role/illustrator-description.md"),
        "Draw a figure of the same idea.",
    )
    .unwrap();
    fs::write(roots[2].join("role/illustrator-svg.md"), "SAFE_SVG").unwrap();
    fs::write(roots[2].join("role/unselected.md"), "UNSELECTED_NOISE").unwrap();
    fs::write(
        roots[2].join("compensation/illustration.md"),
        "---\ndependencies: [ role:unselected ]\nplacement: system\n---\nAUTHORED_RULES\n",
    )
    .unwrap();
    let registry = SkillRegistry::from_context_repositories(
        &roots,
        &["a".repeat(40), "b".repeat(40), "c".repeat(40)],
    )
    .unwrap();
    let roles = Roles {
        role_definition_vector: vec![RoleDefinition {
            role_name: "illustrator".into(),
            surface: Surface::ClaudeAgent,
            model_id: "sonnet".into(),
            effort: Effort::Medium,
            permission: Permission::Unrestricted,
            description_module: "role:illustrator-description".into(),
            context_modules: vec![
                "role:illustrator-svg".into(),
                "compensation:illustration".into(),
            ],
            role_tools: RoleTools::Explicit(vec![]),
            role_callers: RoleCallers::Only(vec![CallerMetaflow {
                first_string: "Field".into(),
                second_string: "Publisher".into(),
                third_string: "Quaternary".into(),
            }]),
        }],
    };
    (
        registry,
        roles,
        RoleSelection {
            role_name: "illustrator".into(),
            surface: Surface::ClaudeAgent,
        },
    )
}
#[test]
fn role_uses_only_selected_authored_ids_and_preserves_policy_through_wire() {
    let tmp = tempfile::tempdir().unwrap();
    let (registry, data, selection) = fixture(tmp.path());
    let result = roles::compose(&registry, &data, &selection).unwrap();
    assert_eq!(result.model_id, "sonnet");
    assert!(matches!(result.effort, Effort::Medium));
    assert!(matches!(&result.role_tools,RoleTools::Explicit(v) if v.is_empty()));
    assert!(
        matches!(&result.role_callers,RoleCallers::Only(v) if v.len()==1 && v[0].second_string=="Publisher")
    );
    assert_eq!(result.context_text.matches("AUTHORED_RULES").count(), 1);
    assert!(!result.context_text.contains("UNSELECTED_NOISE"));
    assert_eq!(result.context_provenance_vector.len(), 3);
    assert_eq!(
        result.context_provenance_vector[2].source_relative_path,
        "compensation/illustration.md"
    );
    let outcome = Outcome::RoleComposed(result);
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&outcome).unwrap();
    assert_eq!(
        rkyv::from_bytes::<Outcome, rkyv::rancor::Error>(&bytes).unwrap(),
        outcome
    );
    assert!(!tmp.path().join(".claude").exists());
}
#[test]
fn inline_paths_missing_ambiguous_roles_and_invalid_caller_tool_data_refuse() {
    let tmp = tempfile::tempdir().unwrap();
    let (registry, data, selection) = fixture(tmp.path());
    for value in [
        "Inline prose is forbidden",
        "/tmp/source.md",
        "role:missing",
    ] {
        let mut d = data.clone();
        d.role_definition_vector[0].context_modules = vec![value.into()];
        assert!(roles::compose(&registry, &d, &selection).is_err());
    }
    let mut d = data.clone();
    d.role_definition_vector
        .push(d.role_definition_vector[0].clone());
    assert!(roles::validate(&d).is_err());
    let mut d = data.clone();
    d.role_definition_vector[0].role_tools = RoleTools::Explicit(vec!["InventedTool".into()]);
    assert!(roles::validate(&d).is_err());
    let mut d = data.clone();
    d.role_definition_vector[0].role_callers = RoleCallers::Only(vec![]);
    assert!(roles::validate(&d).is_err());
    let mut d = data.clone();
    let duplicate = d.role_definition_vector[0].context_modules[0].clone();
    d.role_definition_vector[0].context_modules.push(duplicate);
    assert!(roles::validate(&d).is_err());
}
