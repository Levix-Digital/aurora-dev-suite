//! The commands of the Messages view and the background poll (spec 008,
//! contracts/desktop-commands.md). Every provider call runs on the blocking pool, at
//! most four at once.

use std::sync::Arc;
use std::time::{Duration, Instant};

use aurora_core::domain::{
    ClosingRecord, EngineKind, EngineReport, Message, MessageHeader, MessageKind, PollResult,
    Project, ProjectId, ProjectInbox, Settings, TemplateSection, UserAction,
};
use aurora_core::ports::MessagesSource;
use aurora_core::usecases::{messages, projects, settings};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::commands::{CommandError, Reply, blocking, engines_overview};
use crate::state::AppState;

pub const MESSAGES_NEWS: &str = "messages-news";
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

fn overview_of(
    source: &dyn MessagesSource,
    provider: Option<&EngineReport>,
    current: &Settings,
    list: &[Project],
) -> Vec<ProjectInbox> {
    in_parallel(list, |project| {
        messages::project_inbox(source, provider, current, project)
    })
}

#[tauri::command]
pub async fn messages_overview(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Reply<Vec<ProjectInbox>> {
    let provider = provider(app, &state).await?;
    let (registry, store, source) = (
        Arc::clone(&state.registry),
        Arc::clone(&state.settings),
        Arc::clone(&state.messages),
    );
    let setups = Arc::clone(&state.setups);
    blocking(move || {
        let list = projects::list(registry.as_ref()).projects;
        let current = settings::get(store.as_ref());
        let inboxes = overview_of(source.as_ref(), provider.as_ref(), &current, &list);
        if let Ok(mut memory) = setups.lock() {
            memory.remember(&inboxes);
        }
        Ok(inboxes)
    })
    .await
}

#[tauri::command]
pub async fn messages_conversation(
    app: AppHandle,
    state: State<'_, AppState>,
    project: String,
    thread: String,
) -> Reply<Vec<Message>> {
    let provider = provider(app, &state).await?;
    let project = project_of(&state, &project)?;
    let (store, source) = (Arc::clone(&state.settings), Arc::clone(&state.messages));
    blocking(move || {
        let current = settings::get(store.as_ref());
        messages::conversation(
            source.as_ref(),
            provider.as_ref(),
            &current,
            &project,
            &thread,
        )
        .map_err(CommandError::from)
    })
    .await
}

#[tauri::command]
pub async fn messages_record(
    app: AppHandle,
    state: State<'_, AppState>,
    project: String,
    thread: String,
) -> Reply<ClosingRecord> {
    let provider = provider(app, &state).await?;
    let project = project_of(&state, &project)?;
    let (store, source) = (Arc::clone(&state.settings), Arc::clone(&state.messages));
    blocking(move || {
        let current = settings::get(store.as_ref());
        messages::record(
            source.as_ref(),
            provider.as_ref(),
            &current,
            &project,
            &thread,
        )
        .map_err(CommandError::from)
    })
    .await
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

#[tauri::command]
pub async fn messages_act(
    app: AppHandle,
    state: State<'_, AppState>,
    project: String,
    action: UserAction,
) -> Reply<Vec<MessageHeader>> {
    let provider = provider(app, &state).await?;
    let project = project_of(&state, &project)?;
    let (store, source) = (Arc::clone(&state.settings), Arc::clone(&state.messages));
    blocking(move || {
        let current = settings::get(store.as_ref());
        messages::act(
            source.as_ref(),
            provider.as_ref(),
            &current,
            &project,
            &action,
        )
        .map_err(CommandError::from)
    })
    .await
}

fn poll_all(state: &AppState, provider: Option<&EngineReport>) -> Vec<PollResult> {
    let list = projects::list(state.registry.as_ref()).projects;
    let current = settings::get(state.settings.as_ref());
    let mut results: Vec<PollResult> = in_parallel(&list, |project| {
        messages::poll(
            state.messages.as_ref(),
            provider,
            &current,
            std::slice::from_ref(project),
        )
    })
    .into_iter()
    .flatten()
    .collect();
    if let Ok(mut memory) = state.setups.lock() {
        memory.note(&mut results);
    }
    results
}

#[tauri::command]
pub async fn messages_poll(app: AppHandle, state: State<'_, AppState>) -> Reply<Vec<PollResult>> {
    let provider = provider(app.clone(), &state).await?;
    blocking(move || {
        let state = app.state::<AppState>();
        Ok(poll_all(&state, provider.as_ref()))
    })
    .await
}

/// How long to wait before the next background poll; `None` when it is off.
pub fn next_wait(settings: &Settings) -> Option<Duration> {
    let minutes = settings.messages_poll_minutes.min(24 * 60);
    (minutes > 0).then(|| Duration::from_secs(u64::from(minutes) * 60))
}

/// The results worth an event: only the projects whose inbox changed.
pub fn news(results: Vec<PollResult>) -> Option<Vec<PollResult>> {
    let changed: Vec<PollResult> = results.into_iter().filter(|r| r.changed).collect();
    (!changed.is_empty()).then_some(changed)
}

/// While the setting is off it checks again every minute.
const OFF_CHECK: Duration = Duration::from_secs(60);

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

/// Polls every project and emits `messages-news` with the ones that changed: new
/// messages, or a setup that appeared or went away (IMP-004).
fn poll_and_emit(app: &AppHandle) {
    let state = app.state::<AppState>();
    let Some(provider) = last_provider(&state) else {
        return;
    };
    if let Some(changed) = news(poll_all(&state, Some(&provider))) {
        tracing::info!(projects = changed.len(), "messages news");
        if let Err(error) = app.emit(MESSAGES_NEWS, &changed) {
            tracing::warn!(%error, "could not emit messages-news");
        }
    }
}

/// Starts the background poll: every `messages_poll_minutes`, with the provider of the
/// last probe, it emits `messages-news` with the projects whose inbox or setup changed.
pub fn start_poll(app: AppHandle) {
    std::thread::spawn(move || {
        loop {
            let state = app.state::<AppState>();
            let wait = next_wait(&settings::get(state.settings.as_ref()));
            std::thread::sleep(wait.unwrap_or(OFF_CHECK));
            if wait.is_some() {
                poll_and_emit(&app);
            }
        }
    });
}

/// The least time between two checks started by the window getting the focus.
pub const FOCUS_INTERVAL: Duration = Duration::from_secs(30);

/// Whether a focus at `now` starts a check, given when the last one started.
pub fn focus_check_due(last: Option<Instant>, now: Instant) -> bool {
    last.is_none_or(|last| now.saturating_duration_since(last) >= FOCUS_INTERVAL)
}

/// The window got the focus: the user may come back from a pull or a checkout, so
/// check the projects again, at most once per `FOCUS_INTERVAL` and only while the
/// background poll is on (IMP-004).
pub fn on_focus(app: &AppHandle) {
    let state = app.state::<AppState>();
    if next_wait(&settings::get(state.settings.as_ref())).is_none() {
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
    let app = app.clone();
    std::thread::spawn(move || poll_and_emit(&app));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(id: &str, changed: bool) -> PollResult {
        PollResult {
            project: ProjectId(id.into()),
            changed,
            summary: String::new(),
            set_up: true,
        }
    }

    #[test]
    fn the_timer_follows_the_setting_and_emits_only_changes() {
        let mut s = Settings::default();
        assert_eq!(next_wait(&s), Some(Duration::from_secs(300)));
        s.messages_poll_minutes = 0;
        assert_eq!(next_wait(&s), None);
        s.messages_poll_minutes = 100_000;
        assert_eq!(next_wait(&s), Some(Duration::from_secs(24 * 3600)));
        assert_eq!(news(vec![result("a", false)]), None);
        assert_eq!(
            news(vec![result("a", false), result("b", true)]),
            Some(vec![result("b", true)])
        );
    }

    #[test]
    fn the_focus_starts_one_check_per_interval() {
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

    #[test]
    fn the_overview_uses_the_source_for_every_project() {
        use aurora_core::domain::{InboxSetup, Presence, ProjectName};
        use aurora_core::fixtures::{FixtureMessages, ready_report};
        let source = FixtureMessages::sample();
        let provider = ready_report(EngineKind::Integrations, &["messages"]);
        let list: Vec<Project> = ["aurora-dev-suite", "gedo"]
            .iter()
            .map(|name| Project {
                id: ProjectId((*name).into()),
                name: ProjectName::parse(name).expect("valid"),
                root: std::path::Path::new("/repos").join(name),
                added_at: String::new(),
                presence: Presence::Present,
            })
            .collect();
        let inboxes = overview_of(&source, Some(&provider), &Settings::default(), &list);
        assert_eq!(inboxes[0].setup, InboxSetup::Ready);
        assert!(matches!(inboxes[1].setup, InboxSetup::NotSetUp { .. }));
    }
}
