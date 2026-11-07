//! The messages worker (IMP-007, ADR-0012): one thread owns every provider call. It keeps
//! the snapshot the Messages view reads, brings it up to date in the background and runs
//! the user's writes one at a time, in the order they were asked.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use aurora_core::domain::{
    MessageHeader, MessagesNews, MessagesSnapshot, PollResult, ProjectInbox, UserAction,
};

use crate::commands::CommandError;

/// What the worker does with the provider and the window; the real one runs
/// `dcp-integrations` and emits Tauri events, the tests use a fake.
pub trait Port: Send + 'static {
    /// Brings the local copies up to date and reads every project (sync, then export).
    fn read(&mut self) -> Vec<ProjectInbox>;
    /// Runs one write in a project's name.
    fn act(
        &mut self,
        project: &str,
        action: &UserAction,
    ) -> Result<Vec<MessageHeader>, CommandError>;
    /// Tells the window the snapshot changed.
    fn emit(&mut self, news: &MessagesNews);
    /// The time, RFC 3339.
    fn now(&self) -> String;
}

type ActReply = Sender<Result<Vec<MessageHeader>, CommandError>>;

enum Job {
    /// Read again; the changes go to `reply` when one waits for them.
    Refresh(Option<Sender<Vec<PollResult>>>),
    Act {
        project: String,
        action: UserAction,
        reply: ActReply,
    },
    #[cfg(test)]
    Stop,
}

/// The handle the commands use; cloning it is cheap.
#[derive(Clone)]
pub struct Worker {
    jobs: Sender<Job>,
    snapshot: Arc<RwLock<MessagesSnapshot>>,
}

impl Worker {
    /// Loads the snapshot `file` holds (none, or one that cannot be read, is an empty
    /// one) and starts the thread.
    pub fn start(port: impl Port, file: Option<PathBuf>) -> Worker {
        let loaded = file.as_deref().map(load).unwrap_or_default();
        let snapshot = Arc::new(RwLock::new(loaded));
        let (jobs, inbox) = mpsc::channel();
        let shared = Arc::clone(&snapshot);
        std::thread::Builder::new()
            .name("messages-worker".into())
            .spawn(move || run(port, &inbox, &shared, file.as_deref()))
            .map_err(|e| tracing::error!(error = %e, "could not start the messages worker"))
            .ok();
        Worker { jobs, snapshot }
    }

    /// The snapshot as it is now; never waits for the provider.
    pub fn snapshot(&self) -> MessagesSnapshot {
        self.snapshot.read().map(|s| s.clone()).unwrap_or_default()
    }

    /// Asks for a reading without waiting for it.
    pub fn refresh(&self) {
        let _ = self.jobs.send(Job::Refresh(None));
    }

    /// Reads again and returns what changed (blocking: call off the async runtime).
    pub fn refresh_and_wait(&self) -> Vec<PollResult> {
        let (tx, rx) = mpsc::channel();
        if self.jobs.send(Job::Refresh(Some(tx))).is_err() {
            return Vec::new();
        }
        rx.recv().unwrap_or_default()
    }

    /// Runs a write after the ones asked before it (blocking: call off the async runtime).
    pub fn act(
        &self,
        project: String,
        action: UserAction,
    ) -> Result<Vec<MessageHeader>, CommandError> {
        let (reply, rx) = mpsc::channel();
        self.jobs
            .send(Job::Act {
                project,
                action,
                reply,
            })
            .map_err(|_| CommandError::new("messages", "the messages worker stopped"))?;
        rx.recv()
            .map_err(|_| CommandError::new("messages", "the messages worker stopped"))?
    }

    #[cfg(test)]
    fn stop(&self) {
        let _ = self.jobs.send(Job::Stop);
    }
}

/// The file's snapshot, or an empty one.
fn load(file: &std::path::Path) -> MessagesSnapshot {
    match std::fs::read(file) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|error| {
            tracing::warn!(%error, file = %file.display(), "ignoring an unreadable messages snapshot");
            MessagesSnapshot::default()
        }),
        Err(_) => MessagesSnapshot::default(),
    }
}

