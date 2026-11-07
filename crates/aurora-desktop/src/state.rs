//! The composition root: the real adapters behind the core's ports (ADR-0001).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use aurora_core::domain::EnginesOverview;
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
    /// When the window's focus last started a reading, to keep it to one per interval.
    pub last_focus_check: Arc<Mutex<Option<std::time::Instant>>>,
    /// Whether Aurora's window has the focus: the background interval depends on it.
    pub focused: Arc<std::sync::atomic::AtomicBool>,
    /// The messages worker, started with the application (IMP-007).
    pub worker: Arc<std::sync::OnceLock<crate::worker::Worker>>,
    /// Set by the view's first snapshot request, which starts the first reading.
    pub snapshot_asked: Arc<std::sync::atomic::AtomicBool>,
    /// Aurora's configuration folder, where the messages snapshot is kept.
    pub config_dir: PathBuf,
}

impl AppState {
    pub fn new(config_dir: PathBuf) -> Self {
        tracing::info!(config_dir = %config_dir.display(), "opening Aurora's files");
        let dir = config_dir.clone();
        Self {
            registry: Arc::new(TomlRegistry::open(&dir)),
            settings: Arc::new(TomlSettings::open(&dir)),
            probe: Arc::new(CliProbe::new()),
            cycle: Arc::new(DcpCycleSource::new()),
            config: Arc::new(DcpConfigSource::new()),
            messages: Arc::new(ProviderMessagesSource),
            engines: Arc::new(Mutex::new(None)),
            first_probe: Arc::new(Mutex::new(())),
            last_focus_check: Arc::new(Mutex::new(None)),
            focused: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            worker: Arc::new(std::sync::OnceLock::new()),
            snapshot_asked: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            config_dir,
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
