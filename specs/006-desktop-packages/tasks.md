---
description: "Tasks of the Desktop Packages feature (v0.2.0, FEAT-005)"
---

# Tasks: Desktop Packages

**Input**: Design documents from `/specs/006-desktop-packages/`
**Prerequisites**: plan.md, spec.md, research.md
**Tests**: required by the constitution (principle IV.2, evidence); test tasks come before the code they prove.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel (different files, no dependency on an unfinished task)
- **[Story]**: US1 to US3 of spec.md

---

## Phase 1: Foundational (tag rule)

- [ ] T001 [US3] Write the tests of the tag rule in `xtask/src/release.rs`: final and candidate tags of the workspace version accepted; other versions and malformed tags refused with a message naming the tag and the version
- [ ] T002 [US3] Implement the tag rule in `xtask/src/release.rs` and `cargo xtask release-check <tag>` in `xtask/src/main.rs`, reading the workspace version with `cargo_metadata`

**Checkpoint**: `cargo test -p xtask` and `cargo xtask release-check v0.1.0-rc.1` pass.

---

## Phase 2: User Stories 1 and 2 - Install on Windows and Linux (Priority: P1) 🎯 MVP

- [ ] T003 [US1] [US2] Turn on the bundle in `crates/aurora-desktop/tauri.conf.json`: targets `nsis`, `deb`, `appimage`; publisher, copyright, category, descriptions, license file; NSIS per user with the embedded WebView2 bootstrapper; `.deb` dependencies
- [ ] T004 [US2] Build the `.deb` and the AppImage locally on Linux; check the `.deb` dependencies, desktop entry, icon and installed size

---

## Phase 3: User Story 3 - Every release candidate carries its packages (Priority: P1)

- [ ] T005 [US3] Add `.github/workflows/release.yml`: tag check, tests and packages on Windows and Linux, sizes in the run summary, one publish job with `SHA256SUMS.txt` (pre-release for candidates; releases repository as a setting), dry run by hand
- [ ] T006 [US3] Add `.github/workflows/promote.yml` (trunk release gate, version from `Cargo.toml`, environment `release`)
- [ ] T007 [US3] Run the release workflow by hand on the branch and attach the evidence to the pull request

---

## Phase 4: Polish

- [ ] T008 [P] Manual tests MT-PKG-001 to MT-PKG-005 in `tests/MANUAL_TESTS.md`; check with the quality-assurance validator
- [ ] T009 [P] ADR-0008, `docs/architecture/desktop.md`, TECH_SPEC and the README; build the index and check the docs and the README
- [ ] T010 [P] Capture the code signing backlog item (EPIC-001)
- [ ] T011 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features`, `cargo xtask arch-check`, `cargo deny check` and the quickstart
