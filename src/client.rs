use std::{
    io::{self, Read, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
};

use datom_codec::{Actualizing, Budget, Composing, Datomizable, Potential};
use protos::{Protosizable, ReaderBudget, Textualizable};
use thiserror::Error;

use signal_curriculum_nexus::{Input, Output, RoleSelection, Surface};

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("curriculum accepts exactly one inline Datom request")]
    Arguments,
    #[error("request parsing failed: {0}")]
    Datom(String),
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

pub trait NamesRequestedRole {
    /// The role named by a single `ComposeRole` request for a Claude agent.
    fn requested_claude_role(&self) -> Option<String>;
}

impl NamesRequestedRole for CurriculumClient {
    fn requested_claude_role(&self) -> Option<String> {
        let [argument] = self.arguments.as_slice() else {
            return None;
        };
        match actualize_request(argument).ok()? {
            Input::ComposeRole(RoleSelection {
                role_name,
                surface: Surface::ClaudeAgent,
            }) => Some(role_name.0),
            _ => None,
        }
    }
}

pub trait RunsClient {
    fn run(&self) -> Result<Output, ClientError>;
}

impl RunsClient for CurriculumClient {
    fn run(&self) -> Result<Output, ClientError> {
        let [argument] = self.arguments.as_slice() else {
            return Err(ClientError::Arguments);
        };
        if argument.starts_with('-') {
            return Err(ClientError::Arguments);
        }
        let input = actualize_request(argument)?;
        let request = rkyv::to_bytes::<rkyv::rancor::Error>(&input)
            .map_err(|error| ClientError::Encode(error.to_string()))?;
        let socket = socket_path();
        let mut stream = UnixStream::connect(&socket).map_err(|source| ClientError::Connect {
            path: socket,
            source,
        })?;
        write_frame(&mut stream, &request)?;
        let response = read_frame(&mut stream)?;
        rkyv::from_bytes::<Output, rkyv::rancor::Error>(&response).map_err(|_| ClientError::Decode)
    }
}

pub trait PresentsOutput {
    fn datom_text(&self) -> String;
}

impl PresentsOutput for Output {
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
            reader: ReaderBudget::try_from(usize::try_from(limit).expect("positive fixed budget"))
                .expect("a constant reader budget"),
            depth: 0,
            maximum_depth: limit,
        })
    }
}

fn actualize_request(text: &str) -> Result<Input, ClientError> {
    Potential::<Input>::from(text)
        .actualize_with_limit(1_048_576)
        .map_err(DatomFaulting::datom_fault)
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
