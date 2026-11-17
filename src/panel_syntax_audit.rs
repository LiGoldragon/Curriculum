//! Branch-only corpus probe: current declared source bytes through MiniJinja.
use curriculum::{ExpandsSkills, SkillRegistry};
use minijinja::{Environment, UndefinedBehavior};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let roots: [PathBuf; 3] = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| "provide the three authored source roots")?;
    let registry = SkillRegistry::from_context_repositories(
        &roots,
        &["0".repeat(40), "0".repeat(40), "0".repeat(40)],
    )?;
    let mut environment = Environment::new();
    environment.set_keep_trailing_newline(true);
    environment.set_undefined_behavior(UndefinedBehavior::Strict);
    let mut total = 0;
    let mut changed = 0;
    let mut rejected = 0;
    for module in registry.actual_skills() {
        total += 1;
        let digest = format!("{:x}", Sha256::digest(module.body.as_bytes()));
        println!("Source\t{}\t{digest}", module.name);
        match environment.render_str(&module.body, ()) {
            Ok(rendered) if rendered == module.body => {
                println!("Unchanged\t{}\t{}", module.name, module.path.display())
            }
            Ok(_) => {
                changed += 1;
                println!("Changed\t{}\t{}", module.name, module.path.display());
            }
            Err(error) => {
                rejected += 1;
                println!(
                    "Rejected\t{}\t{}\t{error}",
                    module.name,
                    module.path.display()
                );
            }
        }
    }
    println!("Summary\ttotal={total}\tchanged={changed}\trejected={rejected}");
    Ok(())
}
