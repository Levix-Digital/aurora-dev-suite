//! Running `dcp` and checking the versioned envelope of its replies, shared by the cycle
//! and settings adapters (ADR-0003, ADR-0006).

use std::path::Path;

use aurora_core::domain::{CycleError, UpdateSide};
use serde::Deserialize;

use crate::repo;
use crate::runner::{self, CommandOutput, CommandSpec, MAX_STDOUT_BYTES, RunError};

/// Runs `spec`; a run that failed or exited with another code than 0 is an error.
pub fn run(spec: &CommandSpec) -> Result<CommandOutput, CycleError> {
    let output = runner::run(spec).map_err(|error| match error {
        RunError::TimedOut { timeout, stderr } => CycleError::TimedOut {
            timeout_ms: u64::try_from(timeout.as_millis()).unwrap_or(u64::MAX),
            stderr,
        },
        RunError::TooLarge { stderr, .. } => CycleError::TooLarge {
            limit: MAX_STDOUT_BYTES,
            stderr,
        },
        RunError::Spawn { message, .. } | RunError::Wait { message } => CycleError::Failed {
            exit_code: None,
            stderr: message,
        },
    })?;
    if !output.succeeded() {
        return Err(CycleError::Failed {
            exit_code: output.exit_code,
            stderr: output.stderr,
        });
    }
    Ok(output)
}

#[derive(Deserialize)]
struct Envelope {
    schema: String,
    schema_version: String,
}

/// The reply as JSON once its `schema` is `schema` and its MAJOR is `supported_major`.
pub fn envelope(
    bytes: &[u8],
    schema: &str,
    supported_major: u32,
) -> Result<serde_json::Value, CycleError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| invalid("the reply is not JSON"))?;
    let envelope: Envelope = serde_json::from_value(value.clone())
        .map_err(|_| invalid("the reply is not a versioned JSON envelope"))?;
    if envelope.schema != schema {
        return Err(invalid(format!(
            "the reply has schema `{}`, expected `{schema}`",
            envelope.schema
        )));
    }
    let major = envelope
        .schema_version
        .split('.')
        .next()
        .and_then(|major| major.parse::<u32>().ok())
        .ok_or_else(|| {
            invalid(format!(
                "schema_version `{}` is not MAJOR.MINOR",
                envelope.schema_version
            ))
        })?;
    if major != supported_major {
        let update = if major > supported_major {
            UpdateSide::Aurora
        } else {
            UpdateSide::Engine
        };
        return Err(CycleError::Unsupported {
            found: major,
            update,
        });
    }
    Ok(value)
}

/// An error unless the reply's `root` is the repository asked about.
pub fn check_root(reply: &str, asked: &Path) -> Result<(), CycleError> {
    let reply_path = Path::new(reply);
    let canonical_reply = repo::canonicalize(reply_path).unwrap_or_else(|| reply_path.to_owned());
    let canonical_asked = repo::canonicalize(asked).unwrap_or_else(|| asked.to_owned());
    if repo::same_path(&canonical_reply, &canonical_asked) {
        Ok(())
    } else {
        Err(invalid(format!(
            "the reply is about {reply}, not {}",
            asked.display()
        )))
    }
}

pub fn invalid(message: impl Into<String>) -> CycleError {
    CycleError::Invalid {
        message: message.into(),
        stderr: String::new(),
    }
}

/// Adds the engine's error output to an invalid reply.
pub fn with_stderr(error: CycleError, stderr: String) -> CycleError {
    match error {
        CycleError::Invalid { message, .. } => CycleError::Invalid { message, stderr },
        other => other,
    }
}

/// Milliseconds since `started`, for the logs.
pub fn elapsed_ms(started: std::time::Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}
