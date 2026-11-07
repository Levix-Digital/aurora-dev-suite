//! How the interface reaches the use cases: the desktop through Tauri, or the core's
//! fixtures in the browser preview (research R1, R2). Both answer with the core's types.

mod fixture;
mod tauri;

use std::path::PathBuf;
use std::sync::Arc;

use aurora_core::domain::{
    AddOutcome, ConfigChange, ConfigError, ConfigOutcome, CycleError, CycleState, EngineReport,
    EnginesOverview, MessageHeader, MessageKind, MessagesError, MessagesNews, MessagesSnapshot,
    PollResult, Project, ProjectConfig, ProjectList, Settings, TemplateSection, UserAction,
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
    /// The settings error itself, for the `config` kind.
    #[serde(default)]
    pub config: Option<Box<ConfigError>>,
    /// The messages error itself, for the `messages` kind.
    #[serde(default)]
    pub messages: Option<Box<MessagesError>>,
}

impl CommandError {
    pub fn new(kind: &str, message: impl ToString) -> Self {
        Self {
            kind: kind.to_owned(),
            message: message.to_string(),
            detail: None,
            config: None,
            messages: None,
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

impl From<ConfigError> for CommandError {
    fn from(error: ConfigError) -> Self {
        Self {
            config: Some(Box::new(error.clone())),
            ..Self::new("config", error)
        }
    }
}

impl From<MessagesError> for CommandError {
    fn from(error: MessagesError) -> Self {
        Self {
            messages: Some(Box::new(error.clone())),
            ..Self::new("messages", error)
        }
    }
}

pub type Reply<T> = Result<T, CommandError>;
pub type OnNews = Arc<dyn Fn(MessagesNews) + Send + Sync>;

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

    /// A project's Dev Cycle settings through `dcp config get` (FEAT-027).
    pub async fn project_config(&self, id: String) -> Reply<ProjectConfig> {
        match self {
            Backend::Tauri => tauri::invoke("project_config", &tauri::IdArgs { id }).await,
            Backend::Fixture(fixture) => fixture.project_config(&id),
        }
    }

    /// What `change` would do; nothing is written.
    pub async fn preview_project_config(
        &self,
        id: String,
        change: ConfigChange,
    ) -> Reply<ConfigOutcome> {
        match self {
            Backend::Tauri => {
                tauri::invoke("preview_project_config", &tauri::ChangeArgs { id, change }).await
            }
            Backend::Fixture(fixture) => fixture.set_project_config(&id, &change, true),
        }
    }

    /// Writes `change` through `dcp config set`.
    pub async fn apply_project_config(
        &self,
        id: String,
        change: ConfigChange,
    ) -> Reply<ConfigOutcome> {
        match self {
            Backend::Tauri => {
                tauri::invoke("apply_project_config", &tauri::ChangeArgs { id, change }).await
            }
            Backend::Fixture(fixture) => fixture.set_project_config(&id, &change, false),
        }
    }

    /// Follows `messages-snapshot`: a new snapshot and what the navigation shows
    /// (spec 010, IMP-007).
    pub fn subscribe_messages(&self, on_news: OnNews) {
        match self {
            Backend::Tauri => tauri::subscribe_messages(on_news),
            Backend::Fixture(fixture) => FixtureBackend::subscribe_messages(fixture, on_news),
        }
    }

    /// The worker's snapshot, at once.
    pub async fn messages_snapshot(&self) -> Reply<MessagesSnapshot> {
        match self {
            Backend::Tauri => tauri::invoke("messages_snapshot", &()).await,
            Backend::Fixture(fixture) => Ok(fixture.messages_snapshot()),
        }
    }

    /// "Check now": reads every project again; the projects that changed.
    pub async fn messages_refresh(&self) -> Reply<Vec<PollResult>> {
        match self {
            Backend::Tauri => tauri::invoke("messages_refresh", &()).await,
            Backend::Fixture(fixture) => Ok(fixture.messages_refresh()),
        }
    }

    pub async fn messages_template(&self, kind: MessageKind) -> Reply<Vec<TemplateSection>> {
        match self {
            Backend::Tauri => tauri::invoke("messages_template", &tauri::KindArgs { kind }).await,
            Backend::Fixture(fixture) => fixture.messages_template(&kind),
        }
    }

    pub async fn messages_act(
        &self,
        project: String,
        action: UserAction,
    ) -> Reply<Vec<MessageHeader>> {
        match self {
            Backend::Tauri => {
                tauri::invoke("messages_act", &tauri::ActArgs { project, action }).await
            }
            Backend::Fixture(fixture) => fixture.messages_act(&project, &action),
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
