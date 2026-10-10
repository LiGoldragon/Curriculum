use std::{future::Future, path::PathBuf};

use nexus::{Admission, Changed, MemoryHandle, Nexus, Operating, Remembering, Signaling};

use crate::{
    generated::{Outcome, Query, RolePlan, SkillChange, SkillEdits, SkillPathEdits},
    projection::{ConstructsSkillProjector, ProjectsSkills, SkillProjector},
    registry::{
        AppliesSkillPathEdits, ComparesSkillRegistries, ExpandsSkills, ReadsSkillRepositories,
        RegistryError, ResolvedSkill, SkillRegistry,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct Settings {
    psyche_repository: PathBuf,
    mind_skills: PathBuf,
    field_skills: PathBuf,
    workspace: PathBuf,
    runtime: PathBuf,
}

trait ReadsCurriculumSettings {
    fn from_environment() -> Result<Settings, String>;
}

impl ReadsCurriculumSettings for Settings {
    fn from_environment() -> Result<Settings, String> {
        let path = |name: &str| {
            std::env::var_os(name)
                .map(PathBuf::from)
                .ok_or_else(|| format!("missing environment setting {name}"))
        };
        Ok(Settings {
            psyche_repository: path("CURRICULUM_PSYCHES_REPOSITORY_DIR")?,
            mind_skills: path("CURRICULUM_MIND_SKILLS_DIR")?,
            field_skills: path("CURRICULUM_FIELD_SKILLS_DIR")?,
            workspace: path("CURRICULUM_WORKSPACE")?,
            runtime: path("XDG_RUNTIME_DIR")?.join("curriculum"),
        })
    }
}

#[derive(Clone)]
pub struct SkillSnapshot {
    registry: SkillRegistry,
    settings: Settings,
    projector: SkillProjector,
    last_error: Option<String>,
    last_change: Option<SkillChange>,
}

pub struct SkillMemory {
    registry: SkillRegistry,
    settings: Settings,
    projector: SkillProjector,
    last_error: Option<String>,
    last_change: Option<SkillChange>,
}

impl Remembering for SkillMemory {
    type Change = SkillEdits;
    type Reading = ();
    type Remembered = SkillSnapshot;

    fn open(admission: Admission) -> Option<Self> {
        let settings = Settings::from_environment()
            .map_err(|error| eprintln!("{error}"))
            .ok()?;
        if admission.directory() != settings.runtime {
            return None;
        }
        let registry = SkillRegistry::from_repositories(
            &settings.psyche_repository,
            &settings.mind_skills,
            &settings.field_skills,
        )
        .map_err(|error| eprintln!("{error}"))
        .ok()?;
        let projector = SkillProjector::at(settings.workspace.clone());
        Some(Self {
            registry,
            settings,
            projector,
            last_error: None,
            last_change: None,
        })
    }

    fn change(&mut self, edits: Self::Change) -> Changed {
        let mut next = self.registry.clone();
        let change = match next.apply_paths(
            &edits.skill_path_edits.first_string_vector,
            &edits.skill_path_edits.second_string_vector,
            &edits.skill_path_edits.third_string_vector,
        ) {
            Ok(change) => change,
            Err(error) => {
                self.last_error = Some(error.to_string());
                self.last_change = None;
                return Changed::Failed;
            }
        };
        let skills = match all_skills(&next) {
            Ok(skills) => skills,
            Err(error) => {
                self.last_error = Some(error.to_string());
                self.last_change = None;
                return Changed::Failed;
            }
        };
        if let Err(error) = self.projector.write(&skills, &edits.role_plan) {
            self.last_error = Some(error.to_string());
            self.last_change = None;
            return Changed::Failed;
        }
        self.registry = next;
        self.last_error = None;
        self.last_change = Some(change);
        Changed::Succeeded
    }

    fn read(&self, _: Self::Reading) -> Self::Remembered {
        SkillSnapshot {
            registry: self.registry.clone(),
            settings: self.settings.clone(),
            projector: self.projector.clone(),
            last_error: self.last_error.clone(),
            last_change: self.last_change.clone(),
        }
    }
}

pub struct CurriculumOperation;

impl Operating<SkillMemory> for CurriculumOperation {
    type Operation = Query;
    type Outcome = Outcome;

    fn perform(
        &mut self,
        query: Self::Operation,
        memory: &MemoryHandle<SkillMemory>,
    ) -> impl Future<Output = Self::Outcome> + Send {
        async move {
            match query {
                Query::ResolveSkills(roots) => match memory.read(()).await {
                    Some(snapshot) => match snapshot.registry.expand_skills(&roots) {
                        Ok(skills) => Outcome::ResolvedSkills(
                            skills.into_iter().map(|skill| skill.name).collect(),
                        ),
                        Err(error) => Outcome::Rejected(error.to_string()),
                    },
                    None => Outcome::Rejected("Curriculum memory is unavailable".into()),
                },
                Query::EditSkills(edits) => edit_skills(edits, memory).await,
                Query::CheckSkills(role_plan) => check_skills(role_plan, memory).await,
                Query::RebuildSkills(role_plan) => rebuild_skills(role_plan, memory).await,
            }
        }
    }
}

async fn check_skills(role_plan: RolePlan, memory: &MemoryHandle<SkillMemory>) -> Outcome {
    let Some(snapshot) = memory.read(()).await else {
        return Outcome::Rejected("Curriculum memory is unavailable".into());
    };
    let current = match read_current_sources(&snapshot) {
        Ok(registry) => registry,
        Err(error) => return Outcome::Rejected(error),
    };
    let skills = match all_skills(&current) {
        Ok(skills) => skills,
        Err(error) => return Outcome::Rejected(error.to_string()),
    };
    match snapshot.projector.check(&skills, &role_plan) {
        Ok(()) => Outcome::SkillsChecked(skills.into_iter().map(|skill| skill.name).collect()),
        Err(error) => Outcome::Rejected(error.to_string()),
    }
}

async fn rebuild_skills(role_plan: RolePlan, memory: &MemoryHandle<SkillMemory>) -> Outcome {
    let Some(snapshot) = memory.read(()).await else {
        return Outcome::Rejected("Curriculum memory is unavailable".into());
    };
    if let Err(error) = read_current_sources(&snapshot) {
        return Outcome::Rejected(error);
    }
    let edits = SkillEdits {
        skill_path_edits: SkillPathEdits {
            first_string_vector: Vec::new(),
            second_string_vector: Vec::new(),
            third_string_vector: Vec::new(),
        },
        role_plan,
    };
    if memory.change(edits).await == Changed::Failed {
        return memory
            .read(())
            .await
            .and_then(|snapshot| snapshot.last_error)
            .map(Outcome::Rejected)
            .unwrap_or_else(|| Outcome::Rejected("Curriculum rebuild failed".into()));
    }
    match memory.read(()).await {
        Some(snapshot) => match snapshot.registry.skill_names() {
            Ok(names) => Outcome::SkillsRebuilt(names),
            Err(error) => Outcome::Rejected(error.to_string()),
        },
        None => Outcome::Rejected("Curriculum memory is unavailable".into()),
    }
}

fn read_current_sources(snapshot: &SkillSnapshot) -> Result<SkillRegistry, String> {
    let current = SkillRegistry::from_repositories(
        &snapshot.settings.psyche_repository,
        &snapshot.settings.mind_skills,
        &snapshot.settings.field_skills,
    )
    .map_err(|error| error.to_string())?;
    let changes = snapshot
        .registry
        .changes_to(&current)
        .map_err(|error| error.to_string())?;
    if !changes.first_string_vector.is_empty()
        || !changes.second_string_vector.is_empty()
        || !changes.third_string_vector.is_empty()
    {
        return Err("authored skill sources changed without an EditSkills signal".into());
    }
    Ok(current)
}

async fn edit_skills(edits: SkillEdits, memory: &MemoryHandle<SkillMemory>) -> Outcome {
    if memory.change(edits).await == Changed::Failed {
        return memory
            .read(())
            .await
            .and_then(|snapshot| snapshot.last_error)
            .map(Outcome::Rejected)
            .unwrap_or_else(|| Outcome::Rejected("Curriculum rejected the edit".into()));
    }
    let Some(snapshot) = memory.read(()).await else {
        return Outcome::Rejected("Curriculum memory is unavailable".into());
    };
    Outcome::SkillsChanged(snapshot.last_change.unwrap_or(SkillChange {
        first_string_vector: Vec::new(),
        second_string_vector: Vec::new(),
        third_string_vector: Vec::new(),
    }))
}

fn all_skills(registry: &SkillRegistry) -> Result<Vec<ResolvedSkill>, RegistryError> {
    registry.expand_skills(&registry.skill_names()?)
}

pub struct CurriculumSignal;

impl Signaling for CurriculumSignal {
    type Query = Query;
    type Response = Outcome;
    type Operation = Query;
    type Outcome = Outcome;

    fn decode(&self, frame: &[u8]) -> Option<Self::Query> {
        rkyv::from_bytes::<Query, rkyv::rancor::Error>(frame).ok()
    }

    fn encode(&self, response: Self::Response) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(&response)
            .map(|frame| frame.to_vec())
            .unwrap_or_default()
    }

    fn intend(&self, query: Self::Query) -> Self::Operation {
        query
    }

    fn answer(&self, outcome: Self::Outcome) -> Self::Response {
        outcome
    }

    fn undecodable(&self) -> Self::Response {
        Outcome::Rejected("Curriculum received an invalid typed frame".into())
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
    use crate::generated::{Outcome, Query};

    use super::CurriculumSignal;
    use nexus::Signaling;

    #[test]
    fn signal_decodes_and_answers_only_archived_types() {
        let query = Query::ResolveSkills(vec!["operation-main-flow".into()]);
        let encoded = rkyv::to_bytes::<rkyv::rancor::Error>(&query).expect("archive query");
        assert_eq!(CurriculumSignal.decode(&encoded), Some(query));
        assert_eq!(
            CurriculumSignal.answer(Outcome::ResolvedSkills(vec!["spirit".into()])),
            Outcome::ResolvedSkills(vec!["spirit".into()])
        );
    }
}
