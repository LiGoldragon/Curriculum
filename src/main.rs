use std::process::ExitCode;

use curriculum::{
    claude_agent::RendersClaudeAgent,
    client::{
        ConstructsCurriculumClient, CurriculumClient, NamesRequestedRole, PresentsOutput,
        RunsClient,
    },
};
use signal_curriculum_nexus::Output;

fn main() -> ExitCode {
    let mut arguments: Vec<String> = std::env::args().skip(1).collect();
    let render_agent = arguments
        .first()
        .is_some_and(|a| a == "--claude-agent-markdown");
    if render_agent {
        arguments.remove(0);
    }
    let client = CurriculumClient::from_arguments(arguments);
    let requested_role = render_agent
        .then(|| client.requested_claude_role())
        .flatten();
    if render_agent && requested_role.is_none() {
        eprintln!("expected --claude-agent-markdown with one ClaudeAgent ComposeRole request");
        return ExitCode::FAILURE;
    }

    match client.run() {
        Ok(Output::Rejected(error)) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
        Ok(Output::RoleComposed(role))
            if render_agent && requested_role.as_deref() != Some(role.role_name.0.as_str()) =>
        {
            eprintln!("Curriculum Nexus returned a different role from the requested role");
            ExitCode::FAILURE
        }
        Ok(Output::RoleComposed(role)) if render_agent => match role.claude_agent_markdown() {
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
