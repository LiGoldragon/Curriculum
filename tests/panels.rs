use curriculum::{
    ExpandsSkills, SkillRegistry,
    generated::{ContextModules, DescriptionModule, RoleDefinition, Roles},
    panels::{PanelPrompt, PanelRendering},
    registry::{IdentityText, ModuleIdentityConversion, workspace::WorkspaceProjector},
    roles, startup,
};
use signal_curriculum_nexus::*;
use std::{fs, path::Path};

fn identity(name: &str) -> ModuleIdentity {
    name.module_identity().unwrap()
}
fn fixture(base: &Path) -> SkillRegistry {
    let roots = [base.join("psyche"), base.join("mind"), base.join("field")];
    for root in &roots {
        fs::create_dir_all(root).unwrap();
        fs::write(root.join("modules.datom"), "[]").unwrap();
    }
    let declarations = [
        ("Spirit", "core", "{% include \"panel:shared:0\" %}\nSPIRIT"),
        ("Intent", "seat", "INTENT"),
        (
            "Vision",
            "first",
            "FIRST\n{% include \"panel:shared:0\" %}\n{% include \"panel:missing:0\" %}\nEND",
        ),
        (
            "Vision",
            "second",
            "SECOND\n{% include \"panel:shared:0\" %}",
        ),
        (
            "Panel",
            "shared",
            "PANEL_BODY\n{% include \"panel:nested:0\" %}",
        ),
        (
            "Panel",
            "nested",
            "NESTED_BODY\n{\"message\":{\"usage\":{\"input_tokens\":2}}}\n",
        ),
    ];
    let mut manifest = String::from("[");
    for (kind, topic, body) in declarations {
        let file = format!("{}/{topic}/0.md", kind.to_lowercase());
        fs::create_dir_all(roots[1].join(&file).parent().unwrap()).unwrap();
        let text = if kind == "Panel" {
            body.to_owned()
        } else {
            format!("---\ndescription: A master.\n---\n{body}")
        };
        fs::write(roots[1].join(&file), text).unwrap();
        manifest.push_str(&format!(" {{ {{ {kind} {topic} Single.0 }} «{file}» }}"));
    }
    manifest.push_str(" ]");
    fs::write(roots[1].join("modules.datom"), manifest).unwrap();
    SkillRegistry::from_context_repositories(
        &roots,
        &["a".repeat(40), "b".repeat(40), "c".repeat(40)],
    )
    .unwrap()
}

#[test]
fn panel_is_declared_and_lookupable_without_becoming_a_skill_or_description() {
    let temp = tempfile::tempdir().unwrap();
    let registry = fixture(temp.path());
    assert_eq!(
        registry
            .resolved_exact_sources(&["panel:shared:0".into()])
            .unwrap()[0]
            .module_identity
            .module_kind,
        ModuleKind::Panel
    );
    assert_eq!(registry.skill_names().unwrap().len(), 4);
    assert_eq!(registry.actual_skills().len(), 4);
    assert_eq!(registry.listed_modules_verbose().unwrap().len(), 4);
}

#[test]
fn startup_merges_at_the_first_mark_once_in_each_prompt_and_reports_missing() {
    let temp = tempfile::tempdir().unwrap();
    let registry = fixture(temp.path());
    fs::create_dir(temp.path().join("records")).unwrap();
    let mut selection = StartupSelection {
        target: Target::Codex,
        caller_metaflow: CallerMetaflow {
            first_string: "Mind".into(),
            second_string: "curriculum".into(),
            third_string: "Sol".into(),
        },
        system_modules: SystemModules(vec![identity("spirit:core:0"), identity("intent:seat:0")]),
        user_modules: UserModules(vec![
            identity("vision:first:0"),
            identity("vision:second:0"),
        ]),
        flow_id_vector: vec![],
    };
    let composed = startup::compose(&registry, &temp.path().join("records"), &selection).unwrap();
    assert_eq!(composed.system_text.0.matches("PANEL_BODY").count(), 1);
    assert_eq!(composed.user_text.0.matches("PANEL_BODY").count(), 1);
    assert_eq!(composed.user_text.0.matches("NESTED_BODY").count(), 1);
    assert!(
        composed.user_text.0.find("FIRST").unwrap()
            < composed.user_text.0.find("PANEL_BODY").unwrap()
    );
    assert!(
        composed.user_text.0.find("PANEL_BODY").unwrap()
            < composed.user_text.0.find("SECOND").unwrap()
    );
    assert!(!composed.user_text.0.contains("{% include"));
    assert!(!composed.user_text.0.contains("<!-- module: panel:"));
    assert_eq!(composed.missing_panels.0, [identity("panel:missing:0")]);
    selection.user_modules.0 = vec![identity("panel:shared:0")];
    assert!(startup::compose(&registry, &temp.path().join("records"), &selection).is_err());
}

