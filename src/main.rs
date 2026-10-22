use std::process::ExitCode;

use curriculum::{
    claude_agent::RendersClaudeAgent,
    client::{ConstructsCurriculumClient, CurriculumClient, PresentsOutcome, RunsClient},
    generated::Outcome,
};

fn main() -> ExitCode {
    let mut arguments: Vec<String> = std::env::args().skip(1).collect();
    let render_agent = arguments
        .first()
        .is_some_and(|a| a == "--claude-agent-markdown");
    let mut requested_role = None;
    if render_agent {
        arguments.remove(0);
        requested_role = match arguments.as_slice() {
            [request] if request == "ComposeRole.{ book ClaudeAgent }" => Some("book"),
            [request] if request == "ComposeRole.{ general-purpose ClaudeAgent }" => {
                Some("general-purpose")
            }
            [request] if request == "ComposeRole.{ book-author ClaudeAgent }" => {
                Some("book-author")
            }
            [request] if request == "ComposeRole.{ illustrator ClaudeAgent }" => {
                Some("illustrator")
            }
            _ => {
                eprintln!(
                    "expected --claude-agent-markdown with an exact supported ClaudeAgent ComposeRole request"
                );
                return ExitCode::FAILURE;
            }
        };
    }

    match CurriculumClient::from_arguments(arguments).run() {
        Ok(Outcome::Rejected(error)) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
        Ok(Outcome::RoleComposed(role))
            if render_agent && requested_role != Some(role.role_name.as_str()) =>
        {
            eprintln!("Curriculum Nexus returned a different role from the requested role");
            ExitCode::FAILURE
        }
        Ok(Outcome::RoleComposed(role)) if render_agent => match role.claude_agent_markdown() {
            Ok(markdown) => {
                print!("{markdown}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        },
        Ok(_) if render_agent => {
            eprintln!("Curriculum Nexus did not return a composed role");
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
