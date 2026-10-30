//! Plain data shared by every crate and by the interface.

mod cycle;
mod engine;
mod feature;
mod project;
mod settings;

pub use cycle::{
    CandidateRun, CurrentRelease, CycleError, CycleState, ItemRef, NextOption, NextStep,
    PlanProgress, PullRequestState, UntriagedBug, UpdateSide,
};
pub use engine::{Capability, EngineKind, EngineReport, EngineSettings, EngineStatus};
pub use feature::{AvailabilityState, EnginesOverview, Feature, FeatureAvailability};
pub use project::{
    AddOutcome, Presence, Project, ProjectError, ProjectId, ProjectList, ProjectName,
    RegistryNotice,
};
pub use settings::{EnginesSettings, SETTINGS_VERSION, Settings, Theme, ThemeChoice};
