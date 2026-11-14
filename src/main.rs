use std::process::ExitCode;

use curriculum::client::{
    ConstructsCurriculumClient, CurriculumClient, PresentsOutput, RunsClient,
};
use signal_curriculum_nexus::Output;

fn main() -> ExitCode {
    let client = CurriculumClient::from_arguments(std::env::args().skip(1));
    match client.run() {
        Ok(Output::Rejected(error)) => {
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
