# Quickstart: Project view

1. Run the tests:
   ```sh
   cargo test --workspace --all-features
   ```
   The gates, candidate stage, next step order and plan progress are tested in `crates/aurora-core/src/domain/project_view.rs`.
2. Open the preview:
   ```sh
   cd crates/aurora-ui && trunk serve
   ```
   Press "Open" on `gedo`: the Project view shows Implementation, 3 items waiting, the Decision gate recommending "Fix the blocking bug first", plan "Command line" at 3 of 5 milestones, release v0.4.0 with FEAT-031 and FEAT-034, pull request #41 needing action, BUG-012 blocking and BUG-015 untriaged. Choose `ai-code-agent-skills`: Release candidate, the gates Decision and "Release candidate acceptance" (4 not run, 1 failed) and one finding. Choose `any-context`: Idle, "No gate waits for you". Choose `levix-platform`: it says to locate the folder.
3. In the desktop application without `dcp`, the view says what to install once and reads "Cycle state unavailable".
