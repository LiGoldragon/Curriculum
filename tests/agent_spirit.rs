use std::{fs, path::PathBuf};
use curriculum::{generated::{Effort, Permission, RolePacketPlan, RolePlan, Surface}, projection::{ConstructsSkillProjector, ProjectsSkills, SkillProjector}, registry::ResolvedSkill};

#[test]
fn every_agent_including_slim_book_carries_every_spirit_body() {
    let workspace = tempfile::tempdir().unwrap();
    let skills = [("spirit-core", "CORE_SPIRIT_EXACT"), ("spirit-future", "FUTURE_SPIRIT_EXACT"), ("compensation-example", "OTHER_STANDING_BODY")].into_iter().map(|(name, body)| ResolvedSkill {
        path: PathBuf::from(format!("source/{name}.md")), name: name.into(), body: format!("---\ndependencies: []\n---\n{body}\n"), dependencies: vec![],
    }).collect::<Vec<_>>();
    let mut packets = Vec::new();
    for name in ["read-ordinary", "write-ordinary", "tester", "book", "book-page"] {
        for (surface, path) in [(Surface::ClaudeAgent, format!(".claude/agents/{name}.md")), (Surface::CodexAgent, format!(".codex/agents/{name}.toml")), (Surface::PiAgent, format!(".pi/agents/{name}.md"))] {
            packets.push(RolePacketPlan {role_policy_option: None, surface, first_string: path, second_string: name.into(), third_string: "fixture".into(), fourth_string: "fixture-model".into(), effort: Effort::Medium, permission: Permission::Unrestricted, fifth_string: "ROLE_BASE".into(), sixth_string: "OWNED_APPENDIX".into()});
        }
    }
    let plan = RolePlan { startup_bundle_vector: Vec::new(),role_packet_plan_vector: packets, first_string_vector: vec!["compensation-example".into()], second_string_vector: vec![], string: String::new()};
    let projector = SkillProjector::at(workspace.path().into());
    projector.write(&skills, &plan).unwrap();
    projector.check(&skills, &plan).unwrap();
    for packet in &plan.role_packet_plan_vector {
        let body = fs::read_to_string(workspace.path().join(&packet.first_string)).unwrap();
        for marker in ["CORE_SPIRIT_EXACT", "FUTURE_SPIRIT_EXACT", "OWNED_APPENDIX"] {assert_eq!(body.matches(marker).count(), 1, "{}: {marker}", packet.first_string);}
        if packet.second_string == "book" && matches!(packet.surface, Surface::ClaudeAgent) {assert!(!body.contains("ROLE_BASE")); assert!(!body.contains("OTHER_STANDING_BODY"));}
        else if packet.second_string != "book" {assert!(body.contains("OTHER_STANDING_BODY"));}
    }
}
