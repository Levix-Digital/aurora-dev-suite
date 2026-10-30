//! The ports the use cases need (ADR-0001). Adapters in `aurora-engines` implement them
//! for real; the `fixtures` feature implements them in memory.

use std::path::Path;

use crate::domain::{
    AddOutcome, CycleError, CycleState, EngineKind, EngineReport, EngineSettings, Project,
    ProjectError, ProjectId, ProjectList, ProjectName, Settings,
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
