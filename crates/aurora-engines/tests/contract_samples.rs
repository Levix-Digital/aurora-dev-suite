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

/// The subset of JSON Schema the contract uses: `type` (one or several), `const`,
/// `required`, `properties`, `items`, `allOf`, `minimum` and local `$ref`s.
fn violations(schema: &Value, root: &Value, value: &Value, at: &str, out: &mut Vec<String>) {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        let pointer = reference.trim_start_matches('#');
        let target = root
            .pointer(pointer)
            .unwrap_or_else(|| panic!("unknown $ref {reference}"));
        violations(target, root, value, at, out);
    }
    if let Some(all) = schema.get("allOf").and_then(Value::as_array) {
        for part in all {
            violations(part, root, value, at, out);
        }
    }
    if let Some(expected) = schema.get("const")
        && value != expected
    {
        out.push(format!("{at}: expected {expected}"));
    }
    if let Some(types) = schema.get("type") {
        let allowed: Vec<&str> = match types {
            Value::String(one) => vec![one.as_str()],
            Value::Array(many) => many.iter().filter_map(Value::as_str).collect(),
            _ => Vec::new(),
        };
        if !allowed.iter().any(|kind| has_type(value, kind)) {
            out.push(format!("{at}: expected {allowed:?}, got {value}"));
            return;
        }
    }
    if let (Some(minimum), Some(number)) = (
        schema.get("minimum").and_then(Value::as_f64),
        value.as_f64(),
    ) && number < minimum
    {
        out.push(format!("{at}: below {minimum}"));
    }
    if let Value::Object(object) = value {
        for field in schema
            .get("required")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if !object.contains_key(field) {
                out.push(format!("{at}: missing `{field}`"));
            }
        }
        if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
            for (name, property) in properties {
                if let Some(child) = object.get(name) {
                    violations(property, root, child, &format!("{at}.{name}"), out);
                }
            }
        }
    }
    if let (Value::Array(items), Some(item_schema)) = (value, schema.get("items")) {
        for (index, item) in items.iter().enumerate() {
            violations(item_schema, root, item, &format!("{at}[{index}]"), out);
        }
    }
}

fn has_type(value: &Value, kind: &str) -> bool {
    match kind {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        "integer" => value.is_i64() || value.is_u64(),
        "number" => value.is_number(),
        _ => false,
    }
}

fn check(value: &Value) -> Vec<String> {
    let schema = schema();
    let mut out = Vec::new();
    violations(&schema, &schema, value, "$", &mut out);
    out
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
fn the_checker_catches_contract_breaks() {
    let mut broken = sample("busy.json");
    broken["schema"] = "dcp.version".into();
    broken["plans"][0]["milestones_done"] = (-1).into();
    broken["bugs"]["untriaged"][0]
        .as_object_mut()
        .expect("object")
        .remove("reason");
    broken.as_object_mut().expect("object").remove("findings");
    let problems = check(&broken);
    assert_eq!(problems.len(), 4, "{problems:#?}");
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
