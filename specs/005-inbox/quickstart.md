# Quickstart: Inbox

1. Run the tests:
   ```sh
   cargo test --workspace --all-features
   ```
   The urgency, details and order are tested in `crates/aurora-core/src/domain/portfolio.rs` and `crates/aurora-core/src/domain/inbox.rs`.
2. Open the preview:
   ```sh
   cd crates/aurora-ui && trunk serve
   ```
   Open the Inbox: "4 items wait for you across 2 projects", then, in order: "Blocks a release" with `gedo` BUG-012 (severity high); "Waits for your acceptance" with `ai-code-agent-skills` "Run the manual tests of v2.9.0-rc.1" (4 not run, 1 failed); "Waits for your review" with `gedo` #41 (1 review thread for you); "To triage" with `gedo` BUG-015 (no severity). "Not read" lists `levix-platform` with the text to locate its folder. Press "Open" on the release candidate: the Project view opens on `ai-code-agent-skills`.
3. In the desktop application without `dcp`, the Inbox says what to install once and reads "Cycle state unavailable".
