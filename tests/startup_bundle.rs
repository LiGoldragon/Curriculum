use std::{fs, path::Path};
use curriculum::{generated::*, registry::ResolvedSkill, startup, projection::{ConstructsSkillProjector, ProjectsSkills, SkillProjector}};
use sha2::{Digest, Sha256};

fn digest(text: &str) -> String { format!("{:x}", Sha256::digest(text.as_bytes())) }

fn fixture(root: &Path) -> (Vec<ResolvedSkill>, StartupBundle) {
    let mut skills = Vec::new();
    let mut sources = Vec::new();
    fs::create_dir_all(root.join("knowledge")).unwrap();
    for index in 0..23 {
        let (id, relative, text, placement) = if index == 0 {
            ("knowledge-system".to_owned(), "knowledge/system.md".to_owned(), "SYSTEM_ONLY\n".to_owned(), StartupPlacement::System)
        } else {
            (format!("knowledge-user{:02}", index - 1), format!("knowledge/user{:02}.md", index - 1), format!("BODY_{:02}_EXACT\n", index - 1), StartupPlacement::User)
        };
        let dependencies = if index == 0 { vec![] } else if index == 1 { vec!["knowledge-system".into()] } else { vec![format!("knowledge-user{:02}", index - 2)] };
        let body = format!("---\ndescription: fixture\ndependencies: [{}]\n---\n{text}", dependencies.join(", "));
        let path = root.join(&relative);
        fs::write(&path, &body).unwrap();
        sources.push(StartupSource { module_identity: id.clone(), repository_name: "mind".into(), source_revision: "a".repeat(40), source_root: root.to_string_lossy().into_owned(), source_relative_path: relative, source_sha256: digest(&body), body_sha256: digest(&text), startup_placement: placement });
        skills.push(ResolvedSkill { path, name: id, body, dependencies });
    }
    let bundle = StartupBundle {
        startup_bundle_name: "startup-field-publisher-quaternary".into(),
        caller_metaflow: CallerMetaflow { first_string: "Field".into(), second_string: "Publisher".into(), third_string: "Quaternary".into() },
        startup_cohort: "b".repeat(64), module_identity_vector: vec!["knowledge-user21".into()],
        startup_source_vector: sources, startup_mode: StartupMode::Initial,
    };
    (skills, bundle)
}

fn recovery(bundle: &mut StartupBundle) {
    let loaded = bundle.startup_source_vector.iter().skip(1).take(6).map(|s| StartupLoaded {
        module_identity: s.module_identity.clone(), repository_name: s.repository_name.clone(), source_revision: s.source_revision.clone(), source_relative_path: s.source_relative_path.clone(), source_sha256: s.source_sha256.clone(), body_sha256: s.body_sha256.clone(),
    }).collect();
    bundle.startup_mode = StartupMode::Recovery(RecoveryWitness {
        native_session: "same-native-session".into(), prompt_identity: "original-prompt".into(), startup_cohort: bundle.startup_cohort.clone(), startup_loaded_vector: loaded,
    });
}

#[test]
fn one_native_skill_contains_all_twenty_two_user_bodies_once() {
    let root = tempfile::tempdir().unwrap();
    let (skills, bundle) = fixture(root.path());
    let output = startup::outputs(&skills, &[bundle.clone()]).unwrap();
    assert_eq!(output.len(), 2);
    let body = &output[&startup::skill_path(&bundle)];
    for index in 0..22 { assert_eq!(body.matches(&format!("BODY_{index:02}_EXACT\n")).count(), 1); }
    assert!(!body.contains("SYSTEM_ONLY"));
    assert!(body.contains("disable-model-invocation: true\nuser-invocable: true"));
    let manifest = &output[&startup::manifest_path(&bundle)];
    assert!(manifest.contains("\"userCount\":22,\"emittedCount\":22"));
    assert_eq!(manifest.matches("\"sourceSha256\":").count(), 23);
    assert!(manifest.contains("\"placement\":\"system\",\"disposition\":\"already-composed-system\",\"ordinal\":null"));
}

#[test]
fn recovery_emits_remaining_sixteen_and_preserves_historical_status() {
    let root = tempfile::tempdir().unwrap();
    let (skills, mut bundle) = fixture(root.path()); recovery(&mut bundle);
    let output = startup::outputs(&skills, &[bundle.clone()]).unwrap();
    let body = &output[&startup::skill_path(&bundle)];
    for index in 0..22 { assert_eq!(body.matches(&format!("BODY_{index:02}_EXACT\n")).count(), usize::from(index >= 6)); }
    let manifest = &output[&startup::manifest_path(&bundle)];
    assert!(manifest.contains("\"userCount\":22,\"emittedCount\":16"));
    assert_eq!(manifest.matches("\"disposition\":\"previously-loaded\"").count(), 6);
    assert!(manifest.contains("supplied-witness-matched-not-native-reverified"));
}

