#![cfg(feature = "fake-engine")]

mod common;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use aurora_core::domain::{
    ConfigChange, ConfigError, ConfigOutcome, ConfigValue, EngineKind, EngineReport,
    EngineSettings, KeyValue, ProjectConfig, UpdateSide,
};
use aurora_core::fixtures;
use aurora_core::ports::ConfigSource;
use aurora_engines::DcpConfigSource;

/// A fake `dcp` in `mode` and a repository whose folder name is `repo`.
struct Setup {
    _dir: tempfile::TempDir,
    dcp: PathBuf,
    root: PathBuf,
}

fn setup(mode: &str, repo: &str) -> Setup {
    let dir = common::engine_dir();
    let dcp = common::install_fake(dir.path(), "dcp", mode);
    let root = common::make_repo(dir.path(), repo);
    Setup {
        _dir: dir,
        dcp,
        root,
    }
}

impl Setup {
    fn report(&self) -> EngineReport {
        let mut report = fixtures::ready_report(EngineKind::DevCycle, &["diagnose", "config"]);
        report.location = Some(self.dcp.clone());
        report
    }

    fn read(&self, timeout_ms: u64) -> Result<ProjectConfig, ConfigError> {
        let settings = EngineSettings {
            path: None,
            timeout_ms,
        };
        DcpConfigSource::new().read(&self.report(), &settings, &self.root)
    }

    fn set(&self, dry_run: bool) -> Result<ConfigOutcome, ConfigError> {
        DcpConfigSource::new().set(
            &self.report(),
            &EngineSettings::default(),
            &self.root,
            &change(),
            dry_run,
        )
    }

    /// The arguments the fake `dcp` received for the last `config set`.
    fn last_args(&self) -> Vec<String> {
        let mut file = self.dcp.as_os_str().to_owned();
        file.push(".args");
        std::fs::read_to_string(PathBuf::from(file))
            .expect("arguments recorded")
            .lines()
            .map(str::to_owned)
            .collect()
    }
}

fn change() -> ConfigChange {
    ConfigChange {
        revision: "sha256:4f1c2a".into(),
        values: vec![
            KeyValue {
                key: "devops.branching".into(),
                value: Some(ConfigValue::Text("integration".into())),
            },
            KeyValue {
                key: "devops.release_candidates".into(),
                value: Some(ConfigValue::Boolean(false)),
            },
        ],
    }
}

#[test]
fn a_ready_dcp_gives_the_sample_settings() {
    let setup = setup("ready", "gedo");
    let config = setup.read(5_000).expect("settings");
    assert_eq!(
        config,
        ProjectConfig {
            read_at: config.read_at.clone(),
            ..fixtures::sample_project_config()
        }
    );
}

#[test]
fn a_preview_is_a_dry_run_with_every_value_as_json() {
    let setup = setup("ready", "projeto de ação");
    let outcome = setup.set(true).expect("preview");
    assert!(!outcome.applied);
    assert_eq!(outcome.changes.len(), 2);
    let root = setup.root.to_string_lossy().into_owned();
    assert_eq!(
        setup.last_args(),
        [
            "config",
            "set",
            "--json",
            "--dry-run",
            "--dir",
            root.as_str(),
            "--revision",
            "sha256:4f1c2a",
            "--value",
            r#"devops.branching="integration""#,
            "--value",
            "devops.release_candidates=false",
        ]
    );
}

#[test]
fn an_apply_is_not_a_dry_run() {
    let setup = setup("ready", "gedo");
    let outcome = setup.set(false).expect("apply");
    assert!(outcome.applied);
    assert_eq!(outcome.revision, "sha256:9b07d3");
    assert!(!setup.last_args().contains(&"--dry-run".to_owned()));
}

#[test]
fn refusals_and_stale_revisions_come_back_as_errors() {
    let Err(ConfigError::Rejected { errors }) = setup("refused", "gedo").set(true) else {
        panic!("expected a refusal");
    };
    assert_eq!(errors[0].key.as_deref(), Some("devops.integration_branch"));
    assert_eq!(setup("stale", "gedo").set(false), Err(ConfigError::Stale));
}

#[test]
fn invalid_and_unsupported_replies_are_refused() {
    assert!(matches!(
        setup("invalid", "gedo").read(5_000),
        Err(ConfigError::Invalid { .. })
    ));
    assert!(matches!(
        setup("invalid", "gedo").set(true),
        Err(ConfigError::Invalid { .. })
    ));
    assert_eq!(
        setup("unsupported", "gedo").read(5_000),
        Err(ConfigError::Unsupported {
            found: 2,
            update: UpdateSide::Aurora
        })
    );
}

#[test]
fn a_hanging_dcp_times_out_within_its_limit() {
    let setup = setup("hang", "gedo");
    let started = Instant::now();
    assert!(matches!(
        setup.read(500),
        Err(ConfigError::TimedOut {
            timeout_ms: 500,
            ..
        })
    ));
    assert!(started.elapsed() < Duration::from_millis(1_500));
}

#[test]
fn a_crash_keeps_the_exit_code_and_error_output() {
    let Err(ConfigError::Failed { exit_code, stderr }) = setup("crash", "gedo").read(5_000) else {
        panic!("expected failed");
    };
    assert_eq!(exit_code, Some(101));
    assert!(stderr.contains("crashed on purpose"), "{stderr}");
}

#[test]
fn a_dcp_without_a_location_is_unavailable() {
    let setup = setup("ready", "gedo");
    let mut report = setup.report();
    report.location = None;
    assert!(matches!(
        DcpConfigSource::new().read(&report, &EngineSettings::default(), &setup.root),
        Err(ConfigError::Unavailable { .. })
    ));
}
