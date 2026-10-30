//! How the interface reaches the use cases: the desktop through Tauri, or the core's
//! fixtures in the browser preview (research R1, R2). Both answer with the core's types.

mod fixture;
mod tauri;

use std::path::PathBuf;
use std::sync::Arc;

use aurora_core::domain::{
    AddOutcome, CycleError, CycleState, EngineReport, EnginesOverview, Project, ProjectList,
    Settings,
};
use serde::Deserialize;

pub use fixture::FixtureBackend;

/// The error of a command (contracts/desktop-commands.md).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CommandError {
    pub kind: String,
    pub message: String,
    /// The cycle error itself, for the `cycle` kind.
    #[serde(default)]
    pub detail: Option<CycleError>,
}

impl CommandError {
    pub fn new(kind: &str, message: impl ToString) -> Self {
        Self {
            kind: kind.to_owned(),
            message: message.to_string(),
            detail: None,
        }
    }

    pub fn internal(message: impl ToString) -> Self {
        Self::new("internal", message)
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

pub type Reply<T> = Result<T, CommandError>;

pub type OnReport = Arc<dyn Fn(EngineReport) + Send + Sync>;
pub type OnReady = Arc<dyn Fn(EnginesOverview) + Send + Sync>;

#[derive(Clone)]
pub enum Backend {
    Tauri,
    Fixture(Arc<FixtureBackend>),
}

impl Backend {
    /// Tauri inside the desktop application, the fixtures anywhere else.
    pub fn detect() -> Self {
        if tauri::available() {
            Backend::Tauri
        } else {
            Backend::Fixture(Arc::new(FixtureBackend::sample()))
        }
    }

    pub fn is_preview(&self) -> bool {
        matches!(self, Backend::Fixture(_))
    }

    /// Follows the engine events: each report as it arrives, then the overview.
    pub fn subscribe_engines(&self, on_report: OnReport, on_ready: OnReady) {
        match self {
            Backend::Tauri => tauri::subscribe_engines(on_report, on_ready),
            Backend::Fixture(fixture) => fixture.subscribe_engines(on_report, on_ready),
        }
    }

    pub async fn list_projects(&self) -> Reply<ProjectList> {
        match self {
            Backend::Tauri => tauri::invoke("list_projects", &()).await,
            Backend::Fixture(fixture) => Ok(fixture.list_projects()),
        }
    }

    pub async fn add_project(&self, path: PathBuf, name: Option<String>) -> Reply<AddOutcome> {
        match self {
            Backend::Tauri => tauri::invoke("add_project", &tauri::AddArgs { path, name }).await,
            Backend::Fixture(fixture) => fixture.add_project(&path, name.as_deref()),
        }
    }

    pub async fn pick_and_add_project(&self) -> Reply<Option<AddOutcome>> {
        match self {
            Backend::Tauri => tauri::invoke("pick_and_add_project", &()).await,
            Backend::Fixture(fixture) => fixture.pick_and_add_project(),
        }
    }

    pub async fn rename_project(&self, id: String, name: String) -> Reply<Project> {
        match self {
            Backend::Tauri => {
                tauri::invoke("rename_project", &tauri::RenameArgs { id, name }).await
            }
            Backend::Fixture(fixture) => fixture.rename_project(&id, &name),
        }
    }

    pub async fn remove_project(&self, id: String) -> Reply<()> {
        match self {
            Backend::Tauri => tauri::invoke("remove_project", &tauri::IdArgs { id }).await,
            Backend::Fixture(fixture) => fixture.remove_project(&id),
        }
    }

    pub async fn relocate_project(&self, id: String, path: PathBuf) -> Reply<Project> {
        match self {
            Backend::Tauri => {
                tauri::invoke("relocate_project", &tauri::RelocateArgs { id, path }).await
            }
            Backend::Fixture(fixture) => fixture.relocate_project(&id, &path),
        }
    }

    pub async fn engines_overview(&self) -> Reply<EnginesOverview> {
        match self {
            Backend::Tauri => tauri::invoke("engines_overview", &()).await,
            Backend::Fixture(fixture) => Ok(fixture.engines_overview()),
        }
    }

    pub async fn refresh_engines(&self) -> Reply<()> {
        match self {
            Backend::Tauri => tauri::invoke("refresh_engines", &()).await,
            Backend::Fixture(fixture) => {
                fixture.refresh_engines();
                Ok(())
            }
        }
    }

    /// A project's development cycle state through `dcp` (FEAT-016).
    pub async fn project_cycle_state(&self, id: String) -> Reply<CycleState> {
        match self {
            Backend::Tauri => tauri::invoke("project_cycle_state", &tauri::IdArgs { id }).await,
            Backend::Fixture(fixture) => fixture.project_cycle_state(&id),
        }
    }

    pub async fn get_settings(&self) -> Reply<Settings> {
        match self {
            Backend::Tauri => tauri::invoke("get_settings", &()).await,
            Backend::Fixture(fixture) => Ok(fixture.get_settings()),
        }
    }

    pub async fn save_settings(&self, settings: Settings) -> Reply<Settings> {
        match self {
            Backend::Tauri => {
                tauri::invoke("save_settings", &tauri::SettingsArgs { settings }).await
            }
            Backend::Fixture(fixture) => fixture.save_settings(settings),
        }
    }
}
