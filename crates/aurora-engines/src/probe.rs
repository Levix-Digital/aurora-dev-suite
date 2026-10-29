//! `EngineProbe` over the engines' binaries: locate, ask `version --json`, fall back to
//! `--version` (research R6). Nothing else is ever run or read (ADR-0006).

use std::path::{Path, PathBuf};
use std::time::Duration;

use aurora_core::domain::{EngineKind, EngineReport, EngineSettings, EngineStatus};
use aurora_core::ports::EngineProbe;

use crate::contract::{self, ContractError, SUPPORTED_MAJOR};
use crate::runner::{self, CommandOutput, CommandSpec, RunError};

/// Probes the real engines installed on this machine.
#[derive(Debug, Default, Clone)]
pub struct CliProbe {
    /// Replaces the system search path; used by tests.
    search_path: Option<std::ffi::OsString>,
}

impl CliProbe {
    pub fn new() -> Self {
        Self::default()
    }

    /// A probe that looks for engines only in `dirs` instead of the system search path.
    pub fn with_search_path(dirs: &[&Path]) -> Self {
        Self {
            search_path: std::env::join_paths(dirs).ok(),
        }
    }

    fn locate(&self, kind: EngineKind, settings: &EngineSettings) -> Result<PathBuf, String> {
        if let Some(path) = &settings.path {
            return if path.is_file() {
                Ok(path.clone())
            } else {
                Err(format!(
                    "the configured path {} is not a file",
                    path.display()
                ))
            };
        }
        let found = match &self.search_path {
            Some(dirs) => which::which_in(kind.command(), Some(dirs), "."),
            None => which::which(kind.command()),
        };
        found.map_err(|_| format!("`{}` is not on the search path", kind.command()))
    }
}

impl EngineProbe for CliProbe {
    fn probe(&self, kind: EngineKind, settings: &EngineSettings) -> EngineReport {
        let checked_at = crate::now_rfc3339();
        let location = match self.locate(kind, settings) {
            Ok(location) => location,
            Err(reason) => {
                tracing::info!(engine = kind.command(), %reason, "engine missing");
                return EngineReport {
                    last_error: settings.path.as_ref().map(|_| reason),
                    ..EngineReport::missing(kind, &checked_at)
                };
            }
        };
        let timeout = time_limit(settings.timeout_ms);
        let finding = probe_binary(kind, &location, timeout);
        tracing::info!(
            engine = kind.command(),
            location = %location.display(),
            outcome = ?finding.status,
            "engine probed"
        );
        EngineReport {
            kind,
            status: finding.status,
            location: Some(location),
            version: finding.version,
            capabilities: finding.capabilities,
            last_error: finding.last_error,
            checked_at,
        }
    }
}

/// The shortest and longest time limits accepted from the settings file.
pub const MIN_TIMEOUT_MS: u64 = 100;
pub const MAX_TIMEOUT_MS: u64 = 120_000;

/// The settings' time limit, kept within bounds: the file can be edited by hand.
fn time_limit(timeout_ms: u64) -> Duration {
    Duration::from_millis(timeout_ms.clamp(MIN_TIMEOUT_MS, MAX_TIMEOUT_MS))
}

struct Finding {
    status: EngineStatus,
    version: Option<String>,
    capabilities: Vec<aurora_core::domain::Capability>,
    last_error: Option<String>,
}

impl Finding {
    fn degraded(reason: String, stderr: &str) -> Self {
        Self {
            last_error: Some(error_text(&reason, stderr)),
            status: EngineStatus::Degraded { reason },
            version: None,
            capabilities: Vec::new(),
        }
    }
}

fn probe_binary(kind: EngineKind, location: &Path, timeout: Duration) -> Finding {
    let contract_reply = run_logged(kind, location, &["version", "--json"], timeout);
    let output = match contract_reply {
        Ok(output) => output,
        Err(error) => return Finding::degraded(error.to_string(), error.stderr()),
    };
    if output.succeeded() {
        match contract::parse_version_reply(kind, &output.stdout) {
            Ok(reply) => {
                return Finding {
                    status: EngineStatus::Ready,
                    version: Some(reply.version),
                    capabilities: reply.capabilities,
                    last_error: None,
                };
            }
            Err(ContractError::NotAnEnvelope) => {}
            Err(error) => {
                return Finding::degraded(degraded_reason(kind, &error), &output.stderr);
            }
        }
    }
    plain_version(kind, location, timeout, &output.stderr)
}

