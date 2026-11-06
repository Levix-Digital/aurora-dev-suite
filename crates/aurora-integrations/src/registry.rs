//! `.aurora/integrations.json`: the project's identity and its counterparts, the
//! provider's only registry (FEAT-029, ADR-0010).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const FILE: &str = ".aurora/integrations.json";
pub const SCHEMA: &str = "aurora.integrations";
pub const SCHEMA_VERSION: &str = "1.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Registry {
    pub schema: String,
    pub schema_version: String,
    pub project: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    pub repository: String,
    #[serde(default)]
    pub counterparts: Vec<Counterpart>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counterpart {
    pub slug: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub consumes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub follows: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RegistryError {
    #[error("{FILE} not found in {0}: run 'dcp-integrations init --project <slug>' there")]
    NotSetUp(PathBuf),
    #[error("{FILE} already exists in {0}")]
    Exists(PathBuf),
    #[error("'{0}' is already a counterpart")]
    DuplicateSlug(String),
    #[error("{FILE} is invalid: {0}")]
    Invalid(String),
    #[error("cannot read or write {FILE}: {0}")]
    Io(String),
}

/// Checks a slug: 2 to 40 characters, lowercase letters, digits and dashes, starting with
/// a letter or digit.
pub fn valid_slug(slug: &str) -> bool {
    let len = slug.chars().count();
    (2..=40).contains(&len)
        && slug
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

impl Registry {
    pub fn new(project: &str, name: &str, repository: &str) -> Registry {
        Registry {
            schema: SCHEMA.into(),
            schema_version: SCHEMA_VERSION.into(),
            project: project.into(),
            name: name.into(),
            repository: repository.into(),
            counterparts: Vec::new(),
        }
    }

    /// Every problem of the file, empty when it is valid.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.schema != SCHEMA {
            out.push(format!(
                "schema must be '{SCHEMA}', found '{}'",
                self.schema
            ));
        }
        if !self.schema_version.starts_with("1.") {
            out.push(format!(
                "schema_version {} is not supported (MAJOR 1 expected)",
                self.schema_version
            ));
        }
        if !valid_slug(&self.project) {
            out.push(format!("project '{}' is not a valid slug", self.project));
        }
        if self.repository.trim().is_empty() {
            out.push("repository is empty".into());
        }
        let mut seen = vec![self.project.as_str()];
        for c in &self.counterparts {
            if !valid_slug(&c.slug) {
                out.push(format!("counterpart '{}' is not a valid slug", c.slug));
            }
            if seen.contains(&c.slug.as_str()) {
                out.push(format!("'{}' appears twice", c.slug));
            }
            seen.push(&c.slug);
            if c.url.trim().is_empty() {
                out.push(format!("counterpart '{}' has no url", c.slug));
            }
        }
        out
    }

    /// The repository of a project known to this registry (itself or a counterpart).
    pub fn url_of(&self, slug: &str) -> Option<&str> {
        if slug == self.project {
            return Some(&self.repository);
        }
        self.counterparts
            .iter()
            .find(|c| c.slug == slug)
            .map(|c| c.url.as_str())
    }

    /// Every known slug, this project first.
    pub fn slugs(&self) -> Vec<&str> {
        std::iter::once(self.project.as_str())
            .chain(self.counterparts.iter().map(|c| c.slug.as_str()))
            .collect()
    }

    pub fn add(&mut self, slug: &str, name: &str, url: &str) -> Result<(), RegistryError> {
        if slug == self.project || self.counterparts.iter().any(|c| c.slug == slug) {
            return Err(RegistryError::DuplicateSlug(slug.into()));
        }
        if !valid_slug(slug) {
            return Err(RegistryError::Invalid(format!(
                "'{slug}' is not a valid slug"
            )));
        }
        if url.trim().is_empty() {
            return Err(RegistryError::Invalid("the url is empty".into()));
        }
        self.counterparts.push(Counterpart {
            slug: slug.into(),
            name: name.into(),
            url: url.into(),
            consumes: Vec::new(),
            follows: Vec::new(),
        });
        Ok(())
    }
}

