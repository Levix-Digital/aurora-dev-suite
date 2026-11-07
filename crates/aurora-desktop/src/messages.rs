//! The commands of the Messages view, the messages worker's provider side and its timer
//! (spec 008, spec 010, contracts/desktop-commands.md). The view reads the worker's
//! snapshot; every provider call runs on the worker's thread, at most four at once.

use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(test)]
use aurora_core::domain::Settings;
use aurora_core::domain::{
    EngineKind, EngineReport, MessageHeader, MessageKind, MessagesNews, MessagesSnapshot,
    PollResult, Project, ProjectId, ProjectInbox, TemplateSection, UserAction,
};
use aurora_core::usecases::{messages, projects, settings};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::commands::{CommandError, Reply, blocking, engines_overview};
use crate::state::AppState;
use crate::worker::{self, Worker};

/// The event of a new snapshot, with what the navigation shows (IMP-007).
pub const MESSAGES_SNAPSHOT: &str = "messages-snapshot";
/// The snapshot's file in Aurora's configuration folder (ADR-0012).
pub const SNAPSHOT_FILE: &str = "messages-snapshot.json";
/// Provider processes at once.
pub const PARALLEL: usize = 4;

/// The provider's latest report, probing first when none ran.
async fn provider(app: AppHandle, state: &State<'_, AppState>) -> Reply<Option<EngineReport>> {
    let overview = engines_overview(app, state.clone()).await?;
    Ok(overview
        .reports
        .into_iter()
        .find(|report| report.kind == EngineKind::Integrations))
}

fn project_of(state: &AppState, id: &str) -> Reply<Project> {
    let id = ProjectId(id.to_owned());
    projects::list(state.registry.as_ref())
        .projects
        .into_iter()
        .find(|project| project.id == id)
        .ok_or_else(|| {
            CommandError::from(aurora_core::domain::ProjectError::UnknownProject {
                id: id.0.clone(),
            })
        })
}

/// Runs `work` on every item, `PARALLEL` at a time, keeping the order.
fn in_parallel<T: Sync, R: Send>(items: &[T], work: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let mut out = Vec::with_capacity(items.len());
    for chunk in items.chunks(PARALLEL) {
        std::thread::scope(|scope| {
            let handles: Vec<_> = chunk
                .iter()
                .map(|item| scope.spawn(|| work(item)))
                .collect();
            out.extend(handles.into_iter().filter_map(|h| h.join().ok()));
        });
    }
    out
}

/// The worker's real side: `dcp-integrations` for every registered project, and Tauri
/// events for the window.
pub struct ProviderPort {
    app: AppHandle,
}

impl ProviderPort {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl worker::Port for ProviderPort {
    fn read(&mut self) -> Vec<ProjectInbox> {
        let state = self.app.state::<AppState>();
        // No provider probed yet: the view asks once the probe ran.
        let Some(provider) = last_provider(&state) else {
            return previous_inboxes(&state);
        };
        let list = projects::list(state.registry.as_ref()).projects;
        let current = settings::get(state.settings.as_ref());
        // Bring the local copies up to date first (ADR-0011), then read from them.
        let _ = in_parallel(&list, |project| {
            messages::sync(
                state.messages.as_ref(),
                Some(&provider),
                &current,
                std::slice::from_ref(project),
            )
        });
        messages::complete(in_parallel(&list, |project| {
            messages::export_inbox(state.messages.as_ref(), Some(&provider), &current, project)
        }))
    }

    fn act(
        &mut self,
        project: &str,
        action: &UserAction,
    ) -> Result<Vec<MessageHeader>, CommandError> {
        let state = self.app.state::<AppState>();
        let project = project_of(&state, project)?;
        let current = settings::get(state.settings.as_ref());
        messages::act(
            state.messages.as_ref(),
            last_provider(&state).as_ref(),
            &current,
            &project,
            action,
        )
        .map_err(CommandError::from)
    }

    fn emit(&mut self, news: &MessagesNews) {
        if let Err(error) = self.app.emit(MESSAGES_SNAPSHOT, news) {
            tracing::warn!(%error, "could not emit messages-snapshot");
        }
    }

