//! The Tauri commands, one per core use case (contracts/desktop-commands.md). Each runs
//! on the blocking pool so the window never waits on a file or an engine.

use std::path::PathBuf;
use std::sync::Arc;

use aurora_core::domain::{
    AddOutcome, CycleError, CycleState, EngineKind, EnginesOverview, Project, ProjectError,
    ProjectId, ProjectList, Settings,
};
use aurora_core::usecases::{cycle, projects, settings};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;

use crate::events::{self, ENGINE_REPORT, ENGINES_READY};
use crate::state::AppState;

/// The error every command returns: a kind for the interface and a message for the user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub kind: String,
    pub message: String,
    /// The cycle error itself, for the `cycle` kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<CycleError>,
}

impl CommandError {
    fn new(kind: &str, message: impl ToString) -> Self {
        Self {
            kind: kind.to_owned(),
            message: message.to_string(),
            detail: None,
        }
    }
}

impl From<CycleError> for CommandError {
    fn from(error: CycleError) -> Self {
        Self {
            detail: Some(error.clone()),
            ..Self::new("cycle", error)
        }
    }
}

impl From<ProjectError> for CommandError {
    fn from(error: ProjectError) -> Self {
        let kind = match &error {
            ProjectError::NotFound { .. } => "notFound",
            ProjectError::NotARepository { .. } => "notARepository",
            ProjectError::AlreadyRegistered { .. } => "alreadyRegistered",
            ProjectError::InvalidName { .. } => "invalidName",
            ProjectError::UnknownProject { .. } => "unknownProject",
            ProjectError::Storage { .. } => "storage",
        };
        Self::new(kind, error)
    }
}

type Reply<T> = Result<T, CommandError>;

async fn blocking<T: Send + 'static>(work: impl FnOnce() -> Reply<T> + Send + 'static) -> Reply<T> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|error| CommandError::new("internal", error))?
}

#[tauri::command]
pub async fn list_projects(state: State<'_, AppState>) -> Reply<ProjectList> {
    let registry = Arc::clone(&state.registry);
    blocking(move || Ok(projects::list(registry.as_ref()))).await
}

#[tauri::command]
pub async fn add_project(
    state: State<'_, AppState>,
    path: PathBuf,
    name: Option<String>,
) -> Reply<AddOutcome> {
    let registry = Arc::clone(&state.registry);
    blocking(move || Ok(projects::add(registry.as_ref(), &path, name.as_deref())?)).await
}

