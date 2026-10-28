//! Role metadata contains IDs only. Authored text is resolved by the same registry.
use crate::{
    generated::{
        ContextProvenance, RoleCallers, RoleComposition, RoleDefinition, RoleSelection, RoleTools,
        Roles, StartupPlacement,
    },
    registry::{SkillRegistry, valid_id},
    startup_metadata::sha256,
};

pub fn validate(roles: &Roles) -> Result<(), String> {
    let mut seen = std::collections::BTreeSet::new();
    for role in &roles.role_definition_vector {
        let surface = format!("{:?}", role.surface);
        if !valid_id(&role.role_name)
            || !seen.insert((role.role_name.clone(), surface))
            || role.model_id.is_empty()
            || !role
                .model_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-_".contains(&b))
        {
            return Err("invalid or duplicate role metadata".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        for id in std::iter::once(&role.description_module).chain(&role.context_modules) {
            if !valid_id(id) || !ids.insert(id) {
                return Err("invalid or duplicate role module ID".into());
            }
        }
        if role.context_modules.is_empty() {
            return Err("role context must not be empty".into());
        }
        if let RoleTools::Explicit(tools) = &role.role_tools {
            let mut seen = std::collections::BTreeSet::new();
            for tool in tools {
                if !matches!(
                    tool.as_str(),
                    "Read"
                        | "Grep"
                        | "Glob"
                        | "Bash"
                        | "Edit"
                        | "Write"
                        | "WebFetch"
                        | "WebSearch"
                        | "LSP"
                        | "Agent"
                        | "Artifact"
                        | "ToolSearch"
                        | "ArtifactData"
                        | "ArtifactComments"
                ) || !seen.insert(tool)
                {
                    return Err("invalid or duplicate role tool".into());
                }
            }
        }
        if let RoleCallers::Only(callers) = &role.role_callers {
            let mut seen = std::collections::BTreeSet::new();
            if callers.is_empty() {
                return Err("empty caller allowlist".into());
            }
            for caller in callers {
                if !word(&caller.first_string)
                    || !word(&caller.second_string)
                    || !matches!(
                        caller.third_string.as_str(),
                        "Primary" | "Secondary" | "Tertiary" | "Quaternary"
                    )
                    || !seen.insert((
                        &caller.first_string,
                        &caller.second_string,
                        &caller.third_string,
                    ))
                {
                    return Err("invalid or duplicate caller tuple".into());
                }
            }
        }
    }
    Ok(())
}
fn word(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

pub fn compose(
    registry: &SkillRegistry,
    roles: &Roles,
    selection: &RoleSelection,
) -> Result<RoleComposition, String> {
    validate(roles)?;
    let role: &RoleDefinition = roles
        .role_definition_vector
        .iter()
        .find(|r| r.role_name == selection.role_name && r.surface == selection.surface)
        .ok_or("unknown role/surface")?;
    let ids: Vec<_> = std::iter::once(&role.description_module)
        .chain(&role.context_modules)
        .cloned()
        .collect();
    let (available_ids, omitted_modules) = registry
        .available_declared(&ids)
        .map_err(|e| e.to_string())?;
    // Deliberately exact: no dependency closure, universal module or ambient file.
    let modules = registry
        .resolved_exact_sources(&available_ids)
        .map_err(|e| e.to_string())?;
    let target = match role.surface {
        crate::generated::Surface::ClaudeAgent => crate::generated::Target::Claude,
        crate::generated::Surface::CodexAgent => crate::generated::Target::Codex,
        crate::generated::Surface::PiAgent => crate::generated::Target::Pi,
    };
    let rendered: Vec<String> = modules
        .iter()
        .map(|m| crate::context_render::render(&target, &m.string))
        .collect::<Result<_, _>>()?;
    let description_text = modules
        .iter()
        .zip(&rendered)
        .find(|(module, _)| module.module_identity == role.description_module)
        .map(|(_, body)| body.clone())
        .unwrap_or_default();
    let available_context_modules: Vec<_> = modules
        .iter()
        .filter(|module| module.module_identity != role.description_module)
        .map(|module| module.module_identity.as_str())
        .collect();
    let mut context_text = format!(
        "These are your skills, given here; do not reload them.\n\nModule IDs: {}\n",
        available_context_modules.join(", ")
    );
    for (module, body) in modules.iter().zip(&rendered) {
        if module.module_identity == role.description_module {
            continue;
        }
        context_text.push_str(&format!(
            "\n<!-- module: {} -->\n{}\n<!-- end-module: {} -->\n",
            module.module_identity, body, module.module_identity
        ));
    }
    let provenance = modules
        .into_iter()
        .enumerate()
        .map(|(ordinal, m)| ContextProvenance {
            module_identity: m.module_identity,
            repository_name: m.repository_name,
            source_revision: m.source_revision,
            source_relative_path: m.source_relative_path,
            source_sha256: m.source_sha256,
            body_sha256: m.body_sha256,
            rendered_sha256: sha256(rendered[ordinal].as_bytes()),
            startup_placement: StartupPlacement::System,
            integer: ordinal as i64,
        })
        .collect();
    Ok(RoleComposition {
        role_name: role.role_name.clone(),
        surface: role.surface.clone(),
        model_id: role.model_id.clone(),
        effort: role.effort.clone(),
        permission: role.permission.clone(),
        description_payload_sha256: sha256(description_text.as_bytes()),
        context_payload_sha256: sha256(context_text.as_bytes()),
        description_text,
        context_text,
        role_tools: role.role_tools.clone(),
        role_callers: role.role_callers.clone(),
        omitted_modules,
        context_provenance_vector: provenance,
    })
}

#[cfg(feature = "datom")]
pub fn read_declared() -> Result<Option<Roles>, String> {
    use datom_codec::{Actualizing, Budget, Potential};
    use protos::ReaderBudget;
    let Some(path) = std::env::var_os("CURRICULUM_ROLES_FILE") else {
        return Ok(None);
    };
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let crate::generated::RolesDocument::Roles(roles) =
        Potential::<crate::generated::RolesDocument>::from(text)
            .actualize(&mut Budget {
                remaining: 1_048_576,
                reader: ReaderBudget {
                    remaining: 1_048_576,
                },
                depth: 0,
                maximum_depth: 1024,
            })
            .map_err(|_| "invalid ID-only roles declaration".to_owned())?;
    validate(&roles)?;
    Ok(Some(roles))
}
#[cfg(not(feature = "datom"))]
pub fn read_declared() -> Result<Option<Roles>, String> {
    if std::env::var_os("CURRICULUM_ROLES_FILE").is_some() {
        Err("declared roles require the datom package feature".into())
    } else {
        Ok(None)
    }
}
