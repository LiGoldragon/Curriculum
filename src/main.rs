use std::process::ExitCode;

use dotos::DotosEncode;
use skills::CommandLine;

fn main() -> ExitCode {
    match CommandLine::from_environment().run() {
        Ok(output) => {
            println!("{}", output.to_dotos());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("skills: {error}");
            ExitCode::FAILURE
        }
    }
}
