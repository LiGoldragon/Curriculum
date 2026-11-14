//! The refresh emission is issued as a child process: the program runs to its
//! end and its exit is the outcome Curriculum acts on.
use crate::emission::{Emittable, RefreshOutcome, RefreshProgram};
use std::process::Command;

/// The translator that turns an emission into a child process and observes
/// how it ended.
pub struct ProcessLauncher;

impl Emittable<ProcessLauncher> for RefreshProgram {
    fn emit(&self, _: &ProcessLauncher) -> RefreshOutcome {
        match Command::new(&self.0).status() {
            Ok(status) if status.success() => RefreshOutcome::Refreshed,
            Ok(_) => RefreshOutcome::Failed,
            Err(error) => RefreshOutcome::Unstarted(error.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProcessLauncher;
    use crate::emission::{Emittable, RefreshOutcome, RefreshProgram};
    use std::{fs, os::unix::fs::PermissionsExt, path::Path};

    fn program(directory: &Path, name: &str, exit: u8) -> RefreshProgram {
        let path = directory.join(name);
        fs::write(&path, format!("#!/bin/sh\nexit {exit}\n")).expect("refresh program");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).expect("executable");
        RefreshProgram(path.to_string_lossy().into_owned())
    }

    #[test]
    fn the_outcome_is_what_the_program_did() {
        let directory = tempfile::tempdir().expect("temporary directory");
        assert_eq!(
            program(directory.path(), "succeeds", 0).emit(&ProcessLauncher),
            RefreshOutcome::Refreshed
        );
        assert_eq!(
            program(directory.path(), "fails", 1).emit(&ProcessLauncher),
            RefreshOutcome::Failed
        );
        let absent = RefreshProgram(
            directory
                .path()
                .join("absent")
                .to_string_lossy()
                .into_owned(),
        );
        assert!(matches!(
            absent.emit(&ProcessLauncher),
            RefreshOutcome::Unstarted(_)
        ));
    }
}