    fn now(&self) -> String {
        time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default()
    }
}

/// Without a provider the reading changes nothing.
fn previous_inboxes(state: &AppState) -> Vec<ProjectInbox> {
    state
        .worker
        .get()
        .map(|w| w.snapshot().inboxes)
        .unwrap_or_default()
}

fn worker_of(state: &AppState) -> Reply<Worker> {
    state
        .worker
        .get()
        .cloned()
        .ok_or_else(|| CommandError::new("messages", "the messages worker is not running"))
}

/// Starts the worker with the snapshot saved by the last run.
pub fn start_worker(app: &AppHandle) {
    let state = app.state::<AppState>();
    let file = state.config_dir.join(SNAPSHOT_FILE);
    let worker = Worker::start(ProviderPort::new(app.clone()), Some(file));
    if state.worker.set(worker).is_err() {
        tracing::warn!("the messages worker was already running");
    }
}

/// The snapshot the Messages view shows, at once (IMP-007). The first call after start
/// probes the provider behind it and asks the worker for a reading.
#[tauri::command]
pub async fn messages_snapshot(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Reply<MessagesSnapshot> {
    let worker = worker_of(&state)?;
    let snapshot = worker.snapshot();
    let first = !state
        .snapshot_asked
        .swap(true, std::sync::atomic::Ordering::AcqRel);
    if first {
        tauri::async_runtime::spawn(async move {
            let state = app.state::<AppState>();
            if provider(app.clone(), &state).await.is_ok() {
                worker.refresh();
            }
        });
    }
    Ok(snapshot)
}

/// "Check now": reads every project again and says what changed.
#[tauri::command]
pub async fn messages_refresh(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Reply<Vec<PollResult>> {
    provider(app, &state).await?;
    let worker = worker_of(&state)?;
    blocking(move || Ok(worker.refresh_and_wait())).await
}

#[tauri::command]
pub async fn messages_template(
    app: AppHandle,
    state: State<'_, AppState>,
    kind: MessageKind,
) -> Reply<Vec<TemplateSection>> {
    let provider = provider(app, &state).await?;
    let (store, source) = (Arc::clone(&state.settings), Arc::clone(&state.messages));
    blocking(move || {
        let current = settings::get(store.as_ref());
        messages::template(source.as_ref(), provider.as_ref(), &current, &kind)
            .map_err(CommandError::from)
    })
    .await
}

/// A write, after the ones asked before it; a reading follows (IMP-007).
#[tauri::command]
pub async fn messages_act(
    app: AppHandle,
    state: State<'_, AppState>,
    project: String,
    action: UserAction,
) -> Reply<Vec<MessageHeader>> {
    provider(app, &state).await?;
    let worker = worker_of(&state)?;
    blocking(move || worker.act(project, action)).await
}

/// The provider of the last probe, without probing.
fn last_provider(state: &AppState) -> Option<EngineReport> {
    state.engines.lock().ok().and_then(|last| {
        last.as_ref().and_then(|overview| {
            overview
                .reports
                .iter()
                .find(|r| r.kind == EngineKind::Integrations)
                .cloned()
        })
    })
}

/// Whether a reading is due, `elapsed` after the last one, with the window focused or
/// not; `None` interval means off.
pub fn due(interval: Option<Duration>, elapsed: Duration) -> bool {
    interval.is_some_and(|i| elapsed >= i)
}

/// How often the timer looks at the clock and the focus.
const TICK: Duration = Duration::from_secs(1);

/// Starts the timer: a reading every `messages_poll_seconds` while the window has the
/// focus, at least every minute while it does not (IMP-007).
pub fn start_timer(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last = Instant::now();
        loop {
            std::thread::sleep(TICK);
            let state = app.state::<AppState>();
            let focused = state.focused.load(std::sync::atomic::Ordering::Acquire);
            let interval = settings::get(state.settings.as_ref()).poll_interval(focused);
            if due(interval, last.elapsed()) {
                last = Instant::now();
                if let Some(worker) = state.worker.get() {
                    worker.refresh();
                }
            }
        }
    });
}

/// The least time between two readings started by the window getting the focus.
pub const FOCUS_INTERVAL: Duration = Duration::from_secs(20);

/// Whether a focus at `now` starts a reading, given when the last one started.
pub fn focus_check_due(last: Option<Instant>, now: Instant) -> bool {
    last.is_none_or(|last| now.saturating_duration_since(last) >= FOCUS_INTERVAL)
}

/// The window's focus changed: on getting it, the user may come back from a pull or a
/// checkout, so read again, at most once per `FOCUS_INTERVAL` and only while the
/// background updates are on (IMP-004, IMP-007).
pub fn on_focus(app: &AppHandle, focused: bool) {
    let state = app.state::<AppState>();
    state
        .focused
        .store(focused, std::sync::atomic::Ordering::Release);
    if !focused
        || settings::get(state.settings.as_ref())
            .poll_interval(true)
            .is_none()
    {
        return;
    }
    let now = Instant::now();
    let Ok(mut last) = state.last_focus_check.lock() else {
        return;
    };
    if !focus_check_due(*last, now) {
        return;
    }
    *last = Some(now);
    drop(last);
    if let Some(worker) = state.worker.get() {
        worker.refresh();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_timer_follows_the_setting_and_the_focus() {
        let s = Settings::default();
        assert!(!due(s.poll_interval(true), Duration::from_secs(19)));
        assert!(due(s.poll_interval(true), Duration::from_secs(20)));
        assert!(!due(s.poll_interval(false), Duration::from_secs(59)));
        assert!(due(s.poll_interval(false), Duration::from_secs(60)));
        let off = Settings {
            messages_poll_seconds: 0,
            ..Settings::default()
        };
        assert!(!due(off.poll_interval(true), Duration::from_secs(3600)));
    }

    #[test]
    fn the_focus_starts_one_reading_per_interval() {
        let start = Instant::now();
        assert!(focus_check_due(None, start));
        assert!(!focus_check_due(
            Some(start),
            start + Duration::from_secs(5)
        ));
        assert!(focus_check_due(Some(start), start + FOCUS_INTERVAL));
    }

    #[test]
    fn parallel_work_keeps_the_order() {
        let items: Vec<u32> = (0..10).collect();
        assert_eq!(
            in_parallel(&items, |n| n * 2),
            (0..10).map(|n| n * 2).collect::<Vec<_>>()
        );
    }
}
