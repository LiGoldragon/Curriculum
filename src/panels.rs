//! A composition owns its panel set; each skill starts a separate composition.
use crate::registry::{IdentityText, SkillRegistry};
use signal_curriculum_nexus::{MissingPanels, ModuleIdentity, ModuleKind, ResolvedModule};
use std::collections::BTreeSet;

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
}
pub trait PanelRendering: Sized {
    fn for_registry(registry: &SkillRegistry) -> Self;
    fn render(&self, target: &Target, source: &str) -> Result<String, String>;
    fn missing(&self) -> Result<MissingPanels, String>;
}
impl PanelRendering for PanelPrompt {
    fn for_registry(registry: &SkillRegistry) -> Self {
        Self {
            state: Arc::new(Mutex::new(PanelPromptState {
                registry: registry.clone(),
                included: BTreeSet::new(),
                missing: MissingPanels(vec![]),
            })),
        }
    }
    fn render(&self, target: &Target, source: &str) -> Result<String, String> {
        let mut environment = Environment::empty();
        environment.set_keep_trailing_newline(true);
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
                    "{{% if __panel_claim({name:?}) %}}{}{{% endif %}}",
                    source.string
                ));
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
                Ok(state.included.insert(name))
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
}