/// The engine did not answer the contract: does it answer `--version`?
fn plain_version(kind: EngineKind, location: &Path, timeout: Duration, first: &str) -> Finding {
    match run_logged(kind, location, &["--version"], timeout) {
        Ok(output) if output.succeeded() => Finding {
            status: EngineStatus::InstalledWithoutContract,
            version: version_from_text(&String::from_utf8_lossy(&output.stdout)),
            capabilities: Vec::new(),
            last_error: (!first.trim().is_empty()).then(|| first.to_owned()),
        },
        Ok(output) => Finding::degraded(
            format!(
                "`{} --version` failed with {}",
                kind.command(),
                exit_text(output.exit_code)
            ),
            &output.stderr,
        ),
        Err(error) => Finding::degraded(error.to_string(), error.stderr()),
    }
}

fn run_logged(
    kind: EngineKind,
    location: &Path,
    args: &[&str],
    timeout: Duration,
) -> Result<CommandOutput, RunError> {
    let result = runner::run(&CommandSpec::new(location, args, timeout));
    match &result {
        Ok(output) => tracing::debug!(
            engine = kind.command(),
            ?args,
            duration_ms = output.duration.as_millis() as u64,
            exit = ?output.exit_code,
            "engine command"
        ),
        Err(error) => tracing::debug!(engine = kind.command(), ?args, %error, "engine command"),
    }
    result
}

fn degraded_reason(kind: EngineKind, error: &ContractError) -> String {
    match error {
        ContractError::UnsupportedMajor { found } if *found > SUPPORTED_MAJOR => format!(
            "{} speaks version {found} of the contract; update Aurora",
            kind.display_name()
        ),
        ContractError::UnsupportedMajor { found } => format!(
            "{} speaks version {found} of the contract; update {}",
            kind.display_name(),
            kind.display_name()
        ),
        other => format!("{} gave an invalid reply: {other}", kind.display_name()),
    }
}

fn error_text(reason: &str, stderr: &str) -> String {
    if stderr.trim().is_empty() {
        reason.to_owned()
    } else {
        format!("{reason}\n\n{stderr}")
    }
}

fn exit_text(code: Option<i32>) -> String {
    code.map_or_else(
        || "no exit code".to_owned(),
        |code| format!("exit code {code}"),
    )
}

/// The version in a `--version` line such as `actx 0.38.0` or `gedo v0.3.0 (abc)`.
fn version_from_text(text: &str) -> Option<String> {
    let line = text.lines().next()?.trim();
    line.split_whitespace()
        .map(|word| word.trim_start_matches('v'))
        .find(|word| word.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_limits_from_the_settings_are_kept_within_bounds() {
        assert_eq!(time_limit(0), Duration::from_millis(MIN_TIMEOUT_MS));
        assert_eq!(time_limit(5_000), Duration::from_secs(5));
        assert_eq!(time_limit(u64::MAX), Duration::from_millis(MAX_TIMEOUT_MS));
    }

    #[test]
    fn reads_the_version_from_common_lines() {
        assert_eq!(version_from_text("actx 0.38.0\n"), Some("0.38.0".into()));
        assert_eq!(version_from_text("gedo v0.3.0 (abc)"), Some("0.3.0".into()));
        assert_eq!(version_from_text("no version here"), None);
        assert_eq!(version_from_text(""), None);
    }

    #[test]
    fn newer_contracts_ask_to_update_aurora_and_older_the_engine() {
        let newer = degraded_reason(
            EngineKind::Gedo,
            &ContractError::UnsupportedMajor { found: 2 },
        );
        assert!(newer.ends_with("update Aurora"), "{newer}");
        let older = degraded_reason(
            EngineKind::Gedo,
            &ContractError::UnsupportedMajor { found: 0 },
        );
        assert!(older.ends_with("update Gedo"), "{older}");
    }
}
