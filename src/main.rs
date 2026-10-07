use std::process::ExitCode;

use curriculum::{
    client::{ConstructsCurriculumClient, CurriculumClient, PresentsOutcome, RunsClient},
    generated::Outcome,
};

fn main() -> ExitCode {
    match CurriculumClient::from_arguments(std::env::args().skip(1)).run() {
        Ok(Outcome::Rejected(error)) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
        Ok(output) => {
            println!("{}", output.datom_text());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
