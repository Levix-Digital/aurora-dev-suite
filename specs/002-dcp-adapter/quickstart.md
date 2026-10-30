# Quickstart: dcp adapter

Checks that the adapter works without a real `dcp`.

1. Run the tests, including the fake engine:
   ```sh
   cargo test --workspace --all-features
   ```
   The adapter tests in `crates/aurora-engines/tests/cycle.rs` run the fake engine as `dcp` in every mode and validate the sample replies against `specs/002-dcp-adapter/contracts/dcp-diagnose.schema.json`.
2. Try the fake engine by hand:
   ```sh
   cargo build -p aurora-engines --features fake-engine --bin aurora-fake-engine
   cp target/debug/aurora-fake-engine /tmp/dcp
   /tmp/dcp version --json
   /tmp/dcp diagnose --json --dir "$PWD"
   ```
   The version reply lists `diagnose` with `schema_major` 1; the diagnose reply echoes the root.
3. In the desktop application, set the `dcp` location to `/tmp/dcp` in Settings, refresh the engines, and call `project_cycle_state` for a registered project from the developer console: it returns the sample state. The views keep their "coming in v0.2.0" state; FEAT-014, FEAT-015 and FEAT-017 show it.
