//! The commands of the Messages view and the background poll (spec 008,
//! contracts/desktop-commands.md). Every provider call runs on the blocking pool, at
//! most four at once.

use std::sync::Arc;
use std::time::Duration;

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
    blocking(move || {
        let list = projects::list(registry.as_ref()).projects;
        let current = settings::get(store.as_ref());
        Ok(overview_of(
            source.as_ref(),
            provider.as_ref(),
            &current,
            &list,
        ))
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
    in_parallel(&list, |project| {
        messages::poll(
            state.messages.as_ref(),
            provider,
            &current,
            std::slice::from_ref(project),
        )
    })
    .into_iter()
    .flatten()
    .collect()
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
    settings
        .messages_poll_minutes
        .filter(|minutes| *minutes > 0)
        .map(|minutes| Duration::from_secs(u64::from(minutes.min(24 * 60)) * 60))
}

/// The results worth an event: only the projects whose inbox changed.
pub fn news(results: Vec<PollResult>) -> Option<Vec<PollResult>> {
    let changed: Vec<PollResult> = results.into_iter().filter(|r| r.changed).collect();
    (!changed.is_empty()).then_some(changed)
}

/// While the setting is off it checks again every minute.
const OFF_CHECK: Duration = Duration::from_secs(60);

/// Starts the background poll: every `messages_poll_minutes`, with the provider of the
/// last probe, it emits `messages-news` with the projects whose inbox changed.
pub fn start_poll(app: AppHandle) {
    std::thread::spawn(move || {
        loop {
            let state = app.state::<AppState>();
            let wait = next_wait(&settings::get(state.settings.as_ref()));
            std::thread::sleep(wait.unwrap_or(OFF_CHECK));
            if wait.is_none() {
                continue;
            }
            let provider = state.engines.lock().ok().and_then(|last| {
                last.as_ref().and_then(|overview| {
                    overview
                        .reports
                        .iter()
                        .find(|r| r.kind == EngineKind::Integrations)
                        .cloned()
                })
            });
            if provider.is_none() {
                continue;
            }
            if let Some(changed) = news(poll_all(&state, provider.as_ref())) {
                tracing::info!(projects = changed.len(), "messages news");
                if let Err(error) = app.emit(MESSAGES_NEWS, &changed) {
                    tracing::warn!(%error, "could not emit messages-news");
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(id: &str, changed: bool) -> PollResult {
        PollResult {
            project: ProjectId(id.into()),
            changed,
            summary: String::new(),
        }
    }

    #[test]
    fn the_timer_follows_the_setting_and_emits_only_changes() {
        let mut s = Settings::default();
        assert_eq!(next_wait(&s), Some(Duration::from_secs(300)));
        s.messages_poll_minutes = None;
        assert_eq!(next_wait(&s), None);
        s.messages_poll_minutes = Some(0);
        assert_eq!(next_wait(&s), None);
        assert_eq!(news(vec![result("a", false)]), None);
        assert_eq!(
            news(vec![result("a", false), result("b", true)]),
            Some(vec![result("b", true)])
        );
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
