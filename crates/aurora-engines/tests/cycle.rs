#![cfg(feature = "fake-engine")]

mod common;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use aurora_core::domain::{CycleError, CycleState, EngineKind, EngineSettings, UpdateSide};
use aurora_core::fixtures;
use aurora_core::ports::CycleSource;
use aurora_engines::DcpCycleSource;

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

fn ask(setup: &Setup, timeout_ms: u64) -> Result<CycleState, CycleError> {
    let mut report = fixtures::ready_report(EngineKind::DevCycle, &["diagnose"]);
    report.location = Some(setup.dcp.clone());
    let settings = EngineSettings {
        path: None,
        timeout_ms,
    };
    DcpCycleSource::new().cycle_state(&report, &settings, &setup.root)
}

/// The busy sample, as read now.
fn busy_except_time(state: &CycleState) -> CycleState {
    CycleState {
        read_at: state.read_at.clone(),
        ..fixtures::busy_cycle_state()
    }
}

#[test]
fn a_ready_dcp_gives_the_full_state() {
    let setup = setup("ready", "gedo");
    let state = ask(&setup, 5_000).expect("state");
    assert_eq!(state, busy_except_time(&state));
    assert_ne!(state.read_at, fixtures::CHECKED_AT);
}

#[test]
fn an_idle_project_is_an_empty_state() {
    let setup = setup("idle", "any-context");
    let state = ask(&setup, 5_000).expect("state");
    assert!(state.release.is_none() && state.plans.is_empty() && state.pull_requests.is_empty());
}

#[test]
fn a_path_with_spaces_and_accents_is_passed_intact() {
    let setup = setup("ready", "projeto de ação");
    assert!(ask(&setup, 5_000).is_ok());
}

#[test]
fn warnings_on_standard_error_do_not_fail_a_reply() {
    let setup = setup("warn", "gedo");
    assert!(ask(&setup, 5_000).is_ok());
}

#[test]
fn a_hanging_dcp_times_out_within_its_limit() {
    let setup = setup("hang", "gedo");
    let started = Instant::now();
    let result = ask(&setup, 500);
    assert!(
        matches!(
            result,
            Err(CycleError::TimedOut {
                timeout_ms: 500,
                ..
            })
        ),
        "{result:?}"
    );
    assert!(started.elapsed() < Duration::from_millis(1_500));
}

#[test]
fn a_crash_keeps_the_exit_code_and_error_output() {
    let setup = setup("crash", "gedo");
    let Err(CycleError::Failed { exit_code, stderr }) = ask(&setup, 5_000) else {
        panic!("expected failed");
    };
    assert_eq!(exit_code, Some(101));
    assert!(stderr.contains("crashed on purpose"), "{stderr}");
}

#[test]
fn a_huge_reply_is_too_large() {
    let setup = setup("huge", "gedo");
    assert!(matches!(
        ask(&setup, 5_000),
        Err(CycleError::TooLarge { .. })
    ));
}

#[test]
fn an_invalid_reply_is_refused() {
    let setup = setup("invalid", "gedo");
    assert!(matches!(
        ask(&setup, 5_000),
        Err(CycleError::Invalid { .. })
    ));
}

#[test]
fn a_reply_about_another_repository_is_refused() {
    let setup = setup("other-root", "gedo");
    let Err(CycleError::Invalid { message, .. }) = ask(&setup, 5_000) else {
        panic!("expected invalid");
    };
    assert!(message.contains("/repos/gedo"), "{message}");
}

#[test]
fn unsupported_schema_versions_say_which_side_to_update() {
    assert_eq!(
        ask(&setup("unsupported", "gedo"), 5_000),
        Err(CycleError::Unsupported {
            found: 2,
            update: UpdateSide::Aurora
        })
    );
    assert_eq!(
        ask(&setup("old", "gedo"), 5_000),
        Err(CycleError::Unsupported {
            found: 0,
            update: UpdateSide::Engine
        })
    );
}

#[test]
fn a_dcp_that_cannot_start_fails() {
    let setup = setup("ready", "gedo");
    let mut report = fixtures::ready_report(EngineKind::DevCycle, &["diagnose"]);
    report.location = Some(setup.root.join("no-such-dcp"));
    let result = DcpCycleSource::new().cycle_state(
        &report,
        &EngineSettings::default(),
        Path::new(&setup.root),
    );
    assert!(
        matches!(
            result,
            Err(CycleError::Failed {
                exit_code: None,
                ..
            })
        ),
        "{result:?}"
    );
}
