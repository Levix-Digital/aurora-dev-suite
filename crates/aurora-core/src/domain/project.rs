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
    /// The project's picture in Aurora (FEAT-039): a `data:image/...;base64,` URL of a
    /// small image the user picked; `None` shows its initials.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
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
    #[error("this image cannot be a logo: {reason}")]
    InvalidLogo { reason: String },
}

/// The largest image a project logo may be (FEAT-039).
pub const MAX_LOGO_BYTES: usize = 256 * 1024;

/// A project logo as a `data:` URL from an image file the user picked: PNG, JPEG, WebP or
/// SVG by its extension, at most [`MAX_LOGO_BYTES`].
pub fn logo_data_url(file_name: &str, bytes: &[u8]) -> Result<String, ProjectError> {
    let extension = file_name
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    let mime = match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => {
            return Err(ProjectError::InvalidLogo {
                reason: "use a PNG, JPEG, WebP or SVG file".to_owned(),
            });
        }
    };
    if bytes.is_empty() {
        return Err(ProjectError::InvalidLogo {
            reason: "the file is empty".to_owned(),
        });
    }
    if bytes.len() > MAX_LOGO_BYTES {
        return Err(ProjectError::InvalidLogo {
            reason: format!(
                "the file has {} KB; the limit is {} KB",
                bytes.len().div_ceil(1024),
                MAX_LOGO_BYTES / 1024
            ),
        });
    }
    Ok(format!("data:{mime};base64,{}", base64(bytes)))
}

/// Standard Base64 with padding.
fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        let index = |shift: u32| char::from(TABLE[((n >> shift) & 63) as usize]);
        out.push(index(18));
        out.push(index(12));
        out.push(if chunk.len() > 1 { index(6) } else { '=' });
        out.push(if chunk.len() > 2 { index(0) } else { '=' });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_logo_is_a_small_image_as_a_data_url() {
        assert_eq!(base64(b"Man"), "TWFu");
        assert_eq!(base64(b"Ma"), "TWE=");
        assert_eq!(base64(b"M"), "TQ==");
        assert_eq!(
            logo_data_url("Logo.SVG", b"<svg/>").unwrap(),
            "data:image/svg+xml;base64,PHN2Zy8+"
        );
        assert!(matches!(
            logo_data_url("logo.gif", b"x"),
            Err(ProjectError::InvalidLogo { .. })
        ));
        assert!(logo_data_url("logo.png", &[]).is_err());
        let big = vec![0_u8; MAX_LOGO_BYTES + 1];
        assert!(
            logo_data_url("logo.png", &big)
                .unwrap_err()
                .to_string()
                .contains("the limit is 256 KB")
        );
    }

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
