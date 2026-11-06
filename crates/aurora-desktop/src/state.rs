//! The composition root: the real adapters behind the core's ports (ADR-0001).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use aurora_core::domain::{EnginesOverview, SetupMemory};
use aurora_core::ports::{
    ConfigSource, CycleSource, EngineProbe, MessagesSource, ProjectRegistry, SettingsStore,
};
use aurora_engines::{
    CliProbe, DcpConfigSource, DcpCycleSource, ProviderMessagesSource, TomlRegistry, TomlSettings,
};

pub struct AppState {
    pub registry: Arc<dyn ProjectRegistry>,
    pub settings: Arc<dyn SettingsStore>,
    pub probe: Arc<dyn EngineProbe>,
    pub cycle: Arc<dyn CycleSource>,
    pub config: Arc<dyn ConfigSource>,
    pub messages: Arc<dyn MessagesSource>,
    /// The result of the last complete probe of the engines.
    pub engines: Arc<Mutex<Option<EnginesOverview>>>,
    /// Held during the first probe, so concurrent first calls probe once (spec 003 FR-009).
    pub first_probe: Arc<Mutex<()>>,
    /// Each project's setup as last seen, so a change reads as news (IMP-004).
    pub setups: Arc<Mutex<SetupMemory>>,
    /// When the window's focus last started a check, to keep it to one per interval.
    pub last_focus_check: Arc<Mutex<Option<std::time::Instant>>>,
}

impl AppState {
    pub fn new(config_dir: PathBuf) -> Self {
        tracing::info!(config_dir = %config_dir.display(), "opening Aurora's files");
        Self {
            registry: Arc::new(TomlRegistry::open(&config_dir)),
            settings: Arc::new(TomlSettings::open(&config_dir)),
            probe: Arc::new(CliProbe::new()),
            cycle: Arc::new(DcpCycleSource::new()),
            config: Arc::new(DcpConfigSource::new()),
            messages: Arc::new(ProviderMessagesSource),
            engines: Arc::new(Mutex::new(None)),
            first_probe: Arc::new(Mutex::new(())),
            setups: Arc::new(Mutex::new(SetupMemory::default())),
            last_focus_check: Arc::new(Mutex::new(None)),
        }
    }

    /// Aurora's configuration directory, or a folder in the temporary directory when the
    /// system has no home directory (the registry then says nothing is kept).
    pub fn default_config_dir() -> PathBuf {
        aurora_engines::registry::config_dir().unwrap_or_else(|| {
            tracing::warn!("no home directory; using a temporary configuration folder");
            std::env::temp_dir().join("aurora")
        })
    }
}
