//! A composition owns its panel set; each skill starts a separate composition.
use crate::generated::PanelCycle;
use crate::registry::{IdentityText, SkillRegistry};
use crate::startup_metadata::sha256;
use signal_curriculum_nexus::{
    MergedPanels, MissingPanels, ModuleIdentity, ModuleKind, PanelProvenance, RenderedSha256,
    ResolvedModule,
};
use std::collections::{BTreeMap, BTreeSet};

/// The lookup is the same declared-source registry used by ordinary modules.
pub trait PanelLookup {
    fn panel_sources(&self, name: &ModuleIdentity) -> Result<Vec<ResolvedModule>, String>;
}
impl PanelLookup for SkillRegistry {
    fn panel_sources(&self, name: &ModuleIdentity) -> Result<Vec<ResolvedModule>, String> {
        if name.module_kind != ModuleKind::Panel {
            return Err("a panel mark must name a Panel".into());
        }
        let name = name.identity_text()?;
        match self.resolved_exact_sources(&[name]) {
            Ok(sources) => Ok(sources),
            Err(crate::registry::RegistryError::Missing(_)) => Ok(vec![]),
            Err(error) => Err(error.to_string()),
        }
    }
}

use crate::registry::ModuleIdentityConversion;
use minijinja::{Environment, Error, ErrorKind, UndefinedBehavior, context};
use signal_curriculum_nexus::Target;
use std::sync::{Arc, Mutex};

pub struct PanelPrompt {
    state: Arc<Mutex<PanelPromptState>>,
}
struct PanelPromptState {
    registry: SkillRegistry,
    included: BTreeSet<String>,
    missing: MissingPanels,
    active: Vec<ModuleIdentity>,
    cycles: Vec<PanelCycle>,
    order: Vec<String>,
    sources: BTreeMap<String, ResolvedModule>,
    merged: BTreeMap<String, PanelProvenance>,
}
pub trait PanelRendering: Sized {
    fn for_registry(registry: &SkillRegistry) -> Self;
    fn render(&self, target: &Target, source: &str) -> Result<String, String>;
    fn missing(&self) -> Result<MissingPanels, String>;
    fn provenance(&self) -> Result<MergedPanels, String>;
    fn cycles(&self) -> Result<Vec<PanelCycle>, String>;
}
impl PanelRendering for PanelPrompt {
    fn for_registry(registry: &SkillRegistry) -> Self {
        Self {
            state: Arc::new(Mutex::new(PanelPromptState {
                registry: registry.clone(),
                included: BTreeSet::new(),
                missing: MissingPanels(vec![]),
                active: vec![],
                cycles: vec![],
                order: vec![],
                sources: BTreeMap::new(),
                merged: BTreeMap::new(),
            })),
        }
    }
    fn render(&self, target: &Target, source: &str) -> Result<String, String> {
        let mut environment = Environment::empty();
        environment.set_keep_trailing_newline(true);
        environment.set_trim_blocks(true);
        environment.set_lstrip_blocks(true);
        environment.set_undefined_behavior(UndefinedBehavior::Strict);
        let state = Arc::clone(&self.state);
        environment.set_loader(move |name| {
            let name = name
                .module_identity()
                .map_err(|error| Error::new(ErrorKind::InvalidOperation, error))?;
            let mut state = state
                .lock()
                .map_err(|error| Error::new(ErrorKind::InvalidOperation, error.to_string()))?;
            let sources = state
                .registry
                .panel_sources(&name)
                .map_err(|error| Error::new(ErrorKind::InvalidOperation, error))?;
            if sources.is_empty() && !state.missing.0.contains(&name) {
                state.missing.0.push(name);
            }
            let mut text = String::new();
            for source in sources {
                let name = source
                    .module_identity
                    .identity_text()
                    .map_err(|error| Error::new(ErrorKind::InvalidOperation, error))?;
                text.push_str(&format!(
                    "{{% if __panel_claim({name:?}) %}}{{% set __panel_body %}}{}{{% endset %}}{{{{ __panel_record({name:?}, __panel_body) }}}}{{% endif %}}",
                    source.string
                ));
                state.sources.insert(name, source);
            }
            Ok(Some(text))
        });
        let state = Arc::clone(&self.state);
        environment.add_function(
            "__panel_claim",
            move |name: String| -> Result<bool, Error> {
                let mut state = state
                    .lock()
                    .map_err(|error| Error::new(ErrorKind::InvalidOperation, error.to_string()))?;
                let identity = name
                    .module_identity()
                    .map_err(|error| Error::new(ErrorKind::InvalidOperation, error))?;
                if let Some(index) = state.active.iter().position(|active| active == &identity) {
                    let mut cycle = state.active[index..].to_vec();
                    cycle.push(identity);
                    let cycle = PanelCycle(cycle);
                    if !state.cycles.contains(&cycle) {
                        state.cycles.push(cycle);
                    }
                    return Ok(false);
                }
                if !state.included.insert(name.clone()) {
                    return Ok(false);
                }
                state.order.push(name);
                state.active.push(identity);
                Ok(true)
            },
        );
        let state = Arc::clone(&self.state);
        environment.add_function(
            "__panel_record",
            move |name: String, body: String| -> Result<String, Error> {
                let mut state = state
                    .lock()
                    .map_err(|error| Error::new(ErrorKind::InvalidOperation, error.to_string()))?;
                let source = state
                    .sources
                    .get(&name)
                    .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "panel was not loaded"))?
                    .clone();
                if state.active.pop().as_ref() != Some(&source.module_identity) {
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        "panel inclusion stack disagrees",
                    ));
                }
                state.merged.insert(
                    name,
                    PanelProvenance {
                        module_identity: source.module_identity,
                        repository_name: source.repository_name,
                        source_revision: source.source_revision,
                        source_relative_path: source.source_relative_path,
                        source_sha256: source.source_sha256,
                        body_sha256: source.body_sha256,
                        rendered_sha256: RenderedSha256(sha256(body.as_bytes())),
                    },
                );
                Ok(body)
            },
        );
        environment
            .render_str(
                source,
                context!(
                    claude => matches!(target, Target::Claude),
                    codex => matches!(target, Target::Codex),
                    pi => matches!(target, Target::Pi),
                    opencode => matches!(target, Target::OpenCode),
                ),
            )
            .map_err(|error| {
                if error.kind() == ErrorKind::UndefinedError {
                    "unresolved context variable; no caller-inline substitution is allowed".into()
                } else {
                    error.to_string()
                }
            })
    }
    fn missing(&self) -> Result<MissingPanels, String> {
        self.state
            .lock()
            .map(|state| state.missing.clone())
            .map_err(|error| error.to_string())
    }
    fn provenance(&self) -> Result<MergedPanels, String> {
        let state = self.state.lock().map_err(|error| error.to_string())?;
        state
            .order
            .iter()
            .map(|name| {
                state
                    .merged
                    .get(name)
                    .cloned()
                    .ok_or_else(|| "incomplete panel render".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()
            .map(MergedPanels)
    }
    fn cycles(&self) -> Result<Vec<PanelCycle>, String> {
        self.state
            .lock()
            .map(|state| state.cycles.clone())
            .map_err(|error| error.to_string())
    }
}
