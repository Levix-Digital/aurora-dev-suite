use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A backlog item as the Dev Cycle Protocol reports it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemRef {
    pub id: String,
    /// Priority of a feature, improvement or adjustment, severity of a bug.
    pub rank: Option<String>,
    pub title: String,
}

/// A bug without a severity or a release, and why it counts as untriaged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UntriagedBug {
    pub item: ItemRef,
    pub reason: String,
}

/// The first release of the roadmap that is not released yet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentRelease {
    pub version: String,
    pub name: String,
    pub open_items: Vec<ItemRef>,
}

/// A plan in progress and how far its milestones are.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanProgress {
    pub id: String,
    pub title: String,
    pub status: String,
    pub milestones_done: u32,
    pub milestones_total: u32,
}

/// The latest release candidate run of the manual test suite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateRun {
    pub run: String,
    pub version: String,
    pub not_run: u32,
    /// Fail and Blocked results.
    pub failed: u32,
    pub released: bool,
}

/// An open pull request and what it waits for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestState {
    pub number: u64,
    pub title: String,
    pub summary: String,
    pub needs_action: bool,
    pub threads_for_user: u32,
}

/// One way forward the protocol offers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextOption {
    /// Stable key, for example `plan:<id>`, `pr:<number>`, `rc:<run>`, `bugs`, `release`.
    pub key: String,
    pub title: String,
    pub reason: Option<String>,
}

/// What the protocol recommends doing next.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextStep {
    /// Whoever runs the protocol must choose before work goes on.
    pub ask: bool,
    /// The key of the recommended option.
    pub recommended: Option<String>,
    pub options: Vec<NextOption>,
}

/// A project's development cycle state, read through `dcp diagnose --json` (ADR-0006).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleState {
    pub summary: String,
    pub next: NextStep,
    pub release: Option<CurrentRelease>,
    pub plans: Vec<PlanProgress>,
    pub release_candidate: Option<CandidateRun>,
    pub pull_requests: Vec<PullRequestState>,
    pub blocking_bugs: Vec<ItemRef>,
    pub untriaged_bugs: Vec<UntriagedBug>,
    /// What the engine could not read and why.
    pub findings: Vec<String>,
    /// RFC 3339 date-time.
    pub read_at: String,
}

/// Which side must be updated when the reply's schema MAJOR is not supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateSide {
    Aurora,
    Engine,
}

impl UpdateSide {
    pub const fn advice(self) -> &'static str {
        match self {
            UpdateSide::Aurora => "update Aurora",
            UpdateSide::Engine => "update the engine",
        }
    }
}

/// Why a project's cycle state could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CycleError {
    #[error("the cycle state is unavailable: {hint}")]
    Unavailable { hint: String },
    #[error("the project's folder {path} is missing")]
    ProjectMissing { path: PathBuf },
    #[error("the Dev Cycle Protocol did not answer within {} s", *timeout_ms as f64 / 1000.0)]
    TimedOut { timeout_ms: u64, stderr: String },
    #[error("the Dev Cycle Protocol's reply is larger than {limit} bytes")]
    TooLarge { limit: usize, stderr: String },
    #[error("the Dev Cycle Protocol failed{}", exit_code.map(|code| format!(" with exit code {code}")).unwrap_or_default())]
    Failed {
        exit_code: Option<i32>,
        stderr: String,
    },
    #[error("the Dev Cycle Protocol's reply is invalid: {message}")]
    Invalid { message: String, stderr: String },
    #[error("the Dev Cycle Protocol's reply uses schema version {found}: {}", update.advice())]
    Unsupported { found: u32, update: UpdateSide },
}

impl CycleError {
    /// The engine's error output kept for the user, if any.
    pub fn stderr(&self) -> Option<&str> {
        match self {
            CycleError::TimedOut { stderr, .. }
            | CycleError::TooLarge { stderr, .. }
            | CycleError::Failed { stderr, .. }
            | CycleError::Invalid { stderr, .. } => Some(stderr).filter(|s| !s.is_empty()),
            _ => None,
        }
        .map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn every_error() -> Vec<CycleError> {
        vec![
            CycleError::Unavailable {
                hint: "Install dcp".into(),
            },
            CycleError::ProjectMissing {
                path: PathBuf::from("/src/gone"),
            },
            CycleError::TimedOut {
                timeout_ms: 10_000,
                stderr: String::new(),
            },
            CycleError::TooLarge {
                limit: 1024,
                stderr: String::new(),
            },
            CycleError::Failed {
                exit_code: Some(2),
                stderr: "boom".into(),
            },
            CycleError::Invalid {
                message: "not JSON".into(),
                stderr: String::new(),
            },
            CycleError::Unsupported {
                found: 2,
                update: UpdateSide::Aurora,
            },
        ]
    }

    #[test]
    fn every_error_has_a_message() {
        for error in every_error() {
            assert!(!error.to_string().is_empty(), "{error:?}");
        }
    }

    #[test]
    fn messages_say_what_to_do() {
        let timed_out = CycleError::TimedOut {
            timeout_ms: 10_000,
            stderr: String::new(),
        };
        assert_eq!(
            timed_out.to_string(),
            "the Dev Cycle Protocol did not answer within 10 s"
        );
        let older = CycleError::Unsupported {
            found: 0,
            update: UpdateSide::Engine,
        };
        assert!(older.to_string().ends_with("update the engine"));
    }

    #[test]
    fn empty_error_output_is_not_kept() {
        let failed = CycleError::Failed {
            exit_code: None,
            stderr: String::new(),
        };
        assert_eq!(failed.stderr(), None);
        assert_eq!(every_error()[4].stderr(), Some("boom"));
    }

    #[test]
    fn errors_serialize_with_their_kind() {
        let json = serde_json::to_value(&every_error()[1]).expect("serializes");
        assert_eq!(json["kind"], "projectMissing");
    }
}
