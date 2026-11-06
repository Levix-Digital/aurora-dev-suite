//! Git transport: each project's `aurora/inbox` branch, read and written with Git plumbing
//! on a bare cache per project, never touching a working tree (ADR-0007, research R2).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use crate::format::{self, Message};

pub const BRANCH: &str = "aurora/inbox";
pub const REF: &str = "refs/heads/aurora/inbox";
pub const INBOX_FILE: &str = "inbox.json";
pub const INBOX_SCHEMA: &str = "aurora.inbox";
pub const INBOX_SCHEMA_VERSION: &str = "1.0";

const README: &str = "# aurora/inbox\n\nThis branch is the inbox of this project for messages from other projects.\nIt has no history in common with the code and is never merged.\nDo not open pull requests from it, and keep it out of CI and branch rules.\n";
const ATTEMPTS: u32 = 8;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TransportError {
    #[error("git {command} failed: {stderr}")]
    Git { command: String, stderr: String },
    #[error("cannot read the inbox of {project} ({url}): {stderr}")]
    Unreachable {
        project: String,
        url: String,
        stderr: String,
    },
    #[error(
        "the inbox of {project} has format {found}, this provider reads {INBOX_SCHEMA} {INBOX_SCHEMA_VERSION}: update dcp-integrations"
    )]
    UnsupportedInbox { project: String, found: String },
    #[error("{0}")]
    Io(String),
    #[error("{0}")]
    Parse(String),
}

impl TransportError {
    pub fn stderr(&self) -> Option<&str> {
        match self {
            TransportError::Git { stderr, .. } | TransportError::Unreachable { stderr, .. } => {
                Some(stderr)
            }
            _ => None,
        }
    }
}

/// Who commits the inbox writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    pub name: String,
    pub email: String,
}

impl Identity {
    /// Git's configured identity in the project at `root`, or the provider's own name.
    pub fn of_project(root: &Path) -> Identity {
        let get = |key: &str| {
            Command::new("git")
                .arg("-C")
                .arg(root)
                .args(["config", "--get", key])
                .stdin(Stdio::null())
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
                .filter(|v| !v.is_empty())
        };
        Identity {
            name: get("user.name").unwrap_or_else(|| "dcp-integrations".into()),
            email: get("user.email").unwrap_or_else(|| "dcp-integrations@localhost".into()),
        }
    }
}

/// The bare caches and the identity used to write.
#[derive(Debug, Clone)]
pub struct Transport {
    pub cache: PathBuf,
    pub identity: Identity,
    pub tool: String,
    /// Reads skip the fetch of an inbox fetched less than this long ago, so a reader
    /// that runs one command per project (Aurora) fetches each inbox once (BUG-008).
    /// `None`, the default for writes and checks before writing, always fetches.
    pub reuse_fetch: Option<Duration>,
}

/// How long the read-only commands reuse a fetch.
pub const REUSE_FETCH: Duration = Duration::from_secs(30);

/// Written in a cache after each fetch; its modification time is the fetch's.
const FETCHED_STAMP: &str = "aurora-fetched";
/// How long to wait for another process fetching the same cache.
const LOCK_WAIT: Duration = Duration::from_secs(60);
/// A lock older than this was left by a process that died.
const LOCK_STALE: Duration = Duration::from_secs(120);

/// One process at a time fetches into a cache; the lock file goes away on drop.
struct CacheLock {
    path: Option<PathBuf>,
}

impl CacheLock {
    /// Waits for the lock of `repo`; after `LOCK_WAIT`, or when the lock cannot be
    /// created at all, it goes on without it (Git's own ref locks still apply).
    fn acquire(repo: &Path) -> CacheLock {
        let path = repo.with_extension("git.lock");
        let started = std::time::Instant::now();
        loop {
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(_) => return CacheLock { path: Some(path) },
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    if age(&path).is_some_and(|a| a > LOCK_STALE) {
                        let _ = std::fs::remove_file(&path);
                        continue;
                    }
                    if started.elapsed() > LOCK_WAIT {
                        return CacheLock { path: None };
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                Err(_) => return CacheLock { path: None },
            }
        }
    }
}

