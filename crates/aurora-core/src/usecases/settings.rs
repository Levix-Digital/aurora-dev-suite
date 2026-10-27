//! Reading and saving the user's settings (FR-003, FR-011).

use crate::domain::Settings;
use crate::ports::{SettingsError, SettingsStore};

pub fn get(store: &dyn SettingsStore) -> Settings {
    store.load()
}

pub fn save(store: &dyn SettingsStore, settings: Settings) -> Result<Settings, SettingsError> {
    store.save(&settings)?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use crate::domain::{Theme, ThemeChoice};

    #[test]
    fn system_choice_follows_the_operating_system() {
        assert_eq!(ThemeChoice::System.resolve(true), Theme::Dark);
        assert_eq!(ThemeChoice::System.resolve(false), Theme::Light);
    }

    #[test]
    fn explicit_choice_wins_over_the_operating_system() {
        assert_eq!(ThemeChoice::Light.resolve(true), Theme::Light);
        assert_eq!(ThemeChoice::Dark.resolve(false), Theme::Dark);
    }

    #[cfg(feature = "fixtures")]
    #[test]
    fn saved_settings_are_read_back() {
        use crate::domain::Settings;
        use crate::fixtures::MemorySettings;

        let store = MemorySettings::default();
        let mut settings = super::get(&store);
        settings.theme = ThemeChoice::Dark;
        settings.engines.gedo.timeout_ms = 2_000;
        super::save(&store, settings.clone()).expect("saved");
        assert_eq!(super::get(&store), settings);
        assert_ne!(super::get(&store), Settings::default());
    }
}
