use std::{
    fs,
    io::{self, Read, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
};

use datom_codec::{Actualizing, Budget, Composing, Datomizable, Potential};
use protos::{Protosizable, ReaderBudget, Textualizable};
use thiserror::Error;

use crate::{
    generated::{
        CliRequest, Outcome, Query, RoleManifest, RoleManifestDocument, RolePlan, RolesDocument,
        SkillEdits,
    },
    roles::PlansRoles,
};

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("curriculum accepts exactly one inline Datom request")]
    Arguments,
    #[error("request parsing failed: {0}")]
    Datom(String),
    #[error("role configuration failed: {0}")]
    Roles(String),
    #[error("cannot connect to Curriculum Nexus at {path}: {source}")]
    Connect {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Curriculum Nexus frame failed: {0}")]
    Frame(#[from] io::Error),
    #[error("cannot encode Curriculum request: {0}")]
    Encode(String),
    #[error("Curriculum Nexus returned an invalid response")]
    Decode,
}

pub struct CurriculumClient {
    arguments: Vec<String>,
}

pub trait ConstructsCurriculumClient {
    fn from_arguments(arguments: impl IntoIterator<Item = String>) -> Self;
}

impl ConstructsCurriculumClient for CurriculumClient {
    fn from_arguments(arguments: impl IntoIterator<Item = String>) -> Self {
        Self {
            arguments: arguments.into_iter().collect(),
        }
    }
}

pub trait RunsClient {
    fn run(&self) -> Result<Outcome, ClientError>;
}

impl RunsClient for CurriculumClient {
    fn run(&self) -> Result<Outcome, ClientError> {
        let [argument] = self.arguments.as_slice() else {
            return Err(ClientError::Arguments);
        };
        if argument.starts_with('-') {
            return Err(ClientError::Arguments);
        }
        let request = actualize_request(argument)?;
        let query = prepare_query(request)?;
        let request = rkyv::to_bytes::<rkyv::rancor::Error>(&query)
            .map_err(|error| ClientError::Encode(error.to_string()))?;
        let socket = socket_path();
        let mut stream = UnixStream::connect(&socket).map_err(|source| ClientError::Connect {
            path: socket,
            source,
        })?;
        write_frame(&mut stream, &request)?;
        let response = read_frame(&mut stream)?;
        rkyv::from_bytes::<Outcome, rkyv::rancor::Error>(&response).map_err(|_| ClientError::Decode)
    }
}

pub trait PresentsOutcome {
    fn datom_text(&self) -> String;
}

impl PresentsOutcome for Outcome {
    fn datom_text(&self) -> String {
        self.datomize(vec![]).protosize().textualize()
    }
}

trait DatomFaulting {
    fn datom_fault(self) -> ClientError;
}

impl DatomFaulting for datom_codec::Error {
    fn datom_fault(self) -> ClientError {
        ClientError::Datom(self.datomize(vec![]).protosize().textualize())
    }
}

trait BudgetedActualizing<T> {
    fn actualize_with_limit(&mut self, limit: i64) -> Result<T, datom_codec::Error>;
}

impl<T: Composing> BudgetedActualizing<T> for Potential<T> {
    fn actualize_with_limit(&mut self, limit: i64) -> Result<T, datom_codec::Error> {
        self.actualize(&mut Budget {
            remaining: limit,
            reader: ReaderBudget {
                remaining: usize::try_from(limit).expect("positive fixed budget"),
            },
            depth: 0,
            maximum_depth: limit,
        })
    }
}

fn actualize_request(text: &str) -> Result<CliRequest, ClientError> {
    Potential::<CliRequest>::from(text)
        .actualize_with_limit(1_048_576)
        .map_err(DatomFaulting::datom_fault)
}

fn prepare_query(request: CliRequest) -> Result<Query, ClientError> {
    Ok(match request {
        CliRequest::ResolveSkills(roots) => Query::ResolveSkills(roots),
        CliRequest::EditSkills(edits) => Query::EditSkills(SkillEdits {
            skill_path_edits: edits,
            role_plan: current_role_plan()?,
        }),
        CliRequest::CheckSkills => Query::CheckSkills(current_role_plan()?),
        CliRequest::RebuildSkills => Query::RebuildSkills(current_role_plan()?),
    })
}

fn current_role_plan() -> Result<RolePlan, ClientError> {
    let roles_path = std::env::var_os("CURRICULUM_ROLES_FILE")
        .map(PathBuf::from)
        .ok_or_else(|| ClientError::Roles("missing CURRICULUM_ROLES_FILE".into()))?;
    let workspace = std::env::var_os("CURRICULUM_WORKSPACE")
        .map(PathBuf::from)
        .ok_or_else(|| ClientError::Roles("missing CURRICULUM_WORKSPACE".into()))?;
    let source = fs::read_to_string(&roles_path)
        .map_err(|error| ClientError::Roles(format!("read {}: {error}", roles_path.display())))?;
    let RolesDocument::Roles(roles) = Potential::<RolesDocument>::from(source)
        .actualize_with_limit(16_384)
        .map_err(DatomFaulting::datom_fault)?;
    let mut plan = roles.role_plan(&workspace).map_err(ClientError::Roles)?;
    let manifest_path = workspace.join("skills/generated-role-outputs.datom");
    if manifest_path.exists() {
        let text = fs::read_to_string(&manifest_path).map_err(|error| {
            ClientError::Roles(format!("read {}: {error}", manifest_path.display()))
        })?;
        let RoleManifestDocument::GeneratedRoleOutputs(previous) =
            Potential::<RoleManifestDocument>::from(text)
                .actualize_with_limit(16_384)
                .map_err(DatomFaulting::datom_fault)?;
        plan.second_string_vector = previous.string_vector;
    }
    let paths = plan
        .role_packet_plan_vector
        .iter()
        .map(|packet| packet.first_string.clone())
        .collect();
    plan.string = datom_text(&RoleManifestDocument::GeneratedRoleOutputs(RoleManifest {
        string_vector: paths,
    }));
    Ok(plan)
}

fn datom_text<T: Datomizable>(value: &T) -> String {
    value.datomize(vec![]).protosize().textualize()
}

fn socket_path() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join("curriculum")
        .join("curriculum.sock")
}

fn write_frame(stream: &mut UnixStream, frame: &[u8]) -> io::Result<()> {
    let length = u32::try_from(frame.len()).map_err(|_| io::Error::other("frame too large"))?;
    stream.write_all(&length.to_be_bytes())?;
    stream.write_all(frame)?;
    stream.flush()
}

fn read_frame(stream: &mut UnixStream) -> io::Result<Vec<u8>> {
    let mut length = [0; 4];
    stream.read_exact(&mut length)?;
    let mut frame = vec![0; u32::from_be_bytes(length) as usize];
    stream.read_exact(&mut frame)?;
    Ok(frame)
}