impl Drop for CacheLock {
    fn drop(&mut self) {
        if let Some(path) = &self.path {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// How long ago `path` was modified.
fn age(path: &Path) -> Option<Duration> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    std::time::SystemTime::now().duration_since(modified).ok()
}

/// The default cache directory: `AURORA_INBOX_CACHE`, else the user's cache directory.
pub fn default_cache() -> PathBuf {
    if let Some(dir) = std::env::var_os("AURORA_INBOX_CACHE") {
        return PathBuf::from(dir);
    }
    directories::ProjectDirs::from("digital", "Levix", "Aurora")
        .map(|d| d.cache_dir().join("inbox"))
        .unwrap_or_else(|| std::env::temp_dir().join("aurora-inbox"))
}

struct Output {
    ok: bool,
    stdout: String,
    stderr: String,
}

impl Transport {
    fn run(
        &self,
        repo: Option<&Path>,
        args: &[&str],
        input: Option<&str>,
        index: Option<&Path>,
    ) -> Result<Output, TransportError> {
        let mut cmd = Command::new("git");
        if let Some(repo) = repo {
            cmd.arg("--git-dir").arg(repo);
        }
        cmd.args(args)
            .env("GIT_AUTHOR_NAME", &self.identity.name)
            .env("GIT_AUTHOR_EMAIL", &self.identity.email)
            .env("GIT_COMMITTER_NAME", &self.identity.name)
            .env("GIT_COMMITTER_EMAIL", &self.identity.email)
            .env("GIT_TERMINAL_PROMPT", "0")
            .stdin(if input.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(index) = index {
            cmd.env("GIT_INDEX_FILE", index);
        }
        let mut child = cmd
            .spawn()
            .map_err(|e| TransportError::Io(format!("cannot run git: {e}")))?;
        if let Some(input) = input
            && let Some(mut stdin) = child.stdin.take()
        {
            stdin
                .write_all(input.as_bytes())
                .map_err(|e| TransportError::Io(e.to_string()))?;
        }
        let out = child
            .wait_with_output()
            .map_err(|e| TransportError::Io(e.to_string()))?;
        Ok(Output {
            ok: out.status.success(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).trim().to_owned(),
        })
    }

    fn git(
        &self,
        repo: &Path,
        args: &[&str],
        input: Option<&str>,
        index: Option<&Path>,
    ) -> Result<String, TransportError> {
        let out = self.run(Some(repo), args, input, index)?;
        if out.ok {
            Ok(out.stdout)
        } else {
            Err(TransportError::Git {
                command: args.iter().take(2).copied().collect::<Vec<_>>().join(" "),
                stderr: out.stderr,
            })
        }
    }

    fn cache_for(&self, project: &str) -> Result<PathBuf, TransportError> {
        let repo = self.cache.join(format!("{project}.git"));
        if !repo.exists() {
            std::fs::create_dir_all(&self.cache).map_err(|e| TransportError::Io(e.to_string()))?;
            let out = self.run(
                None,
                &["init", "--quiet", "--bare", &repo.to_string_lossy()],
                None,
                None,
            )?;
            if !out.ok {
                return Err(TransportError::Git {
                    command: "init".into(),
                    stderr: out.stderr,
                });
            }
        }
        Ok(repo)
    }

    /// The remote tip of the project's inbox branch, without fetching (`None`: no branch).
    pub fn remote_tip(&self, project: &str, url: &str) -> Result<Option<String>, TransportError> {
        let out = self.run(None, &["ls-remote", url, REF], None, None)?;
        if !out.ok {
            return Err(TransportError::Unreachable {
                project: project.into(),
                url: url.into(),
                stderr: out.stderr,
            });
        }
        Ok(out.stdout.split_whitespace().next().map(str::to_owned))
    }

    /// Fetches the project's inbox branch into its cache, or reuses a fetch younger than
    /// `reuse_fetch`; the cache and the tip. One process fetches a cache at a time.
    pub fn fetch(
        &self,
        project: &str,
        url: &str,
    ) -> Result<(PathBuf, Option<String>), TransportError> {
        self.fetch_with(project, url, self.reuse_fetch)
    }

    fn fetch_with(
        &self,
        project: &str,
        url: &str,
        reuse: Option<Duration>,
    ) -> Result<(PathBuf, Option<String>), TransportError> {
        let repo = self.cache_for(project)?;
        let _lock = CacheLock::acquire(&repo);
        let stamp = repo.join(FETCHED_STAMP);
        if let Some(max) = reuse
            && age(&stamp).is_some_and(|a| a < max)
        {
            let tip = std::fs::read_to_string(&stamp).unwrap_or_default();
            let tip = tip.trim();
            return Ok((repo, (!tip.is_empty()).then(|| tip.to_owned())));
        }
        let (repo, tip) = self.fetch_now(project, url, repo)?;
        let _ = std::fs::write(&stamp, tip.as_deref().unwrap_or_default());
        Ok((repo, tip))
    }

    fn fetch_now(
        &self,
        project: &str,
        url: &str,
        repo: PathBuf,
    ) -> Result<(PathBuf, Option<String>), TransportError> {
        let refspec = format!("+{REF}:{REF}");
        let out = self.run(
            Some(&repo),
            &["fetch", "--quiet", "--no-tags", url, &refspec],
            None,
            None,
        )?;
        if !out.ok {
            let lower = out.stderr.to_lowercase();
            if lower.contains("couldn't find remote ref") {
                let _ = self.run(Some(&repo), &["update-ref", "-d", REF], None, None);
                return Ok((repo, None));
            }
            return Err(TransportError::Unreachable {
                project: project.into(),
                url: url.into(),
                stderr: out.stderr,
            });
        }
        let tip = self
            .git(&repo, &["rev-parse", REF], None, None)?
            .trim()
            .to_owned();
        Ok((repo, Some(tip)))
    }

    fn check_format(&self, project: &str, repo: &Path, tip: &str) -> Result<(), TransportError> {
        let spec = format!("{tip}:{INBOX_FILE}");
        let out = self.run(Some(repo), &["show", &spec], None, None)?;
        if !out.ok {
            return Ok(()); // written by the provisional tool only: read as 1.0
        }
        let value: serde_json::Value = serde_json::from_str(&out.stdout)
            .map_err(|e| TransportError::Parse(format!("{project}: {INBOX_FILE}: {e}")))?;
        let schema = value
            .get("schema")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let version = value
            .get("schema_version")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if schema != INBOX_SCHEMA || !version.starts_with("1.") {
            return Err(TransportError::UnsupportedInbox {
                project: project.into(),
                found: format!("{schema} {version}"),
            });
        }
        Ok(())
    }

    /// Every message of the project's inbox, in date order.
    pub fn read_inbox(&self, project: &str, url: &str) -> Result<Vec<Message>, TransportError> {
        let (repo, tip) = self.fetch(project, url)?;
        let Some(tip) = tip else {
            return Ok(Vec::new());
        };
        self.check_format(project, &repo, &tip)?;
        let names = self.git(
            &repo,
            &["ls-tree", "-r", "--name-only", &tip, "--", "messages"],
            None,
            None,
        )?;
        let mut messages = Vec::new();
        for name in names.lines().filter(|n| n.ends_with(".md")) {
            let text = self.git(&repo, &["show", &format!("{tip}:{name}")], None, None)?;
            messages.push(
                format::parse(&text, name).map_err(|e| TransportError::Parse(e.to_string()))?,
            );
        }
        messages.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.id.cmp(&b.id)));
        Ok(messages)
    }

    /// Adds or replaces files on the project's inbox branch and pushes, rebuilding on the
    /// new tip when a push loses a race. Adds `README.md` on a new branch and `inbox.json`
    /// when it is missing.
    pub fn write_files(
        &self,
        project: &str,
        url: &str,
        files: &[(String, String)],
        message: &str,
    ) -> Result<(), TransportError> {
        let mut last = String::new();
        for attempt in 0..ATTEMPTS {
            if attempt > 0 {
                thread::sleep(backoff(attempt));
            }
            // A write always starts from the remote tip.
            let (repo, tip) = self.fetch_with(project, url, None)?;
            if let Some(tip) = &tip {
                self.check_format(project, &repo, tip)?;
            }
            let tmp = tempfile_dir()?;
            let index = tmp.join("index");
            let mut all: Vec<(String, String)> = Vec::new();
            match &tip {
                Some(tip) => {
                    self.git(&repo, &["read-tree", tip], None, Some(&index))?;
                }
                None => all.push(("README.md".into(), README.into())),
            }
            let has_inbox_json = match &tip {
                Some(tip) => {
                    self.run(
                        Some(&repo),
                        &["cat-file", "-e", &format!("{tip}:{INBOX_FILE}")],
                        None,
                        None,
                    )?
                    .ok
                }
                None => false,
            };
            if !has_inbox_json {
                all.push((INBOX_FILE.into(), inbox_json(&self.tool)));
            }
            all.extend(files.iter().cloned());
            for (path, text) in &all {
                let blob = self.git(&repo, &["hash-object", "-w", "--stdin"], Some(text), None)?;
                let info = format!("100644,{},{path}", blob.trim());
                self.git(
                    &repo,
                    &["update-index", "--add", "--cacheinfo", &info],
                    None,
                    Some(&index),
                )?;
            }
            let tree = self
                .git(&repo, &["write-tree"], None, Some(&index))?
                .trim()
                .to_owned();
            let _ = std::fs::remove_dir_all(&tmp);
            let mut args = vec!["commit-tree", tree.as_str()];
            if let Some(tip) = &tip {
                args.extend(["-p", tip.as_str()]);
            }
            args.extend(["-m", message]);
            let commit = self.git(&repo, &args, None, None)?.trim().to_owned();
            let refspec = format!("{commit}:{REF}");
            let out = self.run(Some(&repo), &["push", "--quiet", url, &refspec], None, None)?;
            if out.ok {
                self.git(&repo, &["update-ref", REF, &commit], None, None)?;
                // The cache now holds the new tip: a read may reuse it.
                let _ = std::fs::write(repo.join(FETCHED_STAMP), &commit);
                return Ok(());
            }
            last = out.stderr;
        }
        Err(TransportError::Git {
            command: format!("push to the inbox of {project}"),
            stderr: last,
        })
    }
}

fn inbox_json(tool: &str) -> String {
    format!(
        "{{\n  \"schema\": \"{INBOX_SCHEMA}\",\n  \"schema_version\": \"{INBOX_SCHEMA_VERSION}\",\n  \"writer\": \"dcp-integrations {tool}\"\n}}\n"
    )
}

fn backoff(attempt: u32) -> Duration {
    let nanos = u64::from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0),
    );
    let base = 200 * (1u64 << attempt.min(4));
    Duration::from_millis(base + nanos % base)
}

fn tempfile_dir() -> Result<PathBuf, TransportError> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("dcp-integrations-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| TransportError::Io(e.to_string()))?;
    Ok(dir)
}
