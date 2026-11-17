//! Role metadata contains IDs only. Authored text is resolved by the same registry.
use crate::panels::{PanelPrompt, PanelRendering};
use crate::{
    generated::{RoleDefinition, Roles},
    registry::{IdentityText, SkillRegistry, valid_id, valid_segment},
    startup_metadata::sha256,
};
use signal_curriculum_nexus::{
    ContextPayloadSha256, ContextProvenance, ContextText, DescriptionPayloadSha256,
    DescriptionText, RenderedSha256, RoleCallers, RoleComposition, RoleSelection, RoleTools,
    StartupPlacement, Surface, Target,
};

pub fn validate(roles: &Roles) -> Result<(), String> {
    let mut seen = std::collections::BTreeSet::new();
    for role in &roles.role_definition_vector {
        let surface = format!("{:?}", role.surface);
        if !valid_segment(&role.role_name.0)
            || !seen.insert((role.role_name.clone(), surface))
            || role.model_id.0.is_empty()
            || !role
                .model_id
                .0
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-_".contains(&b))
        {
            return Err("invalid or duplicate role metadata".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        for id in std::iter::once(&role.description_module.0).chain(&role.context_modules.0) {
            let text = id.identity_text().map_err(|_| "invalid role module ID")?;
            if !valid_id(&text) || !ids.insert(text) {
                return Err("invalid or duplicate role module ID".into());
            }
        }
        if role.context_modules.0.is_empty() {
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
    let ids: Vec<String> = std::iter::once(&role.description_module.0)
        .chain(&role.context_modules.0)
        .map(|id| id.identity_text())
        .collect::<Result<Vec<_>, _>>()?;
    // Deliberately binding-only: no dependency closure, universal module or
    // ambient file. A numeric binding can select several actual modules.
    let modules = registry
        .resolved_exact_sources(&ids)
        .map_err(|e| e.to_string())?;
    let description_identities = registry
        .resolved_exact_sources(std::slice::from_ref(
            &role.description_module.0.identity_text()?,
        ))
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|module| module.module_identity.identity_text())
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
    let target = match role.surface {
        Surface::ClaudeAgent => Target::Claude,
        Surface::CodexAgent => Target::Codex,
        Surface::PiAgent => Target::Pi,
    };
    let description_panels = PanelPrompt::for_registry(registry);
    let context_panels = PanelPrompt::for_registry(registry);
    let rendered: Vec<String> = modules
        .iter()
        .map(|m| {
            if m.module_identity.module_kind == signal_curriculum_nexus::ModuleKind::Panel {
                return Err("a Panel is composed only through a mark in its master".into());
            }
            let panels = if description_identities.contains(&m.module_identity.identity_text()?) {
                &description_panels
            } else {
                &context_panels
            };
            panels.render(&target, &m.string)
        })
        .collect::<Result<_, _>>()?;
    let description_text = modules
        .iter()
        .zip(&rendered)
        .filter(|(module, _)| {
            module
                .module_identity
                .identity_text()
                .map(|id| description_identities.contains(&id))
                .unwrap_or(false)
        })
        .map(|(_, body)| body.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    let context_modules = modules
        .iter()
        .zip(&rendered)
        .filter(|(module, _)| {
            module
                .module_identity
                .identity_text()
                .map(|id| !description_identities.contains(&id))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    let mut context_text = format!(
        "These are your skills, given here; do not reload them.\n\nModule IDs: {}\n",
        context_modules
            .iter()
            .map(|(module, _)| module.module_identity.identity_text())
            .collect::<Result<Vec<_>, _>>()?
            .join(", ")
    );
    for (module, body) in context_modules {
        context_text.push_str(&format!(
            "\n<!-- module: {} -->\n{}\n<!-- end-module: {} -->\n",
            module.module_identity.identity_text()?,
            body,
            module.module_identity.identity_text()?
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
            rendered_sha256: RenderedSha256(sha256(rendered[ordinal].as_bytes())),
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
        description_payload_sha256: DescriptionPayloadSha256(sha256(description_text.as_bytes())),
        context_payload_sha256: ContextPayloadSha256(sha256(context_text.as_bytes())),
        description_text: DescriptionText(description_text),
        context_text: ContextText(context_text),
        role_tools: role.role_tools.clone(),
        role_callers: role.role_callers.clone(),
        context_provenance_vector: provenance,
        missing_panels: {
            let mut missing = description_panels.missing()?;
            for name in context_panels.missing()?.0 {
                if !missing.0.contains(&name) {
                    missing.0.push(name);
                }
            }
            missing
        },
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
                reader: ReaderBudget::try_from(1_048_576).expect("a constant reader budget"),
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
