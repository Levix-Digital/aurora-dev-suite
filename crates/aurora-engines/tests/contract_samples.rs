//! The `dcp.diagnose` samples follow the published contract
//! (specs/002-dcp-adapter/contracts/dcp-diagnose.schema.json) and read into the same states
//! as the core's fixtures, so the tests, the fake engine and the preview cannot drift from it.

use std::path::{Path, PathBuf};

use aurora_core::domain::CycleState;
use aurora_core::fixtures;
use aurora_engines::cycle::parse_diagnose_reply;
use serde_json::Value;

const SAMPLES: [&str; 4] = [
    "busy.json",
    "idle.json",
    "release-candidate.json",
    "unknown-fields.json",
];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_json(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn schema() -> Value {
    read_json(
        &manifest_dir().join("../../specs/002-dcp-adapter/contracts/dcp-diagnose.schema.json"),
    )
}

fn sample(name: &str) -> Value {
    read_json(
        &manifest_dir()
            .join("tests/fixtures/dcp-diagnose")
            .join(name),
    )
}

fn check(value: &Value) -> Vec<String> {
    let validator =
        jsonschema::draft202012::new(&schema()).expect("the contract is a valid schema");
    validator
        .iter_errors(value)
        .map(|error| format!("{}: {error}", error.instance_path()))
        .collect()
}

fn parsed(name: &str) -> CycleState {
    let value = sample(name);
    let root = value["root"].as_str().expect("root").to_owned();
    let bytes = serde_json::to_vec(&value).expect("serializes");
    parse_diagnose_reply(&bytes, Path::new(&root), fixtures::CHECKED_AT).expect("valid sample")
}

#[test]
fn every_sample_is_valid_against_the_contract() {
    for name in SAMPLES {
        let problems = check(&sample(name));
        assert!(problems.is_empty(), "{name}: {problems:#?}");
    }
}

#[test]
fn the_validator_catches_contract_breaks() {
    let mut broken = sample("busy.json");
    broken["schema"] = "dcp.version".into();
    broken["plans"][0]["milestones_done"] = (-1).into();
    broken["bugs"]["untriaged"][0]
        .as_object_mut()
        .expect("object")
        .remove("reason");
    broken.as_object_mut().expect("object").remove("findings");
    broken["schema_version"] = "2.0".into();
    broken["bugs"]["blocking"][0]["id"] = "bug 12".into();
    let problems = check(&broken);
    assert_eq!(problems.len(), 6, "{problems:#?}");
}

#[test]
fn samples_read_into_the_core_fixtures() {
    assert_eq!(parsed("busy.json"), fixtures::busy_cycle_state());
    assert_eq!(parsed("idle.json"), fixtures::idle_cycle_state());
    assert_eq!(
        parsed("release-candidate.json"),
        fixtures::candidate_cycle_state()
    );
}

#[test]
fn unknown_fields_are_ignored() {
    let state = parsed("unknown-fields.json");
    assert_eq!(state.next.recommended.as_deref(), Some("request"));
    assert_eq!(state.plans.len(), 1);
}
