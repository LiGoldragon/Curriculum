use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::generated::SkillChange;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillEntry {
    pub path: PathBuf,
    pub name: String,
    pub body: String,
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedSkill {
    pub path: PathBuf,
    pub name: String,
    pub body: String,
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SkillRegistry {
    entries: BTreeMap<PathBuf, SkillEntry>,
    psyche_repository_root: PathBuf,
    mind_skills_root: PathBuf,
    field_skills_root: PathBuf,
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("cannot read skill source {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("invalid skill source {path}: {message}")]
    InvalidSource { path: PathBuf, message: String },
    #[error("skill {name} is defined by both {first} and {second}")]
    DuplicateName {
        name: String,
        first: PathBuf,
        second: PathBuf,
    },
    #[error("missing skill dependency {name}")]
    Missing { name: String },
    #[error("skill dependency cycle: {chain}")]
    Cycle { chain: String },
    #[error("skill source path is outside configured repositories: {0}")]
    OutsideSource(PathBuf),
    #[error("a skill path occurs more than once in one edit: {0}")]
    RepeatedEdit(PathBuf),
    #[error("a skill source was both updated and removed: {0}")]
    ConflictingEdit(PathBuf),
}

pub trait ReadsSkillSource {
    fn read(path: &Path) -> Result<SkillEntry, RegistryError>;
}

impl ReadsSkillSource for SkillEntry {
    fn read(path: &Path) -> Result<SkillEntry, RegistryError> {
        Self::read_named(path, None)
    }
}

impl SkillEntry {
    fn read_named(path: &Path, name: Option<String>) -> Result<SkillEntry, RegistryError> {
        let path = fs::canonicalize(path).map_err(|source| RegistryError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        if path.extension().is_none_or(|extension| extension != "md") {
            return Err(RegistryError::InvalidSource {
                path,
                message: "skill sources must end in .md".into(),
            });
        }
        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_owned();
        let name = name.unwrap_or_else(|| {
            if path
                .parent()
                .and_then(Path::file_name)
                .and_then(|value| value.to_str())
                == Some("vision")
            {
                format!("vision-{stem}")
            } else {
                stem
            }
        });
        if !valid_name(&name) {
            return Err(RegistryError::InvalidSource {
                path,
                message: "filename must be a lower-case skill name".into(),
            });
        }
        let body = fs::read_to_string(&path).map_err(|source| RegistryError::Read {
            path: path.clone(),
            source,
        })?;
        let dependencies = dependencies(&path, &body)?;
        if skill_instructions(&body).trim().is_empty() {
            return Err(RegistryError::InvalidSource {
                path,
                message: "skill instructions are empty".into(),
            });
        }
        Ok(SkillEntry {
            path,
            name,
            body,
            dependencies,
        })
    }
}

pub trait ReadsSkillRepositories {
    fn from_repositories(
        psyche_repository: &Path,
        mind_skills: &Path,
        field_skills: &Path,
    ) -> Result<SkillRegistry, RegistryError>;
}

impl ReadsSkillRepositories for SkillRegistry {
    fn from_repositories(
        psyche_repository: &Path,
        mind_skills: &Path,
        field_skills: &Path,
    ) -> Result<SkillRegistry, RegistryError> {
        let psyche_repository_root = canonical_root(psyche_repository)?;
        let mind_skills_root = canonical_root(mind_skills)?;
        let field_skills_root = canonical_root(field_skills)?;
        let mut registry = SkillRegistry {
            entries: BTreeMap::new(),
            psyche_repository_root: psyche_repository_root.clone(),
            mind_skills_root,
            field_skills_root,
        };

        for path in markdown_files(&psyche_repository_root.join("skills"), false)? {
            registry.insert_source(&path)?;
        }
        for path in markdown_files(&psyche_repository_root.join("vision"), true)? {
            registry.insert_source(&path)?;
        }
        for root in [&registry.mind_skills_root, &registry.field_skills_root] {
            for path in markdown_files(root, false)? {
                registry.insert_source(&path)?;
            }
        }

        registry.validate()?;
        Ok(registry)
    }
}

fn canonical_root(root: &Path) -> Result<PathBuf, RegistryError> {
    fs::canonicalize(root).map_err(|source| RegistryError::Read {
        path: root.to_path_buf(),
        source,
    })
}

fn markdown_files(directory: &Path, missing_is_empty: bool) -> Result<Vec<PathBuf>, RegistryError> {
    let items = match fs::read_dir(directory) {
        Ok(items) => items,
        Err(source) if missing_is_empty && source.kind() == io::ErrorKind::NotFound => {
            return Ok(Vec::new());
        }
        Err(source) => {
            return Err(RegistryError::Read {
                path: directory.to_path_buf(),
                source,
            });
        }
    };
    let mut paths = Vec::new();
    for item in items {
        let item = item.map_err(|source| RegistryError::Read {
            path: directory.to_path_buf(),
            source,
        })?;
        let path = item.path();
        if path.extension().is_some_and(|extension| extension == "md") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

pub trait ExpandsSkills {
    fn expand_skills(&self, roots: &[String]) -> Result<Vec<ResolvedSkill>, RegistryError>;
    fn skill_names(&self) -> Result<Vec<String>, RegistryError>;
}

impl ExpandsSkills for SkillRegistry {
    fn expand_skills(&self, roots: &[String]) -> Result<Vec<ResolvedSkill>, RegistryError> {
        let by_name = self.by_name()?;
        let mut output = Vec::new();
        let mut visiting = Vec::<String>::new();
        let mut visited = BTreeSet::<String>::new();
        for root in roots {
            self.visit(root, &by_name, &mut visiting, &mut visited, &mut output)?;
        }
        Ok(output)
    }

    fn skill_names(&self) -> Result<Vec<String>, RegistryError> {
        Ok(self.by_name()?.into_keys().collect())
    }
}

pub trait AppliesSkillPathEdits {
    fn apply_paths(
        &mut self,
        additions: &[String],
        changes: &[String],
        removals: &[String],
    ) -> Result<SkillChange, RegistryError>;
}

impl AppliesSkillPathEdits for SkillRegistry {
    fn apply_paths(
        &mut self,
        additions: &[String],
        changes: &[String],
        removals: &[String],
    ) -> Result<SkillChange, RegistryError> {
        let mut updated_paths = BTreeSet::new();
        let mut removed_paths = BTreeSet::new();
        for path in additions.iter().chain(changes) {
            let canonical = fs::canonicalize(path).map_err(|source| RegistryError::Read {
                path: PathBuf::from(path),
                source,
            })?;
            self.check_source(&canonical)?;
            if !updated_paths.insert(canonical.clone()) {
                return Err(RegistryError::RepeatedEdit(canonical));
            }
        }
        for path in removals {
            let path = canonical_deleted_path(Path::new(path))?;
            self.check_source(&path)?;
            if !removed_paths.insert(path.clone()) {
                return Err(RegistryError::RepeatedEdit(path));
            }
        }
        for path in updated_paths.intersection(&removed_paths) {
            return Err(RegistryError::ConflictingEdit(path.clone()));
        }

        let mut next = self.clone();
        let before = next.by_name()?;
        for path in &removed_paths {
            next.entries.remove(path);
        }
        for path in &updated_paths {
            let entry = next.read_source(path)?;
            next.entries.insert(entry.path.clone(), entry);
        }
        next.validate()?;
        let after = next.by_name()?;
        let mut changed = Vec::new();
        let mut added = Vec::new();
        let mut deleted = Vec::new();
        for (name, entry) in &after {
            match before.get(name) {
                None => added.push(name.clone()),
                Some(previous) if previous != entry => changed.push(name.clone()),
                Some(_) => {}
            }
        }
        for name in before.keys() {
            if !after.contains_key(name) {
                deleted.push(name.clone());
            }
        }
        *self = next;
        Ok(SkillChange {
            first_string_vector: changed,
            second_string_vector: added,
            third_string_vector: deleted,
        })
    }
}

pub trait ComparesSkillRegistries {
    fn changes_to(&self, next: &SkillRegistry) -> Result<SkillChange, RegistryError>;
}

impl ComparesSkillRegistries for SkillRegistry {
    fn changes_to(&self, next: &SkillRegistry) -> Result<SkillChange, RegistryError> {
        let before = self.by_name()?;
        let after = next.by_name()?;
        let mut changed = Vec::new();
        let mut added = Vec::new();
        let mut deleted = Vec::new();
        for (name, entry) in &after {
            match before.get(name) {
                None => added.push(name.clone()),
                Some(previous) if previous != entry => changed.push(name.clone()),
                Some(_) => {}
            }
        }
        for name in before.keys() {
            if !after.contains_key(name) {
                deleted.push(name.clone());
            }
        }
        Ok(SkillChange {
            first_string_vector: changed,
            second_string_vector: added,
            third_string_vector: deleted,
        })
    }
}

trait ValidatesRegistry {
    fn validate(&self) -> Result<(), RegistryError>;
    fn by_name(&self) -> Result<BTreeMap<String, SkillEntry>, RegistryError>;
}

impl ValidatesRegistry for SkillRegistry {
    fn validate(&self) -> Result<(), RegistryError> {
        let by_name = self.by_name()?;
        let roots = by_name.keys().cloned().collect::<Vec<_>>();
        let names = roots;
        let _ = self.expand_skills(&names)?;
        Ok(())
    }

    fn by_name(&self) -> Result<BTreeMap<String, SkillEntry>, RegistryError> {
        let mut names = BTreeMap::<String, SkillEntry>::new();
        for entry in self.entries.values() {
            if let Some(first) = names.get(&entry.name) {
                return Err(RegistryError::DuplicateName {
                    name: entry.name.clone(),
                    first: first.path.clone(),
                    second: entry.path.clone(),
                });
            }
            names.insert(entry.name.clone(), entry.clone());
        }
        Ok(names)
    }
}

trait TraversesDependencies {
    fn visit(
        &self,
        name: &str,
        by_name: &BTreeMap<String, SkillEntry>,
        visiting: &mut Vec<String>,
        visited: &mut BTreeSet<String>,
        output: &mut Vec<ResolvedSkill>,
    ) -> Result<(), RegistryError>;
}

impl TraversesDependencies for SkillRegistry {
    fn visit(
        &self,
        name: &str,
        by_name: &BTreeMap<String, SkillEntry>,
        visiting: &mut Vec<String>,
        visited: &mut BTreeSet<String>,
        output: &mut Vec<ResolvedSkill>,
    ) -> Result<(), RegistryError> {
        if visited.contains(name) {
            return Ok(());
        }
        if let Some(position) = visiting.iter().position(|current| current == name) {
            let mut chain = visiting[position..].to_vec();
            chain.push(name.to_owned());
            return Err(RegistryError::Cycle {
                chain: chain.join(" -> "),
            });
        }
        let entry = by_name
            .get(name)
            .ok_or_else(|| RegistryError::Missing { name: name.into() })?;
        visiting.push(name.to_owned());
        for dependency in &entry.dependencies {
            self.visit(dependency, by_name, visiting, visited, output)?;
        }
        visiting.pop();
        visited.insert(name.to_owned());
        output.push(ResolvedSkill {
            path: entry.path.clone(),
            name: entry.name.clone(),
            body: entry.body.clone(),
            dependencies: entry.dependencies.clone(),
        });
        Ok(())
    }
}

trait ChecksSourceBoundary {
    fn check_source(&self, path: &Path) -> Result<(), RegistryError>;
}

impl ChecksSourceBoundary for SkillRegistry {
    fn check_source(&self, path: &Path) -> Result<(), RegistryError> {
        self.skill_name(path).map(|_| ())
    }
}

impl SkillRegistry {
    fn insert_source(&mut self, path: &Path) -> Result<(), RegistryError> {
        let entry = self.read_source(path)?;
        self.entries.insert(entry.path.clone(), entry);
        Ok(())
    }

    fn read_source(&self, path: &Path) -> Result<SkillEntry, RegistryError> {
        let canonical = fs::canonicalize(path).map_err(|source| RegistryError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let name = self.skill_name(&canonical)?;
        SkillEntry::read_named(&canonical, Some(name))
    }

    fn skill_name(&self, path: &Path) -> Result<String, RegistryError> {
        if path.extension().is_none_or(|extension| extension != "md") {
            return Err(RegistryError::OutsideSource(path.to_path_buf()));
        }
        let stem = || {
            path.file_stem()
                .and_then(|value| value.to_str())
                .filter(|value| valid_name(value))
                .map(str::to_owned)
                .ok_or_else(|| RegistryError::InvalidSource {
                    path: path.to_path_buf(),
                    message: "filename must be a lower-case skill name".into(),
                })
        };
        if let Ok(relative) = path.strip_prefix(&self.psyche_repository_root) {
            match relative.parent() {
                Some(parent) if parent == Path::new("skills") => stem(),
                Some(parent) if parent == Path::new("vision") => {
                    let name = format!("vision-{}", stem()?);
                    if valid_name(&name) {
                        Ok(name)
                    } else {
                        Err(RegistryError::InvalidSource {
                            path: path.to_path_buf(),
                            message: "vision filename must be a lower-case skill name".into(),
                        })
                    }
                }
                _ => Err(RegistryError::OutsideSource(path.to_path_buf())),
            }
        } else if path.parent() == Some(self.mind_skills_root.as_path())
            || path.parent() == Some(self.field_skills_root.as_path())
        {
            stem()
        } else {
            Err(RegistryError::OutsideSource(path.to_path_buf()))
        }
    }
}

fn dependencies(path: &Path, body: &str) -> Result<Vec<String>, RegistryError> {
    let Some(frontmatter) = body.strip_prefix("---\n") else {
        return Ok(Vec::new());
    };
    let Some((header, _)) = frontmatter.split_once("\n---") else {
        return Err(RegistryError::InvalidSource {
            path: path.to_path_buf(),
            message: "frontmatter has no closing delimiter".into(),
        });
    };
    let Some(line) = header
        .lines()
        .find_map(|line| line.trim().strip_prefix("dependencies:"))
    else {
        return Ok(Vec::new());
    };
    let line = line.trim();
    let Some(inner) = line
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Err(RegistryError::InvalidSource {
            path: path.to_path_buf(),
            message: "dependencies must be an inline vector".into(),
        });
    };
    let mut dependencies = Vec::new();
    for value in inner
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if !valid_name(value) {
            return Err(RegistryError::InvalidSource {
                path: path.to_path_buf(),
                message: format!("invalid dependency name {value}"),
            });
        }
        dependencies.push(value.to_owned());
    }
    Ok(dependencies)
}

fn skill_instructions(body: &str) -> &str {
    body.strip_prefix("---\n")
        .and_then(|frontmatter| frontmatter.split_once("\n---").map(|(_, body)| body))
        .unwrap_or(body)
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn canonical_deleted_path(path: &Path) -> Result<PathBuf, RegistryError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|source| RegistryError::Read {
                path: path.to_path_buf(),
                source,
            })?
            .join(path)
    };
    if absolute.exists() {
        return fs::canonicalize(&absolute).map_err(|source| RegistryError::Read {
            path: absolute,
            source,
        });
    }
    let parent = absolute
        .parent()
        .ok_or_else(|| RegistryError::InvalidSource {
            path: absolute.clone(),
            message: "deleted skill path has no parent".into(),
        })?;
    let file = absolute
        .file_name()
        .ok_or_else(|| RegistryError::InvalidSource {
            path: absolute.clone(),
            message: "deleted skill path has no filename".into(),
        })?;
    let parent = fs::canonicalize(parent).map_err(|source| RegistryError::Read {
        path: parent.to_path_buf(),
        source,
    })?;
    Ok(parent.join(file))
}

#[cfg(test)]
mod tests {
    use super::{
        AppliesSkillPathEdits, ExpandsSkills, ReadsSkillRepositories, RegistryError, SkillRegistry,
    };
    use std::{fs, path::Path};
    use tempfile::tempdir;

    fn repositories(root: &Path) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
        let psyche = root.join("psyche-skills");
        let mind = root.join("mind-skills");
        let field = root.join("field-skills");
        fs::create_dir_all(psyche.join("skills")).expect("Psyche skills directory");
        fs::create_dir_all(mind).expect("Mind skills directory");
        fs::create_dir_all(field).expect("Field skills directory");
        (psyche, mind, field)
    }

    #[test]
    fn vision_category_maps_to_logical_name_and_repository_docs_are_excluded() {
        let temporary = tempdir().expect("temporary root");
        let (psyche, mind, field) = repositories(temporary.path());
        fs::write(psyche.join("README.md"), "Repository notes.\n").expect("repository README");
        fs::write(
            psyche.join("skills/knowledge-vocabulary.md"),
            "Vocabulary instructions.\n",
        )
        .expect("direct skill source");
        fs::create_dir_all(psyche.join("vision")).expect("Vision category");
        fs::write(psyche.join("vision/ethos.md"), "Ethos instructions.\n")
            .expect("Vision category source");

        let registry = SkillRegistry::from_repositories(&psyche, &mind, &field)
            .expect("registry reads declared source locations");

        assert_eq!(
            registry.skill_names().expect("skill names"),
            vec!["knowledge-vocabulary".to_owned(), "vision-ethos".to_owned()]
        );
        let resolved = registry
            .expand_skills(&["vision-ethos".into()])
            .expect("logical category name resolves");
        assert_eq!(resolved[0].name, "vision-ethos");
        assert_eq!(
            resolved[0].path,
            fs::canonicalize(psyche.join("vision/ethos.md")).expect("canonical source path")
        );
    }

    #[test]
    fn category_and_direct_sources_with_one_logical_name_are_refused() {
        let temporary = tempdir().expect("temporary root");
        let (psyche, mind, field) = repositories(temporary.path());
        fs::write(
            psyche.join("skills/vision-ethos.md"),
            "Legacy Vision source.\n",
        )
        .expect("direct source");
        fs::create_dir_all(psyche.join("vision")).expect("Vision category");
        fs::write(psyche.join("vision/ethos.md"), "Category Vision source.\n")
            .expect("category source");

        let result = SkillRegistry::from_repositories(&psyche, &mind, &field);

        assert!(matches!(
            result,
            Err(RegistryError::DuplicateName { name, .. }) if name == "vision-ethos"
        ));
    }

    #[test]
    fn edits_reject_markdown_outside_the_configured_source_locations() {
        let temporary = tempdir().expect("temporary root");
        let (psyche, mind, field) = repositories(temporary.path());
        let outside = temporary.path().join("outside/rogue.md");
        fs::create_dir_all(outside.parent().expect("outside parent")).expect("outside directory");
        fs::write(&outside, "Outside source.\n").expect("outside source");
        let mut registry =
            SkillRegistry::from_repositories(&psyche, &mind, &field).expect("empty registry");

        let result = registry.apply_paths(&[outside.display().to_string()], &[], &[]);

        assert!(matches!(result, Err(RegistryError::OutsideSource(_))));
    }
}
