# Quickstart: Project settings

1. Run the tests:
   ```sh
   cargo test --workspace --all-features
   ```
   The setting checks and the draft are tested in `crates/aurora-core/src/domain/config.rs`, the use cases in `usecases/config.rs`, the adapter against the fake `dcp` in `crates/aurora-engines/tests/config.rs`, and the sample replies against the JSON Schemas in `tests/contract_samples.rs`.
2. Open the preview:
   ```sh
   cd crates/aurora-ui && trunk serve
   ```
   Open the Project view on `gedo` and press "Settings". The groups `devops`, `architecture`, `backlog`, `docs` and `principles` are shown; "Branching model" is `trunk`; "Mandatory gates" is read-only with its reason. Change "Branching model" to `integration` and turn "Release candidates" off: "2 changes" is shown. Press "Preview changes": the preview lists `devops.branching` trunk → integration and `devops.release_candidates` true → false, and the diff. Press "Apply": "Saved to .dev-cycle.json. Commit it to share the change." Type `abc` in "README line limit": the field says it needs a whole number. Set "Integration branch" to `main`: the preview shows the error `dcp` returns next to the field.
3. In the desktop application without `dcp` (or with a `dcp` without `config`), the Settings tab says what to install and starts no process.
