#![cfg(feature = "fake-engine")]

mod common;

use std::path::Path;

use aurora_core::domain::{EngineKind, EngineSettings, EngineStatus};
use aurora_core::ports::EngineProbe;
use aurora_engines::CliProbe;

fn probe_in(dir: &Path, kind: EngineKind, timeout_ms: u64) -> aurora_core::domain::EngineReport {
    let settings = EngineSettings {
        path: None,
        timeout_ms,
    };
    CliProbe::with_search_path(&[dir]).probe(kind, &settings)
}

fn installed(kind: EngineKind, mode: &str) -> aurora_core::domain::EngineReport {
    let dir = common::engine_dir();
    common::install_fake(dir.path(), kind.command(), mode);
    probe_in(dir.path(), kind, 2_000)
}

#[test]
fn an_engine_not_on_the_path_is_missing() {
    let dir = common::engine_dir();
    let report = probe_in(dir.path(), EngineKind::DevCycle, 2_000);
    assert_eq!(report.status, EngineStatus::Missing);
    assert_eq!(report.location, None);
}

#[test]
fn an_engine_with_the_contract_is_ready_with_its_capabilities() {
    let report = installed(EngineKind::Gedo, "ready");
    assert_eq!(report.status, EngineStatus::Ready);
    assert_eq!(report.version.as_deref(), Some("1.2.3"));
    assert!(report.has_capability("radar", 1));
    assert!(report.location.is_some());
}

#[test]
fn an_engine_without_the_contract_falls_back_to_version() {
    let report = installed(EngineKind::AnyContext, "no-contract");
    assert_eq!(report.status, EngineStatus::InstalledWithoutContract);
    assert_eq!(report.version.as_deref(), Some("1.2.3"));
    assert!(report.capabilities.is_empty());
}

#[test]
fn a_newer_contract_asks_to_update_aurora() {
    let report = installed(EngineKind::DevCycle, "unsupported");
    let EngineStatus::Degraded { reason } = report.status else {
        panic!("expected degraded, got {:?}", report.status);
    };
    assert!(reason.contains("update Aurora"), "{reason}");
}

#[test]
fn an_invalid_reply_is_degraded() {
    let report = installed(EngineKind::Gedo, "invalid");
    assert!(
        matches!(report.status, EngineStatus::Degraded { .. }),
        "{:?}",
        report.status
    );
}

#[test]
fn a_crash_is_degraded_and_keeps_the_error_output() {
    let report = installed(EngineKind::Gedo, "crash");
    assert!(matches!(report.status, EngineStatus::Degraded { .. }));
    let last_error = report.last_error.unwrap_or_default();
    assert!(last_error.contains("crashed on purpose"), "{last_error}");
}

#[test]
fn a_hanging_engine_is_degraded_after_its_time_limit() {
    let dir = common::engine_dir();
    common::install_fake(dir.path(), "gedo", "hang");
    let started = std::time::Instant::now();
    let report = probe_in(dir.path(), EngineKind::Gedo, 300);
    assert!(matches!(report.status, EngineStatus::Degraded { .. }));
    assert!(started.elapsed() < std::time::Duration::from_secs(5));
}

#[test]
fn a_huge_reply_is_degraded() {
    let report = installed(EngineKind::AnyContext, "huge");
    assert!(matches!(report.status, EngineStatus::Degraded { .. }));
}

#[test]
fn the_configured_path_wins_over_the_search_path() {
    let on_path = common::engine_dir();
    common::install_fake(on_path.path(), "gedo", "no-contract");
    let configured = common::engine_dir();
    let chosen = common::install_fake(configured.path(), "gedo", "ready");
    let settings = EngineSettings {
        path: Some(chosen.clone()),
        timeout_ms: 2_000,
    };
    let report = CliProbe::with_search_path(&[on_path.path()]).probe(EngineKind::Gedo, &settings);
    assert_eq!(report.status, EngineStatus::Ready);
    assert_eq!(report.location, Some(chosen));
}

#[test]
fn a_configured_path_that_does_not_exist_is_missing_with_a_reason() {
    let dir = common::engine_dir();
    let settings = EngineSettings {
        path: Some(dir.path().join("gedo-not-here")),
        timeout_ms: 2_000,
    };
    let report = CliProbe::with_search_path(&[dir.path()]).probe(EngineKind::Gedo, &settings);
    assert_eq!(report.status, EngineStatus::Missing);
    assert!(report.last_error.is_some());
}
