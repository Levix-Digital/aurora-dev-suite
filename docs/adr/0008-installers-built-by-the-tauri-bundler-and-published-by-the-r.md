---
id: 8
title: Installers built by the Tauri bundler and published by the release workflow
status: accepted
date: '2026-10-30'
version: v0.2.0
subsystems: [desktop]
supersedes: []
superseded_by: []
amends: []
amended_by: []
---

# ADR-0008: Installers built by the Tauri bundler and published by the release workflow

## Context

Until v0.1.0 Aurora was built from source; there was no installer and no release workflow. The Dev Cycle's release candidates (trunk branching, candidates `vX.Y.Z-rc.N` on `main`, the promote workflow as the release gate) need an installable package so the user can accept a candidate without a Rust toolchain. FEAT-005 asks for a Windows installer (NSIS or MSI) and Linux packages (AppImage and deb) built in CI. ADR-0005 puts signed installers in a separate public releases repository, which does not exist yet, and there is no signing certificate.

## Decision

- The Tauri bundler builds the packages from the `bundle` section of `crates/aurora-desktop/tauri.conf.json`, with the Tauri CLI 2.x as a build tool (no crate is linked): a Windows NSIS installer, per user without administrator rights and with the WebView2 bootstrapper embedded; a Linux `.deb` that declares WebKitGTK 4.1 and GTK 3; and a Linux AppImage that carries its libraries. Packages take their version from the workspace `Cargo.toml` only.
- `.github/workflows/release.yml` runs on `v*` tags and from the promote workflow, and as a dry run on pull requests that change the packaging and by hand. It checks the tag against the workspace version (`cargo xtask release-check`), tests and packages on Windows and Linux, and only after both succeed one job publishes the release with `SHA256SUMS.txt` through the GitHub CLI: candidates as pre-releases never marked latest. A dry run keeps the packages as run artifacts and publishes nothing.
- `.github/workflows/promote.yml` is the trunk release gate: it tags the accepted candidate's commit `vX.Y.Z` in the `release` environment and calls the release workflow.
- Releases go to the repository named by the `RELEASES_REPOSITORY` variable with the `RELEASES_TOKEN` secret when set, and to the source repository otherwise.

## Alternatives Considered

- **MSI (WiX) instead of or beside NSIS**: per-machine installs for managed deployments, but needs administrator rights and the WiX toolset, and two installers to test per candidate; added later if a deployment needs it.
- **Separate tools (`cargo-deb`, `cargo-wix`, a hand-written NSIS script)**: three configurations for what the bundler does from one, and none of them knows WebView2.
- **Publishing from each build job, or with a third-party release action**: a failed platform would leave a half-published release; a third-party action adds a supply-chain dependency where the preinstalled GitHub CLI suffices.
- **Writing the version from the tag at build time**: the version would live in two places; the tag is checked instead.

## Consequences

- Every candidate and release carries the three packages and their checksums, with no manual step; the user installs a candidate to run its manual tests (section "Packages" of `tests/MANUAL_TESTS.md`).
- The packages are not signed: Windows SmartScreen warns on the installer until a certificate exists (backlog). ADR-0005's public releases repository becomes a repository setting, not a code change.
- A release takes two runner builds after the tests; pull requests run the packaging only when they change it (`release.yml`, `tauri.conf.json`, the icons), so other pull requests stay as fast as before.
- The AppImage is larger than the 30 MB installed-size target, which applies to the installer and the `.deb`; the workflow reports each size in its summary.
- The Tauri CLI must stay on the Tauri 2 line of the `tauri` crate; it is pinned in the release workflow.
