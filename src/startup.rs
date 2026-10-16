//! Main-seat USER composites from the existing resolved registry, not a source registry.
use std::{collections::{BTreeMap, BTreeSet}, path::{Component, Path, PathBuf}};
use crate::{generated::{StartupBundle, StartupMode, StartupPlacement}, registry::{ResolvedSkill, expand_resolved_skills}, startup_metadata::{instructions, quote, sha256}};

pub fn skill_path(bundle: &StartupBundle) -> PathBuf {
    PathBuf::from(format!(".claude/skills/{}/SKILL.md", bundle.startup_bundle_name))
}

pub fn manifest_path(bundle: &StartupBundle) -> PathBuf {
    PathBuf::from(format!("tools/startup-bundles/{}.json", bundle.startup_bundle_name))
}

pub fn outputs(skills: &[ResolvedSkill], bundles: &[StartupBundle]) -> Result<BTreeMap<PathBuf, String>, String> {
    let mut result = BTreeMap::new();
    let mut names = BTreeSet::new();
    for bundle in bundles {
        let name = &bundle.startup_bundle_name;
        if !name.starts_with("startup-") || !identifier(name) || !names.insert(name)
            || skills.iter().any(|skill| &skill.name == name) {
            return Err("invalid, duplicate or source-colliding startup bundle name".into());
        }
        let (body, manifest) = compose(skills, bundle)?;
        result.insert(skill_path(bundle), body);
        result.insert(manifest_path(bundle), manifest);
    }
    Ok(result)
}

