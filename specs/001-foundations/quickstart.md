# Quickstart: validating Foundations

## Prerequisites

- Rust 1.95 or later with the `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`.
- Trunk 0.22: `cargo install trunk --locked`.
- Linux only: `libwebkit2gtk-4.1-dev libsoup-3.0-dev librsvg2-dev libayatana-appindicator3-dev`.
- Windows only: WebView2 (preinstalled on Windows 11).

## Checks a contributor runs

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo xtask arch-check
cargo deny check
```

All tests use fixtures and fake engines only (SC-006).

## The interface preview (US4)

```text
cd crates/aurora-ui
trunk serve
```

Open http://127.0.0.1:1420: every view shows the fixture projects and engines (ready, without contract, missing; each Refresh on Engines switches Gedo between ready and degraded).

## The desktop application (US1 to US3)

```text
cd crates/aurora-ui && trunk build --release && cd ../..
cargo run -p aurora-desktop --release --features custom-protocol
```

1. The window opens on Portfolio within 1.5 s (SC-001). Navigate with Tab and the arrow keys only (SC-005).
2. Add three repositories, rename one, remove one, restart: the list is kept (SC-002). Try a folder outside any repository: refused with a reason.
3. Open Engines: without engines all three are missing. Put a fake engine on the search path (`cargo build -p aurora-engines --features fake-engine` builds `aurora-fake-engine`; copy it as `gedo`; it answers the contract by default, and a `gedo.mode` file next to it or `AURORA_FAKE_MODE` picks another mode), press Refresh: Gedo becomes ready with its capabilities (SC-004). With the mode `hang`, Gedo is shown degraded after its time limit while the others load (SC-003).
4. Corrupt `projects.toml` by hand and restart: the damaged file is kept aside and a notice says where (FR-010).
