//! In-memory startup composition from the server's three authored repositories.
//! Callers supply module IDs and a metaflow address, never paths or text.
use crate::{
    generated::{ContextProvenance, StartupComposition, StartupPlacement, StartupSelection},
    registry::{ExpandsSkills, SkillRegistry},
    startup_metadata::{declared_placement, instructions, sha256},
};
use std::collections::{BTreeMap, BTreeSet};

pub fn compose(
    registry: &SkillRegistry,
    selection: &StartupSelection,
) -> Result<StartupComposition, String> {
    let caller = &selection.caller_metaflow;
    if !word(&caller.first_string)
        || !word(&caller.second_string)
        || !matches!(
            caller.third_string.as_str(),
            "Primary" | "Secondary" | "Tertiary" | "Quaternary"
        )
    {
        return Err("invalid metaflow address".into());
    }
    let mut explicit = BTreeMap::new();
    for (names, system) in [
        (&selection.system_modules, true),
        (&selection.user_modules, false),
    ] {
        for name in names {
            if !identifier(name) || explicit.insert(name.clone(), system).is_some() {
                return Err("invalid, duplicate or cross-placement module ID".into());
            }
        }
    }
    if explicit.is_empty() {
        return Err("startup requires at least one authored module".into());
    }
    let system_closure = registry
        .expand_skills(&selection.system_modules)
        .map_err(|e| e.to_string())?;
    let user_closure = registry
        .expand_skills(&selection.user_modules)
        .map_err(|e| e.to_string())?;
    let systems: BTreeSet<_> = system_closure.iter().map(|s| s.name.as_str()).collect();
    let users: BTreeSet<_> = user_closure.iter().map(|s| s.name.as_str()).collect();
    let roots: Vec<_> = selection
        .system_modules
        .iter()
        .chain(&selection.user_modules)
        .cloned()
        .collect();
    let resolved = registry
        .resolved_sources(&roots)
        .map_err(|e| e.to_string())?;
    let snapshot = registry.expand_skills(&roots).map_err(|e| e.to_string())?;
    let mut system = Vec::new();
    let mut user = Vec::new();
    let mut provenance = Vec::new();
    for (module, source) in resolved.into_iter().zip(snapshot) {
        let actual = std::fs::read(&source.path).map_err(|e| e.to_string())?;
        if actual != source.body.as_bytes() {
            return Err(format!(
                "source changed since registry admission: {}",
                source.name
            ));
        }
        let body = instructions(&source.body)?;
        let declared = match declared_placement(&source.body) {
            "system" => Some(true),
            "user" => Some(false),
            "" => None,
            _ => return Err(format!("module {} is not startup-placed", source.name)),
        };
        let selected = explicit.get(&source.name).copied();
        if selected.zip(declared).is_some_and(|(a, b)| a != b) {
            return Err(format!(
                "placement conflicts with authored module {}",
                source.name
            ));
        }
        let system_only = source.name.starts_with("spirit-") || source.name.starts_with("intent-");
        if system_only && (selected == Some(false) || declared == Some(false)) {
            return Err(format!("spirit/intent must be SYSTEM: {}", source.name));
        }
        let is_system = match selected
            .or(declared)
            .or(if system_only { Some(true) } else { None })
        {
            Some(value) => value,
            None => match (
                systems.contains(source.name.as_str()),
                users.contains(source.name.as_str()),
            ) {
                (true, false) => true,
                (false, true) => false,
                _ => {
                    return Err(format!(
                        "dependency {} has ambiguous placement",
                        source.name
                    ));
                }
            },
        };
        let output = if is_system { &mut system } else { &mut user };
        let ordinal = output.len() as i64;
        let rendered = crate::context_render::render(&selection.target, body)?;
        let rendered_sha256 = sha256(rendered.as_bytes());
        output.push((source.name, rendered));
        provenance.push(ContextProvenance {
            module_identity: module.module_identity,
            repository_name: module.repository_name,
            source_revision: module.source_revision,
            source_relative_path: module.source_relative_path,
            source_sha256: module.source_sha256,
            body_sha256: module.body_sha256,
            rendered_sha256,
            startup_placement: if is_system {
                StartupPlacement::System
            } else {
                StartupPlacement::User
            },
            integer: ordinal,
        });
    }
    if !system.iter().any(|(id, _)| id.starts_with("spirit-"))
        || !system.iter().any(|(id, _)| id.starts_with("intent-"))
    {
        return Err("startup requires explicit Spirit and seat-intent SYSTEM context".into());
    }
    let system_text = render(&system);
    let user_text = render(&user);
    Ok(StartupComposition {
        system_payload_sha256: sha256(system_text.as_bytes()),
        user_payload_sha256: sha256(user_text.as_bytes()),
        system_text,
        user_text,
        context_provenance_vector: provenance,
    })
}

fn render(modules: &[(String, String)]) -> String {
    if modules.is_empty() {
        return String::new();
    }
    let mut text = format!(
        "These are your skills, given here; do not reload them.\n\nModule IDs: {}\n",
        modules
            .iter()
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    for (id, body) in modules {
        text.push_str(&format!(
            "\n<!-- module: {id} -->\n{body}\n<!-- end-module: {id} -->\n"
        ));
    }
    text
}
fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn word(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
}
