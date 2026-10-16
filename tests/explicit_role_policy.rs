use curriculum::{
    generated::*,
    projection::{ConstructsSkillProjector, ProjectsSkills, SkillProjector},
    registry::ResolvedSkill,
    role_policy::{dispatch_policy, DISPATCH_POLICY_PATH},
    roles::{PlansRoles, RendersRolePacketPlan},
};

fn fixture() -> Roles {
    Roles {
        role_module_vector: vec![
            RoleModule { first_string: "universal".into(), second_string: "UNIVERSAL_MARKER".into() },
            RoleModule { first_string: "illustrator-faithfulness".into(), second_string: "FAITHFULNESS_MARKER".into() },
            RoleModule { first_string: "illustrator-svg".into(), second_string: "SVG_MARKER".into() },
        ],
        model_vector: vec![Model { string: "sonnet".into(), provider: Provider::Claude, effort_vector: vec![Effort::Medium] }],
        role_permission_vector: vec![RolePermission { first_string: "write".into(), second_string: String::new(), permission: Permission::Unrestricted }],
        role_depth_vector: vec![RoleDepth {
            string: "ordinary".into(),
            first_model_choice: ModelChoice { string: "sonnet".into(), effort_option: Some(Effort::Medium) },
            second_model_choice: ModelChoice { string: "unused".into(), effort_option: Some(Effort::Medium) },
        }],
        role_description_vector: vec![],
        role_alias_vector: vec![RoleAlias {
            first_string: "illustrator".into(), second_string: "write".into(), third_string: "ordinary".into(),
            fourth_string: "Render one supplied diagram".into(), surface_vector: vec![Surface::ClaudeAgent],
        }],
        first_string_vector: vec!["universal".into()],
        target_insertion_vector: vec![],
        second_string_vector: vec![],
        role_policy_vector: vec![RolePolicy {
            string: "illustrator".into(),
            role_context: RoleContext::Explicit(vec!["illustrator-faithfulness".into(), "illustrator-svg".into()]),
            role_tools: RoleTools::Explicit(vec![]),
            role_callers: RoleCallers::Only(vec![CallerMetaflow {
                first_string: "Field".into(), second_string: "Publisher".into(), third_string: "Quaternary".into(),
            }]),
        }],
    }
}

#[test]
fn explicit_role_ignores_universal_standing_and_workspace_procedure() {
    let workspace = tempfile::tempdir().unwrap();
    std::fs::create_dir(workspace.path().join("subagents")).unwrap();
    std::fs::write(workspace.path().join("subagents/illustrator.md"), "WORKSPACE_NOISE").unwrap();
    let plan = fixture().role_plan(workspace.path()).unwrap();
    let packet = &plan.role_packet_plan_vector[0];
    assert_eq!(packet.fourth_string, "sonnet");
    assert!(matches!(packet.effort, Effort::Medium));
    let rendered = packet.render(&["STANDING_NOISE".into()]);
    assert!(rendered.contains("tools: []\n"));
    for marker in ["FAITHFULNESS_MARKER", "SVG_MARKER"] {
        assert_eq!(rendered.matches(marker).count(), 1);
    }
    for marker in ["UNIVERSAL_MARKER", "WORKSPACE_NOISE", "STANDING_NOISE"] {
        assert!(!rendered.contains(marker));
    }
    let skills = vec![ResolvedSkill {
        path: "fixture/spirit-core.md".into(), name: "spirit-core".into(),
        body: "---\ndependencies: []\n---\nSPIRIT_NOISE".into(), dependencies: vec![],
    }];
    let projector = SkillProjector::at(workspace.path().into());
    projector.write(&skills, &plan).unwrap();
    projector.check(&skills, &plan).unwrap();
    let actual = std::fs::read_to_string(workspace.path().join(&packet.first_string)).unwrap();
    assert_eq!(actual, rendered);
    let policy = std::fs::read_to_string(workspace.path().join(DISPATCH_POLICY_PATH)).unwrap();
    assert_eq!(policy, dispatch_policy(&plan.role_packet_plan_vector).unwrap());
    assert!(policy.contains("\"schemaVersion\":1"));
    assert!(policy.contains("\"metaflows\":[{\"aspect\":\"Field\",\"topic\":\"Publisher\",\"layer\":\"Quaternary\"}]"));
    assert!(policy.contains("\"allowModelOverride\":false"));
    assert!(policy.contains("\"allowNestedCaller\":false"));
    std::fs::write(workspace.path().join(DISPATCH_POLICY_PATH), "{}").unwrap();
    assert!(projector.check(&skills, &plan).is_err());
    projector.write(&skills, &plan).unwrap();
    projector.check(&skills, &plan).unwrap();
}

