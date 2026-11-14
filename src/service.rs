use crate::{
    memory::SkillMemory,
    registry::{ExpandsSkills, SkillRegistry, SourceConfiguration},
};
use nexus::{Admission, Changed, MemoryHandle, Nexus, Operating, Remembering, Signaling};
use signal_curriculum_nexus::{
    Answerable, Answering, Input, ModuleIdentity, Output, ResolvedModule, RoleComposition,
    RoleSelection, StartupComposition, StartupSelection,
};
use std::path::PathBuf;

/// Curriculum's memory is the roles its roles file declared when it opened.
/// It is read once, at start, and never changed or stored.
impl Remembering for SkillMemory {
    type Change = ();
    type Reading = ();
    type Remembered = SkillMemory;
    fn open(admission: Admission) -> Option<Self> {
        if admission.directory() != CurriculumNexus::directory() {
            return None;
        }
        let registry = SourceConfiguration::from_environment()
            .and_then(|c| c.read().map_err(|e| e.to_string()))
            .map_err(|e| eprintln!("{e}"))
            .ok()?;
        // Interim, until Curriculum has an observer surface: each declaration
        // that put no module in the registry is written to stderr at start,
        // so it reaches the journal.
        for report in registry.reports() {
            eprintln!("{report}");
        }
        let roles_option = crate::roles::read_declared()
            .map_err(|e| eprintln!("{e}"))
            .ok()?;
        Some(Self { roles_option })
    }
    fn change(&mut self, _: ()) -> Changed {
        Changed::Failed
    }
    fn read(&self, _: ()) -> SkillMemory {
        self.clone()
    }
}

/// What answers one input: the registry as its sources stand when the input
/// arrives, and the memory.
pub struct Curriculum {
    registry: SkillRegistry,
    memory: SkillMemory,
}

impl Answering for Curriculum {
    fn resolve_skills(
        &self,
        module_identity_vector: &Vec<ModuleIdentity>,
    ) -> Result<Vec<ModuleIdentity>, Output> {
        self.registry
            .expand_skills(&identities(module_identity_vector))
            .map(|skills| {
                skills
                    .into_iter()
                    .map(|skill| ModuleIdentity(skill.name))
                    .collect()
            })
            .map_err(|e| Output::Rejected(e.to_string()))
    }

    fn resolve_sources(
        &self,
        module_identity_vector: &Vec<ModuleIdentity>,
    ) -> Result<Vec<ResolvedModule>, Output> {
        self.registry
            .resolved_sources(&identities(module_identity_vector))
            .map_err(|e| Output::Rejected(e.to_string()))
    }

    fn compose_startup(
        &self,
        startup_selection: &StartupSelection,
    ) -> Result<StartupComposition, Output> {
        let records_root = std::env::var_os("CURRICULUM_RECORDS_DIR")
            .map(PathBuf::from)
            .ok_or_else(|| Output::Rejected("missing CURRICULUM_RECORDS_DIR".into()))?;
        crate::startup::compose(&self.registry, &records_root, startup_selection)
            .map_err(Output::Rejected)
    }

    fn compose_role(&self, role_selection: &RoleSelection) -> Result<RoleComposition, Output> {
        let roles =
            self.memory.roles_option.as_ref().ok_or_else(|| {
                Output::Rejected("no server-declared ID-only roles metadata".into())
            })?;
        crate::roles::compose(&self.registry, roles, role_selection).map_err(Output::Rejected)
    }
}

pub struct CurriculumOperation;
impl Operating<SkillMemory> for CurriculumOperation {
    type Operation = Input;
    type Outcome = Output;
    async fn perform(&mut self, input: Input, memory: &MemoryHandle<SkillMemory>) -> Output {
        let Some(memory) = memory.read(()).await else {
            return Output::Rejected("Curriculum memory unavailable".into());
        };
        let registry = match SourceConfiguration::from_environment()
            .and_then(|configuration| configuration.read().map_err(|e| e.to_string()))
        {
            Ok(registry) => registry,
            Err(error) => return Output::Rejected(error),
        };
        input.answer(&Curriculum { registry, memory })
    }
}

/// The registry names a module by its identity's text.
fn identities(ids: &[ModuleIdentity]) -> Vec<String> {
    ids.iter().map(|ModuleIdentity(id)| id.clone()).collect()
}

pub struct CurriculumSignal;

impl Signaling for CurriculumSignal {
    type Query = Input;
    type Response = Output;
    type Operation = Input;
    type Outcome = Output;

    fn decode(&self, frame: &[u8]) -> Option<Self::Query> {
        rkyv::from_bytes::<Input, rkyv::rancor::Error>(frame).ok()
    }

    fn encode(&self, response: Self::Response) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(&response)
            .map(|frame| frame.to_vec())
            .unwrap_or_default()
    }

    fn intend(&self, input: Self::Query) -> Self::Operation {
        input
    }

    fn answer(&self, output: Self::Outcome) -> Self::Response {
        output
    }

    fn undecodable(&self) -> Self::Response {
        Output::Rejected("Curriculum received an invalid typed frame".into())
    }
}

pub struct CurriculumNexus;

impl Nexus for CurriculumNexus {
    type Memory = SkillMemory;
    type Operation = CurriculumOperation;
    type Signal = CurriculumSignal;

    fn signal() -> Self::Signal {
        CurriculumSignal
    }

    fn operation() -> Self::Operation {
        CurriculumOperation
    }

    fn directory() -> PathBuf {
        std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/run"))
            .join("curriculum")
    }

    fn socket_path() -> PathBuf {
        Self::directory().join("curriculum.sock")
    }
}

#[cfg(test)]
mod tests {
    use signal_curriculum_nexus::{Input, ModuleIdentity, Output};

    use super::CurriculumSignal;
    use nexus::Signaling;

    #[test]
    fn signal_decodes_and_answers_only_archived_types() {
        let input = Input::ResolveSkills(vec![ModuleIdentity("knowledge:main-flow:0".into())]);
        let encoded = rkyv::to_bytes::<rkyv::rancor::Error>(&input).expect("archive input");
        assert_eq!(CurriculumSignal.decode(&encoded), Some(input));
        assert_eq!(
            CurriculumSignal.answer(Output::ResolvedSkills(vec![ModuleIdentity(
                "spirit:core:0".into()
            )])),
            Output::ResolvedSkills(vec![ModuleIdentity("spirit:core:0".into())])
        );
    }
}