#[tauri::command]
pub async fn pick_and_add_project(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Reply<Option<AddOutcome>> {
    let registry = Arc::clone(&state.registry);
    blocking(move || {
        let Some(picked) = app
            .dialog()
            .file()
            .set_title("Add a project")
            .blocking_pick_folder()
        else {
            return Ok(None);
        };
        let path = picked
            .into_path()
            .map_err(|error| CommandError::new("notFound", error))?;
        Ok(Some(projects::add(registry.as_ref(), &path, None)?))
    })
    .await
}

#[tauri::command]
pub async fn rename_project(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Reply<Project> {
    let registry = Arc::clone(&state.registry);
    blocking(move || Ok(projects::rename(registry.as_ref(), &ProjectId(id), &name)?)).await
}

#[tauri::command]
pub async fn remove_project(state: State<'_, AppState>, id: String) -> Reply<()> {
    let registry = Arc::clone(&state.registry);
    blocking(move || Ok(projects::remove(registry.as_ref(), &ProjectId(id))?)).await
}

#[tauri::command]
pub async fn relocate_project(
    state: State<'_, AppState>,
    id: String,
    path: PathBuf,
) -> Reply<Project> {
    let registry = Arc::clone(&state.registry);
    blocking(move || {
        Ok(projects::relocate(
            registry.as_ref(),
            &ProjectId(id),
            &path,
        )?)
    })
    .await
}

/// The last overview; the first call probes the engines and waits for them.
#[tauri::command]
pub async fn engines_overview(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Reply<EnginesOverview> {
    if let Some(overview) = state.engines.lock().ok().and_then(|last| last.clone()) {
        return Ok(overview);
    }
    let (probe, store, last, first_probe) = (
        Arc::clone(&state.probe),
        Arc::clone(&state.settings),
        Arc::clone(&state.engines),
        Arc::clone(&state.first_probe),
    );
    blocking(move || {
        Ok(last_or_probe(&last, &first_probe, || {
            probe_and_publish(&app, probe.as_ref(), store.as_ref(), &last)
        }))
    })
    .await
}

/// The last overview, or the result of `probe` run once while the other callers wait for
/// it (spec 003 FR-009).
fn last_or_probe(
    last: &std::sync::Mutex<Option<EnginesOverview>>,
    first_probe: &std::sync::Mutex<()>,
    probe: impl FnOnce() -> EnginesOverview,
) -> EnginesOverview {
    let cached = || last.lock().ok().and_then(|last| last.clone());
    let _turn = first_probe
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    cached().unwrap_or_else(probe)
}

/// Probes again in the background; the interface follows the events.
#[tauri::command]
pub async fn refresh_engines(app: AppHandle, state: State<'_, AppState>) -> Reply<()> {
    let (probe, store, last) = (
        Arc::clone(&state.probe),
        Arc::clone(&state.settings),
        Arc::clone(&state.engines),
    );
    tauri::async_runtime::spawn_blocking(move || {
        probe_and_publish(&app, probe.as_ref(), store.as_ref(), &last);
    });
    Ok(())
}

fn probe_and_publish(
    app: &AppHandle,
    probe: &dyn aurora_core::ports::EngineProbe,
    store: &dyn aurora_core::ports::SettingsStore,
    last: &std::sync::Mutex<Option<EnginesOverview>>,
) -> EnginesOverview {
    let current = settings::get(store);
    let overview = events::probe_all(probe, &current, &|report| {
        if let Err(error) = app.emit(ENGINE_REPORT, report) {
            tracing::warn!(%error, "could not send an engine report");
        }
    });
    if let Ok(mut slot) = last.lock() {
        *slot = Some(overview.clone());
    }
    if let Err(error) = app.emit(ENGINES_READY, &overview) {
        tracing::warn!(%error, "could not send the engines overview");
    }
    overview
}

/// A project's development cycle state through `dcp`, using the last engine probe (the
/// first call probes and waits, as `engines_overview` does).
#[tauri::command]
pub async fn project_cycle_state(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Reply<CycleState> {
    let overview = engines_overview(app, state.clone()).await?;
    let (registry, store, source) = (
        Arc::clone(&state.registry),
        Arc::clone(&state.settings),
        Arc::clone(&state.cycle),
    );
    blocking(move || {
        let id = ProjectId(id);
        let project = projects::list(registry.as_ref())
            .projects
            .into_iter()
            .find(|project| project.id == id)
            .ok_or_else(|| ProjectError::UnknownProject { id: id.0.clone() })?;
        let dcp = overview
            .reports
            .iter()
            .find(|report| report.kind == EngineKind::DevCycle);
        let current = settings::get(store.as_ref());
        cycle::cycle_state(source.as_ref(), dcp, &current, &project).map_err(CommandError::from)
    })
    .await
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Reply<Settings> {
    let store = Arc::clone(&state.settings);
    blocking(move || Ok(settings::get(store.as_ref()))).await
}

#[tauri::command]
pub async fn save_settings(state: State<'_, AppState>, settings: Settings) -> Reply<Settings> {
    let store = Arc::clone(&state.settings);
    blocking(move || {
        self::settings::save(store.as_ref(), settings)
            .map_err(|error| CommandError::new("storage", error))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_errors_keep_their_kind_and_message() {
        let error = CommandError::from(ProjectError::AlreadyRegistered {
            name: "gedo".into(),
        });
        assert_eq!(error.kind, "alreadyRegistered");
        assert!(error.message.contains("gedo"));
        assert_eq!(error.detail, None);
    }

    #[test]
    fn concurrent_first_calls_probe_once() {
        use std::sync::Mutex;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let (last, first_probe, probes) = (Mutex::new(None), Mutex::new(()), AtomicUsize::new(0));
        std::thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| {
                    last_or_probe(&last, &first_probe, || {
                        probes.fetch_add(1, Ordering::SeqCst);
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        let overview = aurora_core::fixtures::sample_overview();
                        *last.lock().expect("lock") = Some(overview.clone());
                        overview
                    })
                });
            }
        });
        assert_eq!(probes.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn cycle_errors_keep_the_error_as_detail() {
        let cause = CycleError::TimedOut {
            timeout_ms: 10_000,
            stderr: String::new(),
        };
        let error = CommandError::from(cause.clone());
        assert_eq!(error.kind, "cycle");
        assert_eq!(error.message, cause.to_string());
        let json = serde_json::to_value(&error).expect("serializes");
        assert_eq!(json["detail"]["kind"], "timedOut");
    }
}
