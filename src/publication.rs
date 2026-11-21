//! Text edge for the receive command's local announcement. This process
//! admits an event; generation is a separate consumer action.
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use datom_codec::Potential;
use sha2::{Digest, Sha256};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
    time::timeout,
};

use crate::{
    client::{BudgetedActualizing, ClientError, DatomFaulting, PresentsOutput},
    publication_types::{
        Admission, AdmissionConfiguration, Announcement, Publication, PublicationDigest, Refusal,
    },
};

const TEXT_BUDGET: usize = 4096;
static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Parse(#[from] ClientError),
    #[error("publication admission needs absolute paths, a valid UID and a positive timeout")]
    Configuration,
    #[error("publication admission: {0}")]
    Io(#[from] io::Error),
}

struct Listener {
    socket: PathBuf,
    admissions: PathBuf,
    receiver_uid: u32,
    exchange_timeout: Duration,
}

impl TryFrom<AdmissionConfiguration> for Listener {
    type Error = Error;

    fn try_from(value: AdmissionConfiguration) -> Result<Self, Self::Error> {
        let socket = PathBuf::from(value.admission_socket.0);
        let admissions = PathBuf::from(value.admission_directory.0);
        let receiver_uid =
            u32::try_from(value.receiver_user_id.0).map_err(|_| Error::Configuration)?;
        let milliseconds = u64::try_from(value.admission_timeout_milliseconds.0)
            .map_err(|_| Error::Configuration)?;
        if !socket.is_absolute() || !admissions.is_absolute() || milliseconds == 0 {
            return Err(Error::Configuration);
        }
        Ok(Self {
            socket,
            admissions,
            receiver_uid,
            exchange_timeout: Duration::from_millis(milliseconds),
        })
    }
}

pub fn serve(arguments: impl IntoIterator<Item = String>) -> Result<(), Error> {
    let arguments: Vec<_> = arguments.into_iter().collect();
    let [argument] = arguments.as_slice() else {
        return Err(ClientError::Arguments.into());
    };
    let configuration = Potential::<AdmissionConfiguration>::from(argument.as_str())
        .actualize_with_limit(TEXT_BUDGET as i64)
        .map_err(DatomFaulting::datom_fault)?;
    let listener = Listener::try_from(configuration)?;
    // The deployment owns the socket parent and admission directory and
    // grants access only to the configured receive-command identity.
    if !listener.admissions.is_dir() {
        return Err(Error::Configuration);
    }
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(listener.run())
}

impl Listener {
    async fn run(self) -> Result<(), Error> {
        let listener = UnixListener::bind(&self.socket)?;
        loop {
            let (stream, _) = listener.accept().await?;
            let _ = timeout(self.exchange_timeout, self.exchange(stream)).await;
        }
    }

    async fn exchange(&self, mut stream: UnixStream) -> io::Result<()> {
        let admission = if stream
            .peer_cred()
            .is_ok_and(|peer| peer.uid() == self.receiver_uid)
        {
            let mut text = Vec::new();
            (&mut stream)
                .take((TEXT_BUDGET + 1) as u64)
                .read_to_end(&mut text)
                .await?;
            self.admit(&text)
        } else {
            Admission::Refused(Refusal::Untrusted)
        };
        let mut response = admission.datom_text();
        response.push('\n');
        stream.write_all(response.as_bytes()).await?;
        stream.shutdown().await
    }

    fn admit(&self, text: &[u8]) -> Admission {
        if text.len() > TEXT_BUDGET {
            return Admission::Refused(Refusal::Invalid);
        }
        let Some(Announcement::PublicationReceived(publication)) =
            std::str::from_utf8(text).ok().and_then(|text| {
                Potential::<Announcement>::from(text)
                    .actualize_with_limit(TEXT_BUDGET as i64)
                    .ok()
            })
        else {
            return Admission::Refused(Refusal::Invalid);
        };
        let Some(digest) = publication.digest() else {
            return Admission::Refused(Refusal::Invalid);
        };
        match record(&self.admissions, &publication, &digest) {
            Ok(()) => Admission::Accepted(digest),
            Err(_) => Admission::Refused(Refusal::Unavailable),
        }
    }
}

trait Identifiable {
    fn digest(&self) -> Option<PublicationDigest>;
}

impl Identifiable for Publication {
    fn digest(&self) -> Option<PublicationDigest> {
        let stream = &self.publication_stream;
        let repository = &stream.repository_name.0;
        let reference = &stream.reference_name.0;
        let old = &self.first_object_id.0;
        let new = &self.second_object_id.0;
        let valid_segment = |segment: &str| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        };
        let valid_id = |value: &str| {
            matches!(value.len(), 40 | 64)
                && value
                    .bytes()
                    .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
        };
        if !repository
            .split_once('/')
            .is_some_and(|(owner, name)| valid_segment(owner) && valid_segment(name))
            || reference != "refs/heads/main"
            || old.len() != new.len()
            || !valid_id(old)
            || !valid_id(new)
        {
            return None;
        }
        // The transport digest is over the already-landed flat canonical
        // record. The local event carries the typed stream as one value.
        let canonical = format!("{{ {repository} {reference} {old} {new} }}\n");
        Some(PublicationDigest(format!(
            "{:x}",
            Sha256::digest(canonical.as_bytes())
        )))
    }
}

fn record(
    directory: &Path,
    publication: &Publication,
    digest: &PublicationDigest,
) -> io::Result<()> {
    let target = directory.join(format!("{}.datom", digest.0));
    let serial = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
    let temporary = directory.join(format!(".admitting-{}-{serial}", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let bytes = publication.datom_text();
    let result = (|| {
        file.write_all(bytes.as_bytes())?;
        file.sync_all()?;
        match fs::hard_link(&temporary, &target) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                if fs::read(&target)? != bytes.as_bytes() {
                    return Err(io::Error::other("existing admission differs"));
                }
            }
            Err(error) => return Err(error),
        }
        File::open(directory)?.sync_all()
    })();
    let _ = fs::remove_file(temporary);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_the_digest_admits_once_and_refuses_an_invalid_publication() {
        let storage = tempfile::tempdir().unwrap();
        let listener = Listener {
            socket: storage.path().join("admissions.sock"),
            admissions: storage.path().to_owned(),
            receiver_uid: 0,
            exchange_timeout: Duration::from_secs(1),
        };
        let old = "a".repeat(40);
        let new = "b".repeat(40);
        let flat = format!("{{ sample/belief refs/heads/main {old} {new} }}\n");
        let digest = format!("{:x}", Sha256::digest(flat.as_bytes()));
        let event = |new: &str| {
            format!("PublicationReceived.{{ {{ sample/belief refs/heads/main }} {old} {new} }}\n")
        };
        assert!(matches!(
            listener.admit(event(&"z".repeat(40)).as_bytes()),
            Admission::Refused(Refusal::Invalid)
        ));
        assert_eq!(fs::read_dir(storage.path()).unwrap().count(), 0);
        let request = event(&new);
        let expected = format!("Accepted.{digest}");
        assert_eq!(listener.admit(request.as_bytes()).datom_text(), expected);
        let admitted = storage.path().join(format!("{digest}.datom"));
        let initial = fs::read(&admitted).unwrap();
        assert_eq!(listener.admit(request.as_bytes()).datom_text(), expected);
        assert_eq!(fs::read(admitted).unwrap(), initial);
        assert_eq!(fs::read_dir(storage.path()).unwrap().count(), 1);
    }
}