#[test]
fn invalid_placement_provenance_and_recovery_never_produce_a_bundle() {
    let root = tempfile::tempdir().unwrap();
    let (skills, bundle) = fixture(root.path());
    let mut invalid = Vec::new();
    let mut b = bundle.clone(); b.startup_source_vector.pop(); invalid.push(b);
    let mut b = bundle.clone(); b.startup_source_vector.push(b.startup_source_vector[0].clone()); invalid.push(b);
    let mut b = bundle.clone(); b.startup_source_vector[22].startup_placement = StartupPlacement::System; invalid.push(b);
    let mut b = bundle.clone(); b.startup_source_vector[1].source_sha256 = "0".repeat(64); invalid.push(b);
    let mut b = bundle.clone(); b.startup_source_vector[1].body_sha256 = "0".repeat(64); invalid.push(b);
    let mut b = bundle.clone(); b.startup_source_vector[1].source_relative_path = "../escape.md".into(); invalid.push(b);
    let mut b = bundle.clone(); b.startup_bundle_name = "../escape".into(); invalid.push(b);
    let mut b = bundle.clone(); recovery(&mut b); if let StartupMode::Recovery(w) = &mut b.startup_mode { w.startup_cohort = "c".repeat(64); } invalid.push(b);
    let mut b = bundle.clone(); recovery(&mut b); if let StartupMode::Recovery(w) = &mut b.startup_mode { w.startup_loaded_vector.swap(0, 1); } invalid.push(b);
    let mut b = bundle.clone(); recovery(&mut b); if let StartupMode::Recovery(w) = &mut b.startup_mode { w.startup_loaded_vector[0].source_revision = "c".repeat(40); } invalid.push(b);
    for (index, b) in invalid.iter().enumerate() { assert!(startup::outputs(&skills, &[b.clone()]).is_err(), "case {index}"); }
    fs::write(&skills[1].path, "changed after registry resolution").unwrap();
    assert!(startup::outputs(&skills, &[bundle]).is_err());
}

#[test]
fn unsupported_execution_metadata_refuses_even_when_hashes_match() {
    for metadata in ["tools: Bash", "context: fork", "hooks: {}", "agent: unsafe", "user-invocable: false"] {
        let root = tempfile::tempdir().unwrap();
        let (mut skills, mut bundle) = fixture(root.path());
        skills[1].body = skills[1].body.replacen("description: fixture", metadata, 1);
        fs::write(&skills[1].path, &skills[1].body).unwrap();
        bundle.startup_source_vector[1].source_sha256 = digest(&skills[1].body);
        assert!(startup::outputs(&skills, &[bundle]).is_err(), "{metadata}");
    }
}

#[test]
fn existing_projector_manages_composite_and_detects_sidecar_drift() {
    let source = tempfile::tempdir().unwrap(); let workspace = tempfile::tempdir().unwrap();
    let (skills, bundle) = fixture(source.path());
    let plan = RolePlan { role_packet_plan_vector: vec![], first_string_vector: vec![], second_string_vector: vec![], string: String::new(), startup_bundle_vector: vec![bundle.clone()] };
    let projector = SkillProjector::at(workspace.path().into());
    projector.write(&skills, &plan).unwrap(); projector.check(&skills, &plan).unwrap();
    assert!(workspace.path().join(startup::skill_path(&bundle)).exists());
    assert!(!workspace.path().join(format!(".codex/skills/{}/SKILL.md", bundle.startup_bundle_name)).exists());
    fs::write(workspace.path().join(startup::manifest_path(&bundle)), "{}").unwrap();
    assert!(projector.check(&skills, &plan).is_err());
    let mut removed = plan.clone(); removed.startup_bundle_vector.clear();
    removed.second_string_vector = vec![startup::skill_path(&bundle).to_string_lossy().into_owned(), startup::manifest_path(&bundle).to_string_lossy().into_owned()];
    projector.write(&skills, &removed).unwrap(); projector.check(&skills, &removed).unwrap();
    assert!(!workspace.path().join(startup::manifest_path(&bundle)).exists());
    assert!(!workspace.path().join(startup::skill_path(&bundle)).exists());
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&plan).unwrap();
    assert_eq!(rkyv::from_bytes::<RolePlan, rkyv::rancor::Error>(&bytes).unwrap(), plan);
}
