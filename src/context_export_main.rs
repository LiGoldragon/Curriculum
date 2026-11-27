//! Offline export of exactly the files admitted by three typed context sources.
use curriculum::{SkillRegistry, registry::IdentityText};
use std::{
    collections::BTreeSet,
    env, fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
    process::ExitCode,
};

struct Source {
    name: String,
    root: PathBuf,
    revision: String,
}

struct CopyPlan {
    identity: String,
    revision: String,
    source: PathBuf,
    source_relative: PathBuf,
    relative_destination: PathBuf,
}

fn simple_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    let alphanumeric = |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
    alphanumeric(first)
        && value
            .as_bytes()
            .last()
            .is_some_and(|byte| alphanumeric(*byte))
        && bytes.all(|byte| alphanumeric(byte) || byte == b'-')
}

fn absent(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(format!("output already exists: {}", path.display())),
        Err(error) => Err(format!("inspect output {}: {error}", path.display())),
    }
}

fn output_path(value: &str) -> Result<PathBuf, String> {
    let requested = PathBuf::from(value);
    let file_name = requested
        .file_name()
        .ok_or_else(|| "output needs a final path component".to_owned())?;
    let parent = requested
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let canonical_parent = fs::canonicalize(parent)
        .map_err(|error| format!("canonicalize output parent {}: {error}", parent.display()))?;
    let metadata = fs::symlink_metadata(&canonical_parent).map_err(|error| {
        format!(
            "inspect output parent {}: {error}",
            canonical_parent.display()
        )
    })?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(format!(
            "output parent is not a plain directory: {}",
            canonical_parent.display()
        ));
    }
    Ok(canonical_parent.join(file_name))
}

fn parse(arguments: impl IntoIterator<Item = String>) -> Result<(PathBuf, [Source; 3]), String> {
    let arguments: Vec<_> = arguments.into_iter().collect();
    let [
        output,
        first_name,
        first_root,
        first_revision,
        second_name,
        second_root,
        second_revision,
        third_name,
        third_root,
        third_revision,
    ] = arguments.as_slice()
    else {
        return Err(
            "usage: curriculum-context-export OUTPUT NAME ROOT REVISION NAME ROOT REVISION NAME ROOT REVISION"
                .into(),
        );
    };
    let output = output_path(output)?;
    absent(&output)?;
    let triples = [
        (first_name, first_root, first_revision),
        (second_name, second_root, second_revision),
        (third_name, third_root, third_revision),
    ];
    let mut names = BTreeSet::new();
    let mut sources = Vec::new();
    for (name, root, revision) in triples {
        if !simple_name(name) || !names.insert(name.clone()) {
            return Err(format!("invalid or duplicate source name: {name}"));
        }
        sources.push(Source {
            name: name.clone(),
            root: PathBuf::from(root),
            revision: revision.clone(),
        });
    }
    Ok((output, sources.try_into().expect("three sources")))
}