pub fn path(root: &Path) -> PathBuf {
    root.join(FILE)
}

/// Reads and checks the registry of the project at `root`.
pub fn load(root: &Path) -> Result<Registry, RegistryError> {
    let file = path(root);
    let text = match fs::read_to_string(&file) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(RegistryError::NotSetUp(root.to_path_buf()));
        }
        Err(e) => return Err(RegistryError::Io(e.to_string())),
    };
    let registry: Registry =
        serde_json::from_str(&text).map_err(|e| RegistryError::Invalid(e.to_string()))?;
    let problems = registry.problems();
    if problems.is_empty() {
        Ok(registry)
    } else {
        Err(RegistryError::Invalid(problems.join("; ")))
    }
}

/// Writes the registry, creating `.aurora/`, atomically through a temporary file.
pub fn save(root: &Path, registry: &Registry) -> Result<(), RegistryError> {
    let file = path(root);
    let dir = file.parent().unwrap_or(root);
    fs::create_dir_all(dir).map_err(|e| RegistryError::Io(e.to_string()))?;
    let text = serde_json::to_string_pretty(registry)
        .map_err(|e| RegistryError::Io(e.to_string()))?
        + "\n";
    let tmp = dir.join(".integrations.json.tmp");
    fs::write(&tmp, text).map_err(|e| RegistryError::Io(e.to_string()))?;
    fs::rename(&tmp, &file).map_err(|e| RegistryError::Io(e.to_string()))
}

/// `init`: a new registry with no counterparts; refuses an existing file.
pub fn init(
    root: &Path,
    project: &str,
    name: &str,
    repository: &str,
) -> Result<Registry, RegistryError> {
    if path(root).exists() {
        return Err(RegistryError::Exists(root.to_path_buf()));
    }
    let registry = Registry::new(project, name, repository);
    let problems = registry.problems();
    if !problems.is_empty() {
        return Err(RegistryError::Invalid(problems.join("; ")));
    }
    save(root, &registry)?;
    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_add_load_validate() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(load(dir.path()), Err(RegistryError::NotSetUp(_))));
        let mut r = init(dir.path(), "aurora-dev-suite", "Aurora", "https://x/a.git").unwrap();
        assert!(matches!(
            init(dir.path(), "aurora-dev-suite", "", "u"),
            Err(RegistryError::Exists(_))
        ));
        r.add("gedo", "Gedo", "https://x/g.git").unwrap();
        assert!(matches!(
            r.add("gedo", "", "u"),
            Err(RegistryError::DuplicateSlug(_))
        ));
        assert!(matches!(
            r.add("aurora-dev-suite", "", "u"),
            Err(RegistryError::DuplicateSlug(_))
        ));
        assert!(r.add("Bad Slug", "", "u").is_err());
        save(dir.path(), &r).unwrap();
        let back = load(dir.path()).unwrap();
        assert_eq!(back, r);
        assert_eq!(back.url_of("gedo"), Some("https://x/g.git"));
        assert_eq!(back.slugs(), ["aurora-dev-suite", "gedo"]);
    }

    #[test]
    fn invalid_files_are_reported() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".aurora")).unwrap();
        fs::write(path(dir.path()), r#"{"schema":"aurora.integrations","schema_version":"2.0","project":"x","repository":"","counterparts":[]}"#).unwrap();
        let Err(RegistryError::Invalid(msg)) = load(dir.path()) else {
            panic!()
        };
        assert!(
            msg.contains("MAJOR 1")
                && msg.contains("not a valid slug")
                && msg.contains("repository is empty")
        );
        fs::write(path(dir.path()), "not json").unwrap();
        assert!(matches!(load(dir.path()), Err(RegistryError::Invalid(_))));
    }

    #[test]
    fn slugs() {
        assert!(valid_slug("dev-cycle-protocol"));
        assert!(!valid_slug("-x"));
        assert!(!valid_slug("a"));
        assert!(!valid_slug("Gedo"));
    }
}
