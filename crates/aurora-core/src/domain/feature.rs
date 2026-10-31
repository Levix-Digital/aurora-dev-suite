use serde::{Deserialize, Serialize};

use super::{EngineKind, EngineReport};

/// The features of Aurora that depend on an engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Feature {
    Portfolio,
    ProjectView,
    Inbox,
    ProjectSettings,
    Agents,
    Knowledge,
}

impl Feature {
    pub const ALL: [Feature; 6] = [
        Feature::Portfolio,
        Feature::ProjectView,
        Feature::Inbox,
        Feature::ProjectSettings,
        Feature::Agents,
        Feature::Knowledge,
    ];

    pub const fn display_name(self) -> &'static str {
        match self {
            Feature::Portfolio => "Portfolio",
            Feature::ProjectView => "Project view",
            Feature::Inbox => "Decision inbox",
            Feature::ProjectSettings => "Project settings",
            Feature::Agents => "Agent work",
            Feature::Knowledge => "Knowledge",
        }
    }
}

/// Whether a feature can run, and if not, why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum AvailabilityState {
    Available,
    Unavailable {
        engine: EngineKind,
        missing: Vec<String>,
        hint: String,
    },
    NotInThisRelease {
        release: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureAvailability {
    pub feature: Feature,
    pub state: AvailabilityState,
}

/// The engines and what they enable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnginesOverview {
    pub reports: Vec<EngineReport>,
    pub features: Vec<FeatureAvailability>,
}
