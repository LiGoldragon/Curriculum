//! Classifies the exact source bodies a publisher is preparing to export.
//! There is no wire or filesystem authority here: framing is local stdin.
use std::{
    io::{self, Read, Write},
    process::ExitCode,
};

use curriculum::{SkillVisibility, module_placement};

fn classify(mut input: impl Read, mut output: impl Write) -> Result<(), String> {
    let mut frames = String::new();
    input
        .read_to_string(&mut frames)
        .map_err(|error| error.to_string())?;
    if !frames.is_empty() && !frames.ends_with('\0') {
        return Err("skill inventory needs NUL-terminated module bodies".into());
    }
    let classifications = frames
        .split_terminator('\0')
        .map(|source| {
            module_placement(source).map(|placement| {
                if placement.is_public_skill() {
                    "Public"
                } else {
                    "Specialist"
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    for classification in classifications {
        writeln!(output, "{classification}").map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn main() -> ExitCode {
    if std::env::args_os().len() != 1 {
        eprintln!("skill inventory accepts module bodies on standard input only");
        return ExitCode::FAILURE;
    }
    match classify(io::stdin().lock(), io::stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