fn plan(sources: &[Source; 3]) -> Result<Vec<CopyPlan>, String> {
    let roots = sources.each_ref().map(|source| source.root.clone());
    let revisions = sources.each_ref().map(|source| source.revision.clone());
    let registry = SkillRegistry::from_context_repositories(&roots, &revisions)
        .map_err(|error| error.to_string())?;
    if let Some(report) = registry.reports().first() {
        return Err(format!("declaration was not admitted: {report}"));
    }
    let canonical_roots = [
        fs::canonicalize(&roots[0]).map_err(|error| error.to_string())?,
        fs::canonicalize(&roots[1]).map_err(|error| error.to_string())?,
        fs::canonicalize(&roots[2]).map_err(|error| error.to_string())?,
    ];
    let mut destinations = BTreeSet::new();
    let mut plans: Vec<CopyPlan> = registry
        .listed_modules()
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|module| {
            let identity = module
                .module_identity
                .identity_text()
                .map_err(|error| format!("invalid admitted identity: {error}"))?;
            let source = PathBuf::from(module.module_file.0);
            let associations: Vec<_> = canonical_roots
                .iter()
                .enumerate()
                .filter(|(_, root)| source.starts_with(root))
                .collect();
            let [(index, root)] = associations.as_slice() else {
                return Err(format!(
                    "admitted file does not belong uniquely to one source: {}",
                    source.display()
                ));
            };
            let relative = source
                .strip_prefix(root)
                .map_err(|_| format!("admitted file escaped its source: {}", source.display()))?;
            if relative.as_os_str().is_empty()
                || relative
                    .components()
                    .any(|component| !matches!(component, Component::Normal(_)))
            {
                return Err(format!(
                    "unsafe admitted relative path: {}",
                    relative.display()
                ));
            }
            let relative_destination = Path::new(&sources[*index].name).join(relative);
            let relative_text = relative_destination.to_str().ok_or_else(|| {
                format!(
                    "admitted path is not UTF-8: {}",
                    relative_destination.display()
                )
            })?;
            if relative_text.chars().any(char::is_whitespace) {
                return Err(format!(
                    "admitted path cannot enter the whitespace-delimited inventory: {}",
                    relative_destination.display()
                ));
            }
            if !destinations.insert(relative_destination.clone()) {
                return Err(format!(
                    "duplicate export destination: {}",
                    relative_destination.display()
                ));
            }
            Ok(CopyPlan {
                identity,
                revision: sources[*index].revision.clone(),
                source,
                source_relative: relative.to_owned(),
                relative_destination,
            })
        })
        .collect::<Result<_, _>>()?;
    plans.sort_by(|left, right| left.identity.cmp(&right.identity));
    for plan in &plans {
        if plans
            .iter()
            .filter(|candidate| {
                candidate.revision == plan.revision
                    && candidate.source_relative == plan.source_relative
            })
            .count()
            != 1
        {
            return Err(format!(
                "inventory row does not identify one filtered source: {} {}",
                plan.revision,
                plan.source_relative.display()
            ));
        }
    }
    Ok(plans)
}

fn scratch(output: &Path) -> Result<PathBuf, String> {
    let parent = output.parent().expect("canonical output has a parent");
    for serial in 0..1024 {
        let candidate = parent.join(format!(
            ".curriculum-context-export-{}-{serial}",
            std::process::id()
        ));
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => return Err(format!("create export scratch: {error}")),
        }
    }
    Err("no unused export scratch name".into())
}

fn export(output: &Path, plans: &[CopyPlan]) -> Result<(), String> {
    let scratch = scratch(output)?;
    let result = (|| {
        for plan in plans {
            let destination = scratch.join(&plan.relative_destination);
            fs::create_dir_all(destination.parent().expect("file destination has a parent"))
                .map_err(|error| format!("create export directory: {error}"))?;
            fs::copy(&plan.source, &destination).map_err(|error| {
                format!(
                    "copy admitted file {} to {}: {error}",
                    plan.source.display(),
                    destination.display()
                )
            })?;
        }
        let inventory = plans
            .iter()
            .map(|plan| {
                format!(
                    "{} {} {}\n",
                    plan.identity,
                    plan.revision,
                    plan.source_relative.to_string_lossy()
                )
            })
            .collect::<String>();
        fs::write(scratch.join("modules"), inventory)
            .map_err(|error| format!("write admitted module inventory: {error}"))?;
        absent(output)?;
        fs::rename(&scratch, output).map_err(|error| format!("publish export: {error}"))
    })();
    match result {
        Ok(()) => Ok(()),
        Err(error) => match fs::remove_dir_all(&scratch) {
            Ok(()) => Err(error),
            Err(cleanup) => Err(format!(
                "{error}; remove partial export {}: {cleanup}",
                scratch.display()
            )),
        },
    }
}

fn run(arguments: impl IntoIterator<Item = String>) -> Result<(), String> {
    let (output, sources) = parse(arguments)?;
    let plan = plan(&sources)?;
    for source in &sources {
        let root = fs::canonicalize(&source.root).map_err(|error| error.to_string())?;
        if output.starts_with(&root) || root.starts_with(&output) {
            return Err(format!(
                "output overlaps a context source: {}",
                root.display()
            ));
        }
    }
    export(&output, &plan)
}

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
