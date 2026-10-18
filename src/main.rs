use std::process::ExitCode;

use curriculum::{
    claude_agent::RendersClaudeBook,
    client::{ConstructsCurriculumClient, CurriculumClient, PresentsOutcome, RunsClient},
    generated::Outcome,
};

fn main() -> ExitCode {
    let mut arguments: Vec<String> = std::env::args().skip(1).collect();
    let render_book = arguments.first().is_some_and(|a| a == "--claude-agent-markdown");
    if render_book {
        arguments.remove(0);
        if arguments.as_slice() != ["ComposeRole.{ book ClaudeAgent }"] {
            eprintln!("expected --claude-agent-markdown 'ComposeRole.{{ book ClaudeAgent }}'");
            return ExitCode::FAILURE;
        }
    }
    match CurriculumClient::from_arguments(arguments).run() {
        Ok(Outcome::Rejected(error)) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
        Ok(Outcome::RoleComposed(role)) if render_book => {
            match role.claude_book_markdown() {
                Ok(markdown) => {
                    print!("{markdown}");
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::FAILURE
                }
            }
        }
        Ok(_) if render_book => {
            eprintln!("Curriculum Nexus did not return a composed book role");
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
