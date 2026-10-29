use serde::{Deserialize, Serialize};

use super::{EngineKind, EngineSettings};

pub const SETTINGS_VERSION: u32 = 1;

/// The theme the user chose.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemeChoice {
    #[default]
    System,
    Light,
    Dark,
}

/// The theme in effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    Light,
    Dark,
}

impl ThemeChoice {
    /// The theme in effect, given whether the operating system prefers dark.
    pub fn resolve(self, system_prefers_dark: bool) -> Theme {
        match self {
            ThemeChoice::Light => Theme::Light,
            ThemeChoice::Dark => Theme::Dark,
            ThemeChoice::System if system_prefers_dark => Theme::Dark,
            ThemeChoice::System => Theme::Light,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EnginesSettings {
    pub dev_cycle: EngineSettings,
    pub gedo: EngineSettings,
    pub any_context: EngineSettings,
}

impl EnginesSettings {
    pub fn get(&self, kind: EngineKind) -> &EngineSettings {
        match kind {
            EngineKind::DevCycle => &self.dev_cycle,
            EngineKind::Gedo => &self.gedo,
            EngineKind::AnyContext => &self.any_context,
        }
    }
}

/// The user's settings (`settings.toml`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub version: u32,
    pub theme: ThemeChoice,
    pub engines: EnginesSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            theme: ThemeChoice::System,
            engines: EnginesSettings::default(),
        }
    }
}