fn compose(skills: &[ResolvedSkill], bundle: &StartupBundle) -> Result<(String, String), String> {
    let tuple = &bundle.caller_metaflow;
    if !word(&tuple.first_string) || !word(&tuple.second_string)
        || !matches!(tuple.third_string.as_str(), "Primary" | "Secondary" | "Tertiary" | "Quaternary")
        || !hash(&bundle.startup_cohort, 64) {
        return Err("invalid startup tuple or cohort digest".into());
    }
    let mut roots = BTreeSet::new();
    if bundle.module_identity_vector.is_empty() || bundle.module_identity_vector.iter().any(|root| !identifier(root) || !roots.insert(root)) {
        return Err("startup requires unique USER roots".into());
    }
    let closure = expand_resolved_skills(skills, &bundle.module_identity_vector).map_err(|error| error.to_string())?;
    let mut sources = BTreeMap::new();
    for source in &bundle.startup_source_vector {
        if sources.insert(source.module_identity.as_str(), source).is_some() {
            return Err("duplicate source/placement assignment".into());
        }
    }
    if sources.len() != closure.len() { return Err("source assignments do not equal complete closure".into()); }
    for root in &bundle.module_identity_vector {
        if !matches!(sources.get(root.as_str()).map(|s| &s.startup_placement), Some(StartupPlacement::User)) {
            return Err("startup root is not USER placed".into());
        }
    }
    let loaded = match &bundle.startup_mode {
        StartupMode::Initial => &[][..],
        StartupMode::Recovery(witness) => {
            if witness.native_session.trim().is_empty() || witness.prompt_identity.trim().is_empty()
                || witness.startup_cohort != bundle.startup_cohort || witness.startup_loaded_vector.is_empty() {
                return Err("recovery requires session, prompt and same-cohort loaded witness".into());
            }
            witness.startup_loaded_vector.as_slice()
        }
    };
    let mut body = format!("---\nname: {}\ndescription: Explicit main-seat USER startup bundle.\ndisable-model-invocation: true\nuser-invocable: true\n---\n", bundle.startup_bundle_name);
    let mut rows = Vec::new();
    let mut user_index = 0usize;
    let mut emitted = 0usize;
    for skill in &closure {
        let source = sources.get(skill.name.as_str()).ok_or("missing source/placement assignment")?;
        let root = std::fs::canonicalize(&source.source_root).map_err(|error| error.to_string())?;
        let relative = Path::new(&source.source_relative_path);
        if relative.as_os_str().is_empty() || !relative.components().all(|component| matches!(component, Component::Normal(_)))
            || !word(&source.repository_name) || !(hash(&source.source_revision, 40) || hash(&source.source_revision, 64)) {
            return Err("invalid repository revision or relative source path".into());
        }
        let selected_path = std::fs::canonicalize(root.join(relative)).map_err(|error| error.to_string())?;
        let resolved_path = std::fs::canonicalize(&skill.path).map_err(|error| error.to_string())?;
        if !selected_path.starts_with(&root) || selected_path != resolved_path {
            return Err("source provenance disagrees with registered path".into());
        }
        let actual_source = std::fs::read(&resolved_path).map_err(|error| error.to_string())?;
        if actual_source != skill.body.as_bytes() || sha256(&actual_source) != source.source_sha256 {
            return Err("source changed or source hash mismatch".into());
        }
        let text = instructions(&skill.body)?;
        if sha256(text.as_bytes()) != source.body_sha256 { return Err("instruction body hash mismatch".into()); }
        let (placement, disposition, ordinal) = match source.startup_placement {
            StartupPlacement::System => ("system", "already-composed-system", "null".to_owned()),
            StartupPlacement::User => {
                let skipped = if let Some(witness) = loaded.get(user_index) {
                    if witness.module_identity != skill.name || witness.repository_name != source.repository_name
                        || witness.source_revision != source.source_revision || witness.source_relative_path != source.source_relative_path
                        || witness.source_sha256 != source.source_sha256 || witness.body_sha256 != source.body_sha256 {
                        return Err("loaded witness is not the exact same-source USER prefix".into());
                    }
                    true
                } else { false };
                user_index += 1;
                if skipped { ("user", "previously-loaded", "null".to_owned()) }
                else {
                    let ordinal = emitted.to_string(); emitted += 1;
                    body.push_str(&format!("\n<!-- module: {} -->\n", skill.name));
                    body.push_str(text);
                    body.push_str(&format!("\n<!-- end-module: {} -->\n", skill.name));
                    ("user", "emitted", ordinal)
                }
            }
        };
        rows.push(format!("{{\"id\":{},\"repository\":{},\"revision\":{},\"sourceRoot\":{},\"relativePath\":{},\"sourceSha256\":{},\"bodySha256\":{},\"placement\":{},\"disposition\":{},\"ordinal\":{}}}", quote(&skill.name), quote(&source.repository_name), quote(&source.source_revision), quote(&root.to_string_lossy()), quote(&source.source_relative_path), quote(&source.source_sha256), quote(&source.body_sha256), quote(placement), quote(disposition), ordinal));
    }
    if loaded.len() > user_index || emitted == 0 { return Err("recovery prefix exceeds USER closure or nothing remains".into()); }
    let mode = match &bundle.startup_mode {
        StartupMode::Initial => "{\"kind\":\"initial\"}".into(),
        StartupMode::Recovery(witness) => format!("{{\"kind\":\"recovery\",\"session\":{},\"promptId\":{},\"evidenceStatus\":\"supplied-witness-matched-not-native-reverified\"}}", quote(&witness.native_session), quote(&witness.prompt_identity)),
    };
    let manifest = format!("{{\"schemaVersion\":1,\"bundle\":{},\"metaflow\":{{\"aspect\":{},\"topic\":{},\"layer\":{}}},\"cohort\":{},\"userRoots\":{},\"mode\":{},\"systemCompositionStatus\":\"declared-by-cohort-not-native-reverified\",\"userCount\":{},\"emittedCount\":{},\"skillSha256\":{},\"modules\":[{}]}}\n", quote(&bundle.startup_bundle_name), quote(&tuple.first_string), quote(&tuple.second_string), quote(&tuple.third_string), quote(&bundle.startup_cohort), crate::role_policy::string_array(&bundle.module_identity_vector), mode, user_index, emitted, quote(&sha256(body.as_bytes())), rows.join(","));
    Ok((body, manifest))
}

fn identifier(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn word(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}
fn hash(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
