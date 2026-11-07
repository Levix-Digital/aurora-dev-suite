use serde::{Deserialize, Serialize};

use super::{EngineKind, EngineSettings};

pub const SETTINGS_VERSION: u32 = 3;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EnginesSettings {
    pub dev_cycle: EngineSettings,
    pub gedo: EngineSettings,
    pub any_context: EngineSettings,
    pub integrations: EngineSettings,
}

impl EnginesSettings {
    /// The provider's first read of a project fetches every counterpart's inbox branch,
    /// which takes longer than the other engines' answers (BUG-008).
    pub const INTEGRATIONS_TIMEOUT_MS: u64 = 60_000;
}

impl Default for EnginesSettings {
    fn default() -> Self {
        Self {
            dev_cycle: EngineSettings::default(),
            gedo: EngineSettings::default(),
            any_context: EngineSettings::default(),
            integrations: EngineSettings {
                path: None,
                timeout_ms: Self::INTEGRATIONS_TIMEOUT_MS,
            },
        }
    }
}

impl EnginesSettings {
    pub fn get(&self, kind: EngineKind) -> &EngineSettings {
        match kind {
            EngineKind::DevCycle => &self.dev_cycle,
            EngineKind::Gedo => &self.gedo,
            EngineKind::AnyContext => &self.any_context,
            EngineKind::Integrations => &self.integrations,
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
    /// Seconds between two background updates of the messages while Aurora's window has
    /// the focus; at least [`Settings::BACKGROUND_POLL_SECONDS`] while it does not; 0
    /// turns them off (IMP-007).
    pub messages_poll_seconds: u32,
    /// Version 2 kept the interval in minutes; read once by [`Settings::upgraded`].
    #[serde(skip_serializing)]
    pub messages_poll_minutes: Option<u32>,
}

impl Settings {
    pub const DEFAULT_POLL_SECONDS: u32 = 20;
    /// The least interval, so a typo cannot flood the inboxes' hosts.
    pub const MIN_POLL_SECONDS: u32 = 10;
    /// The interval while the window does not have the focus, at least.
    pub const BACKGROUND_POLL_SECONDS: u32 = 60;

    /// The interval of the background updates, `None` when they are off.
    pub fn poll_interval(&self, focused: bool) -> Option<std::time::Duration> {
        let seconds = self.messages_poll_seconds;
        if seconds == 0 {
            return None;
        }
        let seconds = seconds.clamp(Self::MIN_POLL_SECONDS, 24 * 60 * 60);
        let seconds = if focused {
            seconds
        } else {
            seconds.max(Self::BACKGROUND_POLL_SECONDS)
        };
        Some(std::time::Duration::from_secs(u64::from(seconds)))
    }

    /// Brings settings saved by an older Aurora to this version: version 1 kept the
    /// provider at the general 10 s limit, raised to its own default (BUG-008); a
    /// limit the user changed stays.
    pub fn upgraded(mut self) -> Self {
        if self.version < 2
            && self.engines.integrations.timeout_ms == EngineSettings::DEFAULT_TIMEOUT_MS
        {
            self.engines.integrations.timeout_ms = EnginesSettings::INTEGRATIONS_TIMEOUT_MS;
        }
        // Version 2: minutes; 1 was the default and becomes the new default (IMP-007).
        if let Some(minutes) = self.messages_poll_minutes.take()
            && self.version < 3
        {
            self.messages_poll_seconds = match minutes {
                0 => 0,
                1 => Self::DEFAULT_POLL_SECONDS,
                m => m.saturating_mul(60),
            };
        }
        self.version = SETTINGS_VERSION;
        self
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            theme: ThemeChoice::System,
            engines: EnginesSettings::default(),
            messages_poll_seconds: Settings::DEFAULT_POLL_SECONDS,
            messages_poll_minutes: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_poll_interval_adapts_to_the_focus_and_old_minutes_convert() {
        // IMP-007: 20 s with the focus, 1 min without (Levi, 2026-11-07).
        let s = Settings::default();
        assert_eq!(
            s.poll_interval(true),
            Some(std::time::Duration::from_secs(20))
        );
        assert_eq!(
            s.poll_interval(false),
            Some(std::time::Duration::from_secs(60))
        );
        let off = Settings {
            messages_poll_seconds: 0,
            ..Settings::default()
        };
        assert_eq!(off.poll_interval(true), None);
        let tiny = Settings {
            messages_poll_seconds: 1,
            ..Settings::default()
        };
        assert_eq!(
            tiny.poll_interval(true),
            Some(std::time::Duration::from_secs(10))
        );
        let old = |minutes: u32| {
            Settings {
                version: 2,
                messages_poll_seconds: Settings::DEFAULT_POLL_SECONDS,
                messages_poll_minutes: Some(minutes),
                ..Settings::default()
            }
            .upgraded()
            .messages_poll_seconds
        };
        assert_eq!((old(0), old(1), old(5)), (0, 20, 300));
    }

    #[test]
    fn the_provider_waits_longer_and_old_settings_are_raised() {
        // BUG-008: 10 s is too short for the provider's first read of a project.
        assert_eq!(
            Settings::default().engines.integrations.timeout_ms,
            EnginesSettings::INTEGRATIONS_TIMEOUT_MS
        );
        let saved = |version: u32, timeout_ms: u64| {
            let mut engines = EnginesSettings::default();
            engines.integrations.timeout_ms = timeout_ms;
            Settings {
                version,
                engines,
                ..Settings::default()
            }
        };
        let up = saved(1, EngineSettings::DEFAULT_TIMEOUT_MS).upgraded();
        assert_eq!(up.version, SETTINGS_VERSION);
        assert_eq!(
            up.engines.integrations.timeout_ms,
            EnginesSettings::INTEGRATIONS_TIMEOUT_MS
        );
        // A limit the user chose stays, and so does 10 s chosen after the upgrade.
        assert_eq!(
            saved(1, 30_000).upgraded().engines.integrations.timeout_ms,
            30_000
        );
        let now = saved(SETTINGS_VERSION, EngineSettings::DEFAULT_TIMEOUT_MS).upgraded();
        assert_eq!(
            now.engines.integrations.timeout_ms,
            EngineSettings::DEFAULT_TIMEOUT_MS
        );
    }
}
