//! Runs an engine command without a shell, with a time limit and size limits on its
//! output (FR-015, research R5).

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use wait_timeout::ChildExt;

/// The largest standard output accepted from an engine.
pub const MAX_STDOUT_BYTES: usize = 1024 * 1024;
/// The largest standard error kept for diagnostics.
pub const MAX_STDERR_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub timeout: Duration,
    pub working_dir: Option<PathBuf>,
}

impl CommandSpec {
    pub fn new(program: &Path, args: &[&str], timeout: Duration) -> Self {
        Self {
            program: program.to_owned(),
            args: args.iter().map(|arg| (*arg).to_owned()).collect(),
            timeout,
            working_dir: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub stdout: Vec<u8>,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration: Duration,
}

impl CommandOutput {
    pub fn succeeded(&self) -> bool {
        self.exit_code == Some(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RunError {
    #[error("could not start {program}: {message}")]
    Spawn { program: String, message: String },
    #[error("no answer after {} s", timeout.as_secs_f32())]
    TimedOut { timeout: Duration, stderr: String },
    #[error("the reply is larger than {limit} bytes")]
    TooLarge { limit: usize, stderr: String },
    #[error("waiting for the command failed: {message}")]
    Wait { message: String },
}

impl RunError {
    /// The standard error captured before the failure, if any.
    pub fn stderr(&self) -> &str {
        match self {
            RunError::TimedOut { stderr, .. } | RunError::TooLarge { stderr, .. } => stderr,
            _ => "",
        }
    }
}

pub fn run(spec: &CommandSpec) -> Result<CommandOutput, RunError> {
    let started = Instant::now();
    let mut command = Command::new(&spec.program);
    command
        .args(&spec.args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(dir) = &spec.working_dir {
        command.current_dir(dir);
    }
    let mut child = command.spawn().map_err(|error| RunError::Spawn {
        program: spec.program.display().to_string(),
        message: error.to_string(),
    })?;

    let stdout_reader = child
        .stdout
        .take()
        .map(|pipe| read_capped(pipe, MAX_STDOUT_BYTES));
    let stderr_reader = child
        .stderr
        .take()
        .map(|pipe| read_capped(pipe, MAX_STDERR_BYTES));

    let waited = child.wait_timeout(spec.timeout);
    let status = match waited {
        Ok(Some(status)) => Some(status),
        Ok(None) => {
            // Best effort: the child may have exited between the timeout and the kill.
            let _ = child.kill();
            let _ = child.wait();
            None
        }
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(RunError::Wait {
                message: error.to_string(),
            });
        }
    };

    let (stdout, stdout_overflow) = join(stdout_reader);
    let (stderr_bytes, _) = join(stderr_reader);
    let stderr = String::from_utf8_lossy(&stderr_bytes).into_owned();

    let Some(status) = status else {
        return Err(RunError::TimedOut {
            timeout: spec.timeout,
            stderr,
        });
    };
    if stdout_overflow {
        return Err(RunError::TooLarge {
            limit: MAX_STDOUT_BYTES,
            stderr,
        });
    }
    Ok(CommandOutput {
        stdout,
        stderr,
        exit_code: status.code(),
        duration: started.elapsed(),
    })
}

type Reader = thread::JoinHandle<(Vec<u8>, bool)>;

/// Reads a pipe to its end, keeping at most `limit` bytes; the rest is drained so the
/// child never blocks on a full pipe. Returns the bytes and whether more were discarded.
fn read_capped<R: Read + Send + 'static>(mut pipe: R, limit: usize) -> Reader {
    thread::spawn(move || {
        let mut kept = Vec::new();
        let mut overflow = false;
        let mut buffer = [0_u8; 8192];
        loop {
            match pipe.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(read) => {
                    let room = limit.saturating_sub(kept.len());
                    if read > room {
                        overflow = true;
                    }
                    kept.extend_from_slice(&buffer[..read.min(room)]);
                }
            }
        }
        (kept, overflow)
    })
}

fn join(reader: Option<Reader>) -> (Vec<u8>, bool) {
    reader
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default()
}
