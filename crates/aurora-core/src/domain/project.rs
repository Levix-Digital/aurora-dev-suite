use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Identifier of a registered project (a UUID assigned by the registry).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProjectId(pub String);

impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A display name: trimmed, 1 to 80 characters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ProjectName(String);

impl ProjectName {
    pub const MAX_CHARS: usize = 80;

    pub fn parse(raw: &str) -> Result<Self, ProjectError> {
        let trimmed = raw.trim();
        let chars = trimmed.chars().count();
        if chars == 0 {
            return Err(ProjectError::InvalidName {
                reason: "the name is empty".to_owned(),
            });
        }
        if chars > Self::MAX_CHARS {
            return Err(ProjectError::InvalidName {
                reason: format!(
                    "the name has {chars} characters; the limit is {}",
                    Self::MAX_CHARS
                ),
            });
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ProjectName {
    type Error = ProjectError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<ProjectName> for String {
    fn from(name: ProjectName) -> Self {
        name.0
    }
}

impl fmt::Display for ProjectName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Whether a registered project's folder is still a repository on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Presence {
    Present,
    Missing,
}

/// A Git repository the user follows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: ProjectId,
    pub name: ProjectName,
    /// Canonical repository root.
    pub root: PathBuf,
    /// RFC 3339 date-time.
    pub added_at: String,
    pub presence: Presence,
}

/// The result of adding a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddOutcome {
    pub project: Project,
    /// The chosen folder was inside the repository and its root was registered instead.
    pub used_repository_root: bool,
}

/// Something the user must know about the stored list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RegistryNotice {
    /// The stored list could not be read; it was moved aside and the list starts empty.
    Recovered { moved_to: PathBuf },
    /// The list cannot be saved; changes last for this session only.
    ReadOnly { reason: String },
}

/// The registered projects and an optional notice about their storage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectList {
    pub projects: Vec<Project>,
    pub notice: Option<RegistryNotice>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ProjectError {
    #[error("the folder {path} does not exist")]
    NotFound { path: PathBuf },
    #[error("the folder {path} is not inside a Git repository")]
    NotARepository { path: PathBuf },
    #[error("this repository is already registered as \"{name}\"")]
    AlreadyRegistered { name: String },
    #[error("invalid name: {reason}")]
    InvalidName { reason: String },
    #[error("no registered project has the id {id}")]
    UnknownProject { id: String },
    #[error("the project list could not be saved: {message}")]
    Storage { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_trimmed() {
        assert_eq!(
            ProjectName::parse("  gedo ").map(|n| n.0),
            Ok("gedo".to_owned())
        );
    }

    #[test]
    fn empty_name_is_refused() {
        assert!(matches!(
            ProjectName::parse("   "),
            Err(ProjectError::InvalidName { .. })
        ));
    }

    #[test]
    fn name_limit_counts_characters_not_bytes() {
        let accented = "é".repeat(ProjectName::MAX_CHARS);
        assert!(ProjectName::parse(&accented).is_ok());
        let too_long = "a".repeat(ProjectName::MAX_CHARS + 1);
        assert!(matches!(
            ProjectName::parse(&too_long),
            Err(ProjectError::InvalidName { .. })
        ));
    }

    #[test]
    fn name_deserialization_validates() {
        let parsed: Result<ProjectName, _> = serde_json::from_str("\"\"");
        assert!(parsed.is_err());
    }
}
