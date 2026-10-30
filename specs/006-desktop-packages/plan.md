# Implementation Plan: Desktop Packages

**Branch**: `feat/006-desktop-packages` | **Date**: 2026-10-30 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/006-desktop-packages/spec.md`

## Summary

Turn on the Tauri bundler for Aurora: a Windows NSIS installer (per user, embedded WebView2 bootstrapper) and Linux `.deb` and AppImage packages, all carrying the workspace version. Add the Dev Cycle's release and promote workflows (trunk model): a `v*` tag is checked against the workspace version by a new `cargo xtask release-check`, built and tested on Windows and Linux, packaged, and published once both platforms succeed, with `SHA256SUMS.txt`; candidates as pre-releases. The workflow also runs by hand as a dry run. The README's installation, update and uninstall sections move to the packages. No application code changes and no new crate.

## Technical Context

**Language/Version**: Rust stable, edition 2024, MSRV 1.95 (unchanged)

**Primary Dependencies**: no new crate. New build tool: the Tauri CLI 2.x (`tauri-cli`, MIT or Apache-2.0), installed in CI with `taiki-e/install-action` (already used); at build time it downloads NSIS (zlib license; its installer stub is part of the Windows installer) and the AppImage tools (`linuxdeploy`, AppImage runtime, MIT)

**Storage**: none (the packages keep the user's files of ADR-0004 where they are)

**Testing**: unit tests of the tag rule in `xtask`; the release workflow's dry run; manual tests of installing, opening, updating and uninstalling on Windows and Linux

**Target Platform**: Windows 10 22H2+ and 11 x64, Linux x86_64 (Ubuntu 22.04+ for the `.deb`)

**Project Type**: desktop application (unchanged)

**Performance Goals**: installed size under 30 MB for the installer and the `.deb` (TECH_SPEC), reported by the workflow

**Constraints**: no signing certificate; GitHub-hosted runners; actions limited to the ones already used plus `actions/upload-artifact` and `actions/download-artifact` (GitHub's own)

**Scale/Scope**: 3 packages per release

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | How this plan complies | Status |
|---|---|---|
| I. Design: one responsibility per unit, explicit errors | The tag rule is one pure function returning a typed result; the workflows only orchestrate | Pass |
| II. Simplicity First | One bundler from the existing Tauri configuration; NSIS only on Windows; the Dev Cycle's templates | Pass |
| III. Hexagonal | No application code changes; `xtask arch-check` unchanged | Pass |
| IV. Development cycle: evidence | The tag rule has unit tests; packages are proven by the dry run and manual tests | Pass |
| IV. Security | No secrets in the repository; the releases token is optional and scoped; checksums published; publish only after both builds | Pass |
| IV. 9. No AI attribution | Plan, code, commits, workflows and pull request carry none | Pass |
| ADR-0005 | Proprietary license in the packages; the public releases repository is a setting (research R6); signing deferred to a backlog item | Pass |

Re-checked after Phase 1 design: no violation.

## Project Structure

### Documentation (this feature)

```text
specs/006-desktop-packages/
├── spec.md
├── plan.md
├── research.md
├── quickstart.md
├── checklists/requirements.md
└── tasks.md
```

No `data-model.md` and no `contracts/`: no data and no command or engine contract changes.

### Source Code (repository root)

```text
crates/aurora-desktop/tauri.conf.json   bundle: targets, metadata, license, NSIS and WebView2, deb
xtask/src/release.rs                    the tag rule (new)
xtask/src/main.rs                       `release-check <tag>`
.github/workflows/release.yml           tag check, build, test, package, publish (new)
.github/workflows/promote.yml           the release gate of the trunk model (new)
README.md                               installation, update, uninstall from the packages
```

**Structure Decision**: same crates; one new `xtask` module and two workflows.

## Complexity Tracking

No violations.

## Dev Cycle Protocol

### Backlog items and release

FEAT-005 Desktop packages (EPIC-001), release v0.2.0 "Portfolio on the dcp contract" (`docs/ROADMAP.md`), its last open item. A new backlog item is captured: code signing of the Windows installer and the Linux packages (EPIC-001, no release yet).

### Tests (quality-assurance)

- **Automated** (`xtask`): the tag rule accepts `v0.2.0` (final) and `v0.2.0-rc.1` (candidate) for version `0.2.0`; refuses a different version, a missing `v`, `-rc` without a number, `-beta.1`, extra parts and an empty tag, each with a message naming the tag and the workspace version.
- **CI**: the existing CI is unchanged and still runs the tests on both platforms; the release workflow runs them again on the tag's commit before packaging.
- **Dry run**: the release workflow run by hand on this branch produces the three packages and the checksum file as run artifacts (evidence in the pull request).
- **Manual tests** (`tests/MANUAL_TESTS.md`, new section "Packages", prefix MT-PKG): MT-PKG-001 Windows installer installs per user, opens from the Start menu with the candidate's version, and uninstalls keeping the data; MT-PKG-002 a newer installer updates an older install and keeps the project list; MT-PKG-003 the `.deb` installs with its dependencies, opens from the applications menu and is removed keeping the data; MT-PKG-004 the AppImage runs; MT-PKG-005 checksums match `SHA256SUMS.txt`.
- **Acceptance criteria**: the scenarios of US1 to US3 and SC-001 to SC-004.

### Documentation (architecture-docs, user-docs)

- New ADR-0008 "Installers built by the Tauri bundler and published by the release workflow" (research R1 to R4, R6): NSIS over MSI, deb and AppImage, publish once both builds pass, the releases repository as a setting.
- Edit in place `docs/architecture/desktop.md` (packaging: bundle configuration, where each package installs) and TECH_SPEC (the Tauri CLI as a build tool in the stack table; the installed-size target applies to the installer and the `.deb`).
- README: Installation (download the package of a release, Windows SmartScreen note, `.deb`, AppImage; building from source kept), Updating and Uninstalling from the packages, Troubleshooting (SmartScreen, AppImage needs FUSE).

### CI, release and observability (devops-release)

- New `release.yml` (tags `v*`, `workflow_call` from promote, `workflow_dispatch` as a dry run) and `promote.yml` (environment `release`). `ci.yml` unchanged.
- Version impact: none by itself; the v0.2.0 candidate needs the workspace version bumped to `0.2.0` in a separate pull request (Phase 8).
- Observability: the workflow writes each package's size and the installed size to the run summary; the published release lists its checksums.
- Repository settings for Levi (not code): the `release` environment with required reviewers; optionally `RELEASES_REPOSITORY` and `RELEASES_TOKEN` for ADR-0005's public releases repository.

### Security, migration and rollback

- The optional releases token is a secret used only by the publish job; the workflow token has `contents: write` only in the jobs that create tags or releases.
- No data migration: packages use the same configuration and data folders as the binary built from source.
- Rollback: revert the pull request (no release has been published by it); a bad release candidate is deleted from the releases page and replaced by the next candidate number.
