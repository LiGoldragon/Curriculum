mod support;
use curriculum::{generated::*, registry::SkillRegistry, roles};
use signal_curriculum_nexus::*;
use std::fs;
fn fixture(base: &std::path::Path) -> (SkillRegistry, Roles, RoleSelection) {
    let roots = [base.join("psyche"), base.join("mind"), base.join("field")];
    for root in &roots {
        fs::create_dir_all(root).unwrap();
    }
    fs::create_dir(roots[2].join("role")).unwrap();
    fs::create_dir(roots[2].join("compensation")).unwrap();
    write_source(
        roots[2].join("role/illustrator-description/0.md"),
        "Draw a figure of the same idea.",
    )
    .unwrap();
    write_source(roots[2].join("role/illustrator-svg/0.md"), "SAFE_SVG").unwrap();
    write_source(roots[2].join("role/unselected/0.md"), "UNSELECTED_NOISE").unwrap();
    write_source(
        roots[2].join("compensation/illustration/0.md"),
        "---\ndependencies: [ role:unselected:0 ]\nplacement: system\n---\nAUTHORED_RULES\n",
    )
    .unwrap();
    let registry =
        support::registry(&roots, &["a".repeat(40), "b".repeat(40), "c".repeat(40)]).unwrap();
    let roles = Roles {
        role_definition_vector: vec![RoleDefinition {
            role_name: RoleName("illustrator".into()),
            surface: Surface::ClaudeAgent,
            model_id: ModelId("sonnet".into()),
            effort: Effort::Medium,
            permission: Permission::Unrestricted,
            description_module: DescriptionModule(ModuleIdentity(
                "role:illustrator-description:0".into(),
            )),
            context_modules: ContextModules(vec![
                ModuleIdentity("role:illustrator-svg:0".into()),
                ModuleIdentity("compensation:illustration:0".into()),
            ]),
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
            role_name: RoleName("illustrator".into()),
            surface: Surface::ClaudeAgent,
        },
    )
}
#[test]
fn role_uses_only_selected_authored_ids_and_preserves_policy_through_wire() {
    let tmp = tempfile::tempdir().unwrap();
    let (registry, data, selection) = fixture(tmp.path());
    let result = roles::compose(&registry, &data, &selection).unwrap();
    assert_eq!(result.model_id.0, "sonnet");
    assert!(matches!(result.effort, Effort::Medium));
    assert!(matches!(&result.role_tools,RoleTools::Explicit(v) if v.is_empty()));
    assert!(
        matches!(&result.role_callers,RoleCallers::Only(v) if v.len()==1 && v[0].second_string=="Publisher")
    );
    assert_eq!(result.context_text.0.matches("AUTHORED_RULES").count(), 1);
    assert!(!result.context_text.0.contains("UNSELECTED_NOISE"));
    assert_eq!(result.context_provenance_vector.len(), 3);
    assert_eq!(
        result.context_provenance_vector[2].source_relative_path.0,
        "compensation/illustration/0.md"
    );
    let outcome = Output::RoleComposed(result);
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&outcome).unwrap();
    assert_eq!(
        rkyv::from_bytes::<Output, rkyv::rancor::Error>(&bytes).unwrap(),
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
        "role:missing:0",
    ] {
        let mut d = data.clone();
        d.role_definition_vector[0].context_modules =
            ContextModules(vec![ModuleIdentity(value.into())]);
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
    let duplicate = d.role_definition_vector[0].context_modules.0[0].clone();
    d.role_definition_vector[0]
        .context_modules
        .0
        .push(duplicate);
    assert!(roles::validate(&d).is_err());
}
#[test]
fn a_role_naming_a_module_the_belief_repositories_lack_is_refused_naming_the_module() {
    let tmp = tempfile::tempdir().unwrap();
    let (registry, data, selection) = fixture(tmp.path());
    let mut d = data.clone();
    d.role_definition_vector[0]
        .context_modules
        .0
        .push(ModuleIdentity("knowledge:book-making:0".into()));
    assert_eq!(
        roles::compose(&registry, &d, &selection).unwrap_err(),
        "missing module knowledge:book-making:0"
    );
    let mut d = data;
    d.role_definition_vector[0].description_module =
        DescriptionModule(ModuleIdentity("role:absent-description:0".into()));
    assert_eq!(
        roles::compose(&registry, &d, &selection).unwrap_err(),
        "missing module role:absent-description:0"
    );
}

fn write_source(
    path: impl AsRef<std::path::Path>,
    contents: impl AsRef<[u8]>,
) -> std::io::Result<()> {
    let path = path.as_ref();
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, contents)
}