#[test]
fn roles_and_each_skill_have_their_own_complete_panel_scope() {
    let temp = tempfile::tempdir().unwrap();
    let registry = fixture(temp.path());
    let roles = Roles {
        role_definition_vector: vec![RoleDefinition {
            role_name: RoleName("witness".into()),
            surface: Surface::CodexAgent,
            model_id: ModelId("test".into()),
            effort: Effort::Medium,
            permission: Permission::Unrestricted,
            description_module: DescriptionModule(identity("spirit:core:0")),
            context_modules: ContextModules(vec![
                identity("vision:first:0"),
                identity("vision:second:0"),
            ]),
            role_tools: RoleTools::Inherit,
            role_callers: RoleCallers::Any,
        }],
    };
    let composed = roles::compose(
        &registry,
        &roles,
        &RoleSelection {
            role_name: RoleName("witness".into()),
            surface: Surface::CodexAgent,
        },
    )
    .unwrap();
    assert_eq!(composed.description_text.0.matches("PANEL_BODY").count(), 1);
    assert_eq!(composed.context_text.0.matches("PANEL_BODY").count(), 1);
    assert_eq!(composed.missing_panels.0, [identity("panel:missing:0")]);
    let workspace = temp.path().join("workspace");
    let projected = WorkspaceProjector::new(&workspace)
        .project(&registry)
        .unwrap();
    assert_eq!(projected.missing_panels.0, [identity("panel:missing:0")]);
    for directory in [".claude", ".agents", ".codex", ".pi", ".opencode"] {
        for master in ["vision:first:0", "vision:second:0"] {
            let body = fs::read_to_string(
                workspace
                    .join(directory)
                    .join("skills")
                    .join(master)
                    .join("SKILL.md"),
            )
            .unwrap();
            assert_eq!(body.matches("PANEL_BODY").count(), 1);
            assert_eq!(body.matches("NESTED_BODY").count(), 1);
            assert!(!body.contains("{% include"));
        }
        assert!(
            !workspace
                .join(directory)
                .join("skills/panel:shared:0")
                .exists()
        );
    }
}

#[test]
fn conditionals_raw_and_literal_fences_keep_their_meaning() {
    let temp = tempfile::tempdir().unwrap();
    let registry = fixture(temp.path());
    let prompt = PanelPrompt::for_registry(&registry);
    let body = "{% if claude %}\n{% include \"panel:shared:0\" %}\n{% endif %}\n{% raw %}\n{{literal}}\n{% endraw %}\n{% include \"panel:shared:0\" %}";
    let rendered = prompt.render(&Target::Codex, body).unwrap();
    assert!(rendered.starts_with("{{literal}}\nPANEL_BODY"));
    assert_eq!(rendered.matches("PANEL_BODY").count(), 1);
    assert!(rendered.contains(r#"{"message":{"usage":{"input_tokens":2}}}"#));
    assert!(prompt.render(&Target::Codex, "{{unknown}}").is_err());
    assert!(
        prompt
            .render(&Target::Codex, "{% include \"vision:first:0\" %}")
            .is_err()
    );
    assert!(
        prompt
            .render(&Target::Codex, "{% include \"/tmp/arbitrary\" %}")
            .is_err()
    );
    assert_eq!(prompt.missing().unwrap().0.len(), 0);
}

#[test]
fn panel_changes_update_the_merged_hash_without_changing_the_master_source_hash() {
    let temp = tempfile::tempdir().unwrap();
    let registry = fixture(temp.path());
    let source = registry
        .resolved_exact_sources(&["vision:second:0".into()])
        .unwrap()
        .remove(0);
    let before = PanelPrompt::for_registry(&registry)
        .render(&Target::Codex, &source.string)
        .unwrap();
    fs::write(temp.path().join("mind/panel/shared/0.md"), "UPDATED_PANEL").unwrap();
    assert!(
        PanelPrompt::for_registry(&registry)
            .render(&Target::Codex, &source.string)
            .is_err()
    );
    let roots = [
        temp.path().join("psyche"),
        temp.path().join("mind"),
        temp.path().join("field"),
    ];
    let updated = SkillRegistry::from_context_repositories(
        &roots,
        &["a".repeat(40), "d".repeat(40), "c".repeat(40)],
    )
    .unwrap();
    let current = updated
        .resolved_exact_sources(&["vision:second:0".into()])
        .unwrap()
        .remove(0);
    assert_eq!(current.source_sha256, source.source_sha256);
    assert_eq!(
        current.module_identity.identity_text().unwrap(),
        "vision:second:0"
    );
    let after = PanelPrompt::for_registry(&updated)
        .render(&Target::Codex, &current.string)
        .unwrap();
    assert_ne!(before, after);
    assert!(after.contains("UPDATED_PANEL"));
}

#[test]
fn actual_authored_harness_fences_pass_the_include_engine_unchanged() {
    let temp = tempfile::tempdir().unwrap();
    let registry = fixture(temp.path());
    let source = include_str!("fixtures/notions-404d40/mind-belief/notion/harness-nexus/0.md");
    assert_eq!(
        PanelPrompt::for_registry(&registry)
            .render(&Target::Codex, source)
            .unwrap(),
        source
    );
}
