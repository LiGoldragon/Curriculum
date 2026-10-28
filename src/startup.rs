//! In-memory startup composition from the server's three authored repositories.
//! Callers supply module IDs and a metaflow address, never paths or text.
use crate::{
    generated::{ContextProvenance, StartupComposition, StartupPlacement, StartupSelection},
    registry::{ExpandsSkills, SkillRegistry},
    startup_metadata::{instructions, sha256},
};
use std::{collections::BTreeSet, path::Path};

pub fn compose(
    registry: &SkillRegistry,
    records_root: &Path,
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
    let mut explicit = BTreeSet::new();
    for (names, system) in [
        (&selection.system_modules, true),
        (&selection.user_modules, false),
    ] {
        for name in names {
            if !identifier(name) || !explicit.insert(name.clone()) {
                return Err("invalid, duplicate or cross-placement module ID".into());
            }
            if system != system_module(name) {
                return Err(format!("module has the wrong startup placement: {name}"));
            }
        }
    }
    if explicit.is_empty() {
        return Err("startup requires at least one authored module".into());
    }
    let roots: Vec<_> = selection
        .system_modules
        .iter()
        .chain(&selection.user_modules)
        .cloned()
        .collect();
    let (available_roots, omitted_modules) = registry
        .available_declared(&roots)
        .map_err(|e| e.to_string())?;
    let resolved = registry
        .resolved_sources(&available_roots)
        .map_err(|e| e.to_string())?;
    let snapshot = registry
        .expand_skills(&available_roots)
        .map_err(|e| e.to_string())?;
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
        let is_system = system_module(&source.name);
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
    if !system.iter().any(|(id, _)| id.starts_with("spirit:"))
        || !system.iter().any(|(id, _)| id.starts_with("intent:"))
    {
        return Err("startup requires explicit Spirit and seat-intent SYSTEM context".into());
    }
    let system_text = render(&system);
    let user_text = render(&user);
    let records_text = records(records_root, &selection.flow_id_vector)?;
    Ok(StartupComposition {
        system_payload_sha256: sha256(system_text.as_bytes()),
        user_payload_sha256: sha256(user_text.as_bytes()),
        records_payload_sha256: sha256(records_text.as_bytes()),
        omitted_modules,
        system_text,
        user_text,
        records_text,
        context_provenance_vector: provenance,
    })
}

fn records(root: &Path, flows: &[String]) -> Result<String, String> {
    let root =
        std::fs::canonicalize(root).map_err(|_| "record missing: records root".to_owned())?;
    let mut output = String::new();
    for flow in flows {
        if !flow_id(flow) {
            return Err(format!("record missing: {flow}"));
        }
        let flow_root = root.join(flow);
        if std::fs::symlink_metadata(&flow_root)
            .map(|metadata| !metadata.is_dir() || metadata.file_type().is_symlink())
            .unwrap_or(true)
        {
            return Err(format!("record missing: {flow}"));
        }
        let flow_root =
            std::fs::canonicalize(&flow_root).map_err(|_| format!("record missing: {flow}"))?;
        if !flow_root.starts_with(&root) {
            return Err(format!("record missing: {flow}"));
        }
        let mut paths = Vec::new();
        for topic in ["vision", "notion"] {
            let directory = flow_root.join(topic);
            match std::fs::symlink_metadata(&directory) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(_) => return Err(format!("record missing: {flow}")),
                Ok(metadata) if !metadata.is_dir() || metadata.file_type().is_symlink() => {
                    return Err(format!("record missing: {flow}"));
                }
                Ok(_) => {}
            }
            let mut topic_paths = std::fs::read_dir(&directory)
                .map_err(|_| format!("record missing: {flow}"))?
                .map(|entry| entry.map(|entry| entry.path()))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| format!("record missing: {flow}"))?;
            topic_paths.retain(|path| {
                path.extension().is_some_and(|extension| extension == "md")
                    && path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| !name.starts_with("archive-"))
            });
            topic_paths.sort();
            paths.extend(topic_paths);
        }
        if paths.is_empty() {
            return Err(format!("record missing: {flow}"));
        }
        for path in paths {
            let metadata =
                std::fs::symlink_metadata(&path).map_err(|_| format!("record missing: {flow}"))?;
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                return Err(format!("record missing: {flow}"));
            }
            let bytes = std::fs::read(&path).map_err(|_| format!("record missing: {flow}"))?;
            if std::fs::read(&path).ok().as_deref() != Some(bytes.as_slice()) {
                return Err(format!("record missing: {flow}"));
            }
            let text = String::from_utf8(bytes).map_err(|_| format!("record missing: {flow}"))?;
            if !output.is_empty() && !output.ends_with('\n') {
                output.push('\n');
            }
            output.push_str(&text);
            if !output.ends_with('\n') {
                output.push('\n');
            }
        }
    }
    Ok(output)
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
    value.split_once(':').is_some_and(|(kind, subject)| {
        matches!(
            kind,
            "spirit"
                | "intent"
                | "vision"
                | "knowledge"
                | "inquiry"
                | "compensation"
                | "role"
                | "raw"
        ) && !subject.is_empty()
            && !subject.contains(':')
            && subject
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    })
}
fn system_module(value: &str) -> bool {
    value.starts_with("spirit:") || value.starts_with("intent:")
}
fn flow_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}
fn word(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
}
