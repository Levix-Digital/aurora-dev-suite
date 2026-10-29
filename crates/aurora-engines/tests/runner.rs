#![cfg(feature = "fake-engine")]

mod common;

use std::time::{Duration, Instant};

use aurora_engines::runner::{self, CommandSpec, MAX_STDOUT_BYTES, RunError};

fn spec(program: &std::path::Path, args: &[&str], timeout_ms: u64) -> CommandSpec {
    CommandSpec::new(program, args, Duration::from_millis(timeout_ms))
}

#[test]
fn returns_the_reply_and_exit_code() {
    let dir = common::engine_dir();
    let gedo = common::install_fake(dir.path(), "gedo", "ready");
    let output = runner::run(&spec(&gedo, &["version", "--json"], 10_000)).unwrap();
    assert!(output.succeeded());
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"gedo.version\""));
}

#[test]
fn a_hanging_engine_is_killed_at_the_time_limit() {
    let dir = common::engine_dir();
    let gedo = common::install_fake(dir.path(), "gedo", "hang");
    let started = Instant::now();
    let result = runner::run(&spec(&gedo, &["version", "--json"], 500));
    assert!(
        matches!(result, Err(RunError::TimedOut { .. })),
        "{result:?}"
    );
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "{:?}",
        started.elapsed()
    );
}

#[test]
fn a_reply_over_the_limit_is_refused() {
    let dir = common::engine_dir();
    let actx = common::install_fake(dir.path(), "actx", "huge");
    let result = runner::run(&spec(&actx, &["version", "--json"], 10_000));
    assert!(
        matches!(result, Err(RunError::TooLarge { limit, .. }) if limit == MAX_STDOUT_BYTES),
        "{result:?}"
    );
}

#[test]
fn error_output_is_kept_on_failure() {
    let dir = common::engine_dir();
    let dcp = common::install_fake(dir.path(), "dcp", "crash");
    let output = runner::run(&spec(&dcp, &["version", "--json"], 10_000)).unwrap();
    assert_eq!(output.exit_code, Some(101));
    assert!(
        output.stderr.contains("crashed on purpose"),
        "{}",
        output.stderr
    );
}

#[test]
fn arguments_reach_the_engine_without_a_shell() {
    let dir = common::engine_dir();
    let gedo = common::install_fake(dir.path(), "gedo", "echo");
    let tricky = ["a b", "$(whoami)", "x;y", "*", "|", "\"q\""];
    let output = runner::run(&spec(&gedo, &tricky, 10_000)).unwrap();
    let echoed: Vec<String> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(echoed, tricky);
}

#[test]
fn a_missing_program_fails_to_start() {
    let dir = common::engine_dir();
    let result = runner::run(&spec(&dir.path().join("nothing"), &[], 1_000));
    assert!(matches!(result, Err(RunError::Spawn { .. })), "{result:?}");
}

#[test]
fn an_engine_leaving_a_child_on_its_pipes_does_not_block_past_the_limit() {
    let dir = common::engine_dir();
    let gedo = common::install_fake(dir.path(), "gedo", "orphan");
    let started = Instant::now();
    let result = runner::run(&spec(&gedo, &["--version"], 1_000));
    assert!(
        matches!(result, Err(RunError::TimedOut { .. })),
        "{result:?}"
    );
    assert!(
        started.elapsed() < Duration::from_secs(3),
        "{:?}",
        started.elapsed()
    );
}