/// Writes the snapshot next to `file`, then moves it in place, so a crash never leaves
/// half a file.
fn save(file: &std::path::Path, snapshot: &MessagesSnapshot) {
    let result = (|| -> std::io::Result<()> {
        if let Some(dir) = file.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = file.with_extension("json.tmp");
        std::fs::write(
            &tmp,
            serde_json::to_vec(snapshot).map_err(std::io::Error::other)?,
        )?;
        std::fs::rename(&tmp, file)
    })();
    if let Err(error) = result {
        tracing::warn!(%error, file = %file.display(), "could not save the messages snapshot");
    }
}

/// How long the worker waits after a write before it reads again, so a run of writes
/// (marking several messages read) costs one reading.
const SETTLE: Duration = Duration::from_millis(300);

fn run(
    mut port: impl Port,
    jobs: &Receiver<Job>,
    shared: &RwLock<MessagesSnapshot>,
    file: Option<&std::path::Path>,
) {
    // A write asks for a reading once the queue is quiet.
    let mut refresh_due: Option<Instant> = None;
    let mut waiting: Vec<Sender<Vec<PollResult>>> = Vec::new();
    loop {
        let job = match refresh_due {
            Some(due) => match jobs.recv_timeout(due.saturating_duration_since(Instant::now())) {
                Ok(job) => Some(job),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => return,
            },
            None => match jobs.recv() {
                Ok(job) => Some(job),
                Err(_) => return,
            },
        };
        match job {
            Some(Job::Refresh(reply)) => {
                waiting.extend(reply);
                // Every refresh already queued is answered by the same reading.
                let mut later = Vec::new();
                while let Ok(next) = jobs.try_recv() {
                    match next {
                        Job::Refresh(reply) => waiting.extend(reply),
                        other => later.push(other),
                    }
                }
                let changes = read(&mut port, shared, file);
                refresh_due = None;
                for w in waiting.drain(..) {
                    let _ = w.send(changes.clone());
                }
                for job in later {
                    if !handle(job, &mut port, &mut refresh_due, &mut waiting) {
                        return;
                    }
                }
            }
            Some(job) => {
                if !handle(job, &mut port, &mut refresh_due, &mut waiting) {
                    return;
                }
            }
            None => {
                let changes = read(&mut port, shared, file);
                refresh_due = None;
                for w in waiting.drain(..) {
                    let _ = w.send(changes.clone());
                }
            }
        }
    }
}

/// Runs a write, or queues a refresh; false to stop.
fn handle(
    job: Job,
    port: &mut impl Port,
    refresh_due: &mut Option<Instant>,
    waiting: &mut Vec<Sender<Vec<PollResult>>>,
) -> bool {
    match job {
        Job::Act {
            project,
            action,
            reply,
        } => {
            let started = Instant::now();
            let result = port.act(&project, &action);
            tracing::info!(
                action = action_name(&action),
                %project,
                elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
                ok = result.is_ok(),
                "messages write"
            );
            let _ = reply.send(result);
            *refresh_due = Some(Instant::now() + SETTLE);
        }
        Job::Refresh(reply) => {
            waiting.extend(reply);
            *refresh_due = Some(Instant::now());
        }
        #[cfg(test)]
        Job::Stop => return false,
    }
    true
}

fn action_name(action: &UserAction) -> &'static str {
    match action {
        UserAction::Send { .. } => "send",
        UserAction::Reply { .. } => "reply",
        UserAction::Decide { .. } => "decide",
        UserAction::Revise { .. } => "revise",
        UserAction::Guide { .. } => "guide",
        UserAction::MarkSeen { .. } => "mark-seen",
    }
}

