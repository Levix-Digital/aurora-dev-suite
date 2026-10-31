//! The composition root: the real adapters behind the core's ports (ADR-0001).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use aurora_core::domain::EnginesOverview;
use aurora_core::ports::{ConfigSource, CycleSource, EngineProbe, ProjectRegistry, SettingsStore};
use aurora_engines::{CliProbe, DcpConfigSource, DcpCycleSource, TomlRegistry, TomlSettings};

pub struct AppState {
    pub registry: Arc<dyn ProjectRegistry>,
    pub settings: Arc<dyn SettingsStore>,
    pub probe: Arc<dyn EngineProbe>,
    pub cycle: Arc<dyn CycleSource>,
    pub config: Arc<dyn ConfigSource>,
    /// The result of the last complete probe of the engines.
    pub engines: Arc<Mutex<Option<EnginesOverview>>>,
    /// Held during the first probe, so concurrent first calls probe once (spec 003 FR-009).
    pub first_probe: Arc<Mutex<()>>,
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
            engines: Arc::new(Mutex::new(None)),
            first_probe: Arc::new(Mutex::new(())),
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