#[test]
fn absent_policy_preserves_ordinary_inherited_rendering() {
    let workspace = tempfile::tempdir().unwrap();
    let mut roles = fixture();
    roles.role_policy_vector.clear();
    let baseline = roles.role_plan(workspace.path()).unwrap();
    let rendered = baseline.role_packet_plan_vector[0].render(&["STANDING".into()]);
    assert!(rendered.contains("UNIVERSAL_MARKER"));
    assert!(rendered.contains("STANDING"));
    assert!(rendered.contains("tools: Read, Grep, Glob, Bash, Edit, Write"));
    assert_eq!(dispatch_policy(&baseline.role_packet_plan_vector).unwrap(), "{\"schemaVersion\":1,\"roles\":[]}\n");
    roles.role_policy_vector.push(RolePolicy { string: "illustrator".into(), role_context: RoleContext::Inherit, role_tools: RoleTools::Inherit, role_callers: RoleCallers::Any });
    let explicit_inherit = roles.role_plan(workspace.path()).unwrap();
    assert_eq!(rendered, explicit_inherit.role_packet_plan_vector[0].render(&["STANDING".into()]));
}

#[test]
fn invalid_policy_data_refuses_before_projection() {
    let workspace = tempfile::tempdir().unwrap();
    let invalid: Vec<Box<dyn Fn(&mut Roles)>> = vec![
        Box::new(|r| r.role_policy_vector.push(r.role_policy_vector[0].clone())),
        Box::new(|r| r.role_policy_vector[0].string = "unknown".into()),
        Box::new(|r| r.role_policy_vector[0].string = "../escape".into()),
        Box::new(|r| r.role_policy_vector[0].role_context = RoleContext::Explicit(vec!["missing".into()])),
        Box::new(|r| r.role_policy_vector[0].role_context = RoleContext::Explicit(vec![])),
        Box::new(|r| r.role_policy_vector[0].role_context = RoleContext::Explicit(vec!["illustrator-svg".into(), "illustrator-svg".into()])),
        Box::new(|r| r.role_module_vector.push(r.role_module_vector[1].clone())),
        Box::new(|r| r.role_module_vector[1].second_string.clear()),
        Box::new(|r| r.role_policy_vector[0].role_tools = RoleTools::Explicit(vec!["UnknownTool".into()])),
        Box::new(|r| r.role_policy_vector[0].role_callers = RoleCallers::Only(vec![])),
        Box::new(|r| r.role_policy_vector[0].role_callers = RoleCallers::Only(vec![CallerMetaflow { first_string: "Field Publisher".into(), second_string: "Publisher".into(), third_string: "Quaternary".into() }])),
        Box::new(|r| r.role_alias_vector.push(r.role_alias_vector[0].clone())),
        Box::new(|r| r.role_alias_vector[0].surface_vector.push(Surface::CodexAgent)),
    ];
    for (index, mutate) in invalid.into_iter().enumerate() {
        let mut roles = fixture();
        mutate(&mut roles);
        assert!(roles.role_plan(workspace.path()).is_err(), "invalid case {index}");
    }
}

#[test]
fn exported_packet_validation_rejects_duplicate_or_retargeted_policy() {
    let plan = fixture().role_plan(std::path::Path::new(".")).unwrap();
    let packet = plan.role_packet_plan_vector[0].clone();
    assert!(dispatch_policy(&[packet.clone(), packet.clone()]).is_err());
    let mut wrong = packet;
    wrong.first_string = ".claude/agents/different.md".into();
    assert!(dispatch_policy(&[wrong]).is_err());
}

#[test]
fn policy_survives_typed_transport_without_losing_callers_or_empty_tools() {
    let plan = fixture().role_plan(std::path::Path::new(".")).unwrap();
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&plan).unwrap();
    let decoded = rkyv::from_bytes::<RolePlan, rkyv::rancor::Error>(&bytes).unwrap();
    assert_eq!(decoded, plan);
    assert_eq!(dispatch_policy(&decoded.role_packet_plan_vector).unwrap(), dispatch_policy(&plan.role_packet_plan_vector).unwrap());
}

#[test]
fn duplicate_and_invalid_caller_tuples_refuse() {
    for layer in ["Unknown", "quaternary", "Quaternary\n"] {
        let mut roles = fixture();
        let RoleCallers::Only(callers) = &mut roles.role_policy_vector[0].role_callers else { unreachable!() };
        callers[0].third_string = layer.into();
        assert!(roles.role_plan(std::path::Path::new(".")).is_err());
    }
    let mut roles = fixture();
    let RoleCallers::Only(callers) = &mut roles.role_policy_vector[0].role_callers else { unreachable!() };
    callers.push(callers[0].clone());
    assert!(roles.role_plan(std::path::Path::new(".")).is_err());
}
