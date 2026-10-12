use datom_codec::{Actualizing, Budget, Potential};
use protos::ReaderBudget;
use curriculum::{generated::{Roles, RolesDocument}, roles::PlansRoles, projection::{ConstructsSkillProjector, ProjectsSkills, SkillProjector}};
fn roles(source: &str) -> Roles {
 let mut potential = Potential::<RolesDocument>::from(source);
 let RolesDocument::Roles(roles) = potential.actualize(&mut Budget { remaining: 65536, reader: ReaderBudget { remaining: 65536 }, depth: 0, maximum_depth: 65536 }).unwrap();
 roles
}
#[test]
fn main_role_is_projected_alone_and_never_inserted_in_other_agents() {
 let mut configuration = roles(include_str!("../roles.datom"));
 configuration.second_string_vector.clear();
 let workspace = tempfile::tempdir().unwrap();
 let plan = configuration.role_plan(workspace.path()).unwrap();
 let target = "tools/main-seat-roles/psyche-core-tertiary.md";
 let packet = plan.role_packet_plan_vector.iter().find(|p| p.first_string == target).unwrap();
 assert!(packet.fifth_string.contains("read every comment"));
 assert!(packet.fifth_string.contains("full anchored passage"));
 assert!(packet.fifth_string.contains("comment-relay-ledger.jsonl"));
 assert!(!packet.fifth_string.contains("Do not create a lane"));
 for other in plan.role_packet_plan_vector.iter().filter(|p| p.first_string != target) { assert!(!other.fifth_string.contains("comment-relay-ledger.jsonl")); }
 let projector = SkillProjector::at(workspace.path().into());
 let skills = [curriculum::registry::ResolvedSkill { path: std::path::PathBuf::from("fixture/unrelated.md"), name: "unrelated".into(), body: "---\ndependencies: []\n---\nUNRELATED_BODY\n".into(), dependencies: vec![] }];
 projector.write(&skills, &plan).unwrap(); projector.check(&skills, &plan).unwrap();
 let body = std::fs::read_to_string(workspace.path().join(target)).unwrap();
 assert!(body.contains("record each successful delivery with its receipt"));
 assert!(!body.contains("The brief is your authority"));
 std::fs::write(workspace.path().join(target), "wrong role").unwrap();
 assert!(projector.check(&skills, &plan).is_err());
}
#[test]
fn duplicate_and_escaping_role_identifiers_refuse() {
 let mut configuration = roles(include_str!("../roles.datom"));
 let module = configuration.role_module_vector.iter().find(|m| m.first_string == "main-system-psyche-core-tertiary").unwrap().clone();
 configuration.role_module_vector.push(module);
 assert!(configuration.role_plan(std::path::Path::new(".")).is_err());
 let mut configuration = roles(include_str!("../roles.datom"));
 configuration.role_module_vector.iter_mut().find(|m| m.first_string == "main-system-psyche-core-tertiary").unwrap().first_string = "main-system-../escape".into();
 assert!(configuration.role_plan(std::path::Path::new(".")).is_err());
}
