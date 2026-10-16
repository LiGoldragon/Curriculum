use std::{future::Future, path::PathBuf};
use nexus::{Admission, Changed, MemoryHandle, Nexus, Operating, Remembering, Signaling};
use crate::{generated::{Outcome, Query, Roles}, registry::{ExpandsSkills, SkillRegistry, SourceConfiguration}};

#[derive(Clone)]
pub struct SkillSnapshot { registry: SkillRegistry, roles: Option<Roles> }
pub struct SkillMemory { registry: SkillRegistry, roles: Option<Roles> }
impl Remembering for SkillMemory {
    type Change = ();
    type Reading = ();
    type Remembered = SkillSnapshot;
    fn open(admission: Admission) -> Option<Self> {
        if admission.directory() != CurriculumNexus::directory() { return None; }
        let registry = SourceConfiguration::from_environment().and_then(|c|c.read().map_err(|e|e.to_string())).map_err(|e|eprintln!("{e}")).ok()?;
        let roles = crate::roles::read_declared().map_err(|e|eprintln!("{e}")).ok()?;
        Some(Self { registry, roles })
    }
    fn change(&mut self, _: ()) -> Changed { Changed::Failed }
    fn read(&self, _: ()) -> SkillSnapshot { SkillSnapshot { registry:self.registry.clone(), roles:self.roles.clone() } }
}
pub struct CurriculumOperation;
impl Operating<SkillMemory> for CurriculumOperation {
    type Operation = Query;
    type Outcome = Outcome;
    fn perform(&mut self, query: Query, memory: &MemoryHandle<SkillMemory>) -> impl Future<Output=Outcome> + Send {
        async move {
            let Some(snapshot) = memory.read(()).await else { return Outcome::Rejected("Curriculum memory unavailable".into()) };
            match query {
                Query::ResolveSkills(ids) => match snapshot.registry.expand_skills(&ids) {
                    Ok(s) => Outcome::ResolvedSkills(s.into_iter().map(|m|m.name).collect()), Err(e) => Outcome::Rejected(e.to_string())
                },
                Query::ResolveSources(ids) => match snapshot.registry.resolved_sources(&ids) {
                    Ok(s) => Outcome::ResolvedSources(s), Err(e) => Outcome::Rejected(e.to_string())
                },
                Query::ComposeStartup(selection) => match crate::startup::compose(&snapshot.registry,&selection) {
                    Ok(s) => Outcome::StartupComposed(s), Err(e) => Outcome::Rejected(e)
                },
                Query::ComposeRole(selection) => match snapshot.roles {
                    Some(roles) => match crate::roles::compose(&snapshot.registry,&roles,&selection) {
                        Ok(s) => Outcome::RoleComposed(s), Err(e) => Outcome::Rejected(e)
                    },
                    None => Outcome::Rejected("no server-declared ID-only roles metadata".into())
                }
            }
        }
    }
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
        let query = Query::ResolveSkills(vec!["knowledge-main-flow".into()]);
        let encoded = rkyv::to_bytes::<rkyv::rancor::Error>(&query).expect("archive query");
        assert_eq!(CurriculumSignal.decode(&encoded), Some(query));
        assert_eq!(
            CurriculumSignal.answer(Outcome::ResolvedSkills(vec!["spirit-core".into()])),
            Outcome::ResolvedSkills(vec!["spirit-core".into()])
        );
    }
}
