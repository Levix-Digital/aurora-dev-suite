# Quickstart: Portfolio view

1. Run the tests:
   ```sh
   cargo test --workspace --all-features
   ```
   The phase and waiting rules are tested in `crates/aurora-core/src/domain/portfolio.rs`, the loader in `crates/aurora-ui/src/views/loader.rs`.
2. Open the preview:
   ```sh
   cd crates/aurora-ui && trunk serve
   ```
   The Portfolio shows `gedo` in Implementation with 3 items waiting, `any-context` Idle, `ai-code-agent-skills` in Release candidate with 1 item, and `levix-platform` missing.
3. In the desktop application without `dcp`, the Portfolio says what to install once and every present project reads "Cycle state unavailable". With the fake engine as `dcp` (see `specs/002-dcp-adapter/quickstart.md`), every present project shows the busy sample.
