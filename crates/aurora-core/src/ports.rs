//! The ports the use cases need (ADR-0001). Adapters in `aurora-engines` implement them
//! for real; the `fixtures` feature implements them in memory.

use std::path::Path;

use crate::domain::{
    AddOutcome, ClosingRecord, ConfigChange, ConfigError, ConfigOutcome, Conversation, CycleError,
    CycleState, EngineKind, EngineReport, EngineSettings, Message, MessageHeader, MessageKind,
    MessagesError, Project, ProjectConfig, ProjectError, ProjectId, ProjectList, ProjectName,
    Settings, TemplateSection, UserAction,
};

/// The list of projects the user follows (ADR-0004).
pub trait ProjectRegistry: Send + Sync {
    fn load(&self) -> ProjectList;
    fn add(&self, path: &Path, name: Option<ProjectName>) -> Result<AddOutcome, ProjectError>;
    fn rename(&self, id: &ProjectId, name: ProjectName) -> Result<Project, ProjectError>;
    fn remove(&self, id: &ProjectId) -> Result<(), ProjectError>;
    fn relocate(&self, id: &ProjectId, path: &Path) -> Result<Project, ProjectError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("the settings could not be saved: {message}")]
pub struct SettingsError {
    pub message: String,
}

/// The user's settings.
pub trait SettingsStore: Send + Sync {
    fn load(&self) -> Settings;
    fn save(&self, settings: &Settings) -> Result<(), SettingsError>;
}

/// Finds an engine and asks it for its version and capabilities (ADR-0003).
pub trait EngineProbe: Send + Sync {
    fn probe(&self, kind: EngineKind, settings: &EngineSettings) -> EngineReport;
}

/// Reads a project's development cycle state from the Dev Cycle Protocol (ADR-0003,
/// ADR-0006). `engine` is the latest report of `dcp`, which gives its location.
pub trait CycleSource: Send + Sync {
    fn cycle_state(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
    ) -> Result<CycleState, CycleError>;
}

/// Reads and changes a project's Dev Cycle settings through the Dev Cycle Protocol
/// (FEAT-027, ADR-0006). `engine` is the latest report of `dcp`, which gives its location.
pub trait ConfigSource: Send + Sync {
    fn read(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
    ) -> Result<ProjectConfig, ConfigError>;

    /// Previews (`dry_run`) or applies `change`. A reply with refusals is
    /// `Err(ConfigError::from_field_errors(..))`.
    fn set(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
        change: &ConfigChange,
        dry_run: bool,
    ) -> Result<ConfigOutcome, ConfigError>;
}

/// What one project's inbox holds, as `dcp-integrations check` and `projects` answer it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboxRead {
    pub slug: String,
    pub counterparts: Vec<String>,
    /// Every project the registry lists, this one included, with its repository URL.
    pub listed: Vec<ListedProject>,
    pub conversations: Vec<Conversation>,
}

/// A project as one registry lists it: the slug it is known by and its repository URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedProject {
    pub slug: String,
    pub url: String,
}

/// Reads and writes messages between projects through `dcp-integrations` (FEAT-031,
/// ADR-0010). `engine` is the latest report of the provider, which gives its location;
/// `root` is the project's folder, with its `.aurora/integrations.json`. Aurora itself
/// runs no Git.
pub trait MessagesSource: Send + Sync {
    fn inbox(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
    ) -> Result<InboxRead, MessagesError>;

    /// Every message of a conversation, oldest first.
    fn thread(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
        thread: &str,
    ) -> Result<Vec<Message>, MessagesError>;

    fn record(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
        thread: &str,
    ) -> Result<ClosingRecord, MessagesError>;

    fn template(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        kind: &MessageKind,
    ) -> Result<Vec<TemplateSection>, MessagesError>;

    /// Runs one action; the messages sent or changed.
    fn act(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
        action: &UserAction,
    ) -> Result<Vec<MessageHeader>, MessagesError>;

    /// Whether the inbox changed since the last poll, without fetching; a summary when
    /// it did.
    fn poll(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
    ) -> Result<Option<String>, MessagesError>;

    /// Brings the local copies of the project's inboxes up to date, so the reads above
    /// answer from them without the network (ADR-0011); whether anything changed. A
    /// provider that reads from the network on every call has nothing to do.
    fn sync(
        &self,
        _engine: &EngineReport,
        _settings: &EngineSettings,
        _root: &Path,
    ) -> Result<bool, MessagesError> {
        Ok(false)
    }
}