/// One reading: the new snapshot stored, saved and announced when something changed.
fn read(
    port: &mut impl Port,
    shared: &RwLock<MessagesSnapshot>,
    file: Option<&std::path::Path>,
) -> Vec<PollResult> {
    let started = Instant::now();
    let inboxes = port.read();
    let now = port.now();
    let before = shared.read().map(|s| s.clone()).unwrap_or_default();
    let Some(next) = before.next(inboxes, &now) else {
        tracing::debug!(
            elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
            "messages unchanged"
        );
        return Vec::new();
    };
    let news = next.news(&before);
    if let Some(file) = file {
        save(file, &next);
    }
    if let Ok(mut s) = shared.write() {
        *s = next;
    }
    tracing::info!(
        elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        version = news.version,
        changed = news.changes.len(),
        "messages snapshot"
    );
    port.emit(&news);
    news.changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::domain::{InboxSetup, ProjectId};
    use std::sync::Mutex;

    #[derive(Clone, Default)]
    struct Log(Arc<Mutex<Vec<String>>>);

    impl Log {
        fn push(&self, s: String) {
            self.0.lock().unwrap_or_else(|e| e.into_inner()).push(s);
        }
        fn all(&self) -> Vec<String> {
            self.0.lock().unwrap_or_else(|e| e.into_inner()).clone()
        }
    }

    struct Fake {
        log: Log,
        reads: u32,
    }

    fn inbox(name: &str) -> ProjectInbox {
        ProjectInbox {
            project: ProjectId("p".into()),
            name: name.into(),
            slug: Some("p".into()),
            counterparts: Vec::new(),
            setup: InboxSetup::Ready,
            conversations: Vec::new(),
        }
    }

    impl Port for Fake {
        fn read(&mut self) -> Vec<ProjectInbox> {
            self.reads += 1;
            self.log.push(format!("read {}", self.reads));
            // The name changes with each reading, so each one is news.
            vec![inbox(&format!("P{}", self.reads))]
        }
        fn act(
            &mut self,
            project: &str,
            action: &UserAction,
        ) -> Result<Vec<MessageHeader>, CommandError> {
            self.log
                .push(format!("act {project} {}", action_name(action)));
            if project == "refused" {
                return Err(CommandError::new("messages", "refused"));
            }
            Ok(Vec::new())
        }
        fn emit(&mut self, news: &MessagesNews) {
            self.log.push(format!("emit {}", news.version));
        }
        fn now(&self) -> String {
            "2026-11-07T15:00:00Z".into()
        }
    }

    fn seen(id: &str) -> UserAction {
        UserAction::MarkSeen { id: id.into() }
    }

    #[test]
    fn writes_run_in_order_and_one_reading_follows_them() {
        let log = Log::default();
        let dir = tempfile::tempdir().expect("temp dir");
        let file = dir.path().join("messages-snapshot.json");
        let w = Worker::start(
            Fake {
                log: log.clone(),
                reads: 0,
            },
            Some(file.clone()),
        );
        assert_eq!(w.snapshot().version, 0, "empty before the first reading");
        assert!(w.act("a".into(), seen("1")).is_ok());
        assert!(w.act("a".into(), seen("2")).is_ok());
        assert!(w.act("refused".into(), seen("3")).is_err());
        let changes = w.refresh_and_wait();
        assert_eq!(changes.len(), 1);
        w.stop();
        let all = log.all();
        assert_eq!(
            all[..3],
            [
                "act a mark-seen".to_owned(),
                "act a mark-seen".to_owned(),
                "act refused mark-seen".to_owned()
            ]
        );
        assert_eq!(
            all.iter().filter(|l| l.starts_with("read")).count(),
            1,
            "{all:?}"
        );
        assert_eq!(w.snapshot().version, 1);
        // Saved, and read back by the next start.
        let again = Worker::start(
            Fake {
                log: Log::default(),
                reads: 0,
            },
            Some(file),
        );
        assert_eq!(again.snapshot().version, 1);
        assert_eq!(again.snapshot().inboxes[0].name, "P1");
        again.stop();
    }

    #[test]
    fn an_unreadable_snapshot_file_starts_empty() {
        let dir = tempfile::tempdir().expect("temp dir");
        let file = dir.path().join("messages-snapshot.json");
        std::fs::write(&file, "not json").expect("write");
        assert_eq!(load(&file), MessagesSnapshot::default());
    }
}
