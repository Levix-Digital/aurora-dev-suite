# Feature Specification: Desktop Packages

**Feature Branch**: `feat/006-desktop-packages`

**Created**: 2026-10-30

**Status**: Draft

**Input**: User description: "Windows installer (NSIS or MSI) and Linux packages (AppImage and deb) built in CI for release candidates. Backlog item FEAT-005, release v0.2.0 Portfolio on the dcp contract."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Install Aurora on Windows without building it (Priority: P1)

A Levix developer on Windows downloads Aurora's installer from a release candidate, runs it without administrator rights and finds Aurora in the Start menu; the installer also uninstalls it from the system's apps list.

**Why this priority**: today Aurora is built from source; release candidates cannot be accepted (gate 9) by someone who only wants to run them.

**Independent Test**: Download `Aurora_<version>_x64-setup.exe` from a release candidate, install it, open Aurora from the Start menu, check the version on the sidebar and uninstall it.

**Acceptance Scenarios**:

1. **Given** a release candidate, **When** the developer runs its Windows installer, **Then** Aurora installs for the current user without asking for administrator rights and adds a Start menu entry.
2. **Given** a Windows machine without the WebView2 runtime, **When** the installer runs, **Then** it installs the runtime first with the bootstrapper embedded in the installer.
3. **Given** Aurora installed, **When** the developer opens it, **Then** the sidebar shows the version of the release candidate.
4. **Given** Aurora installed, **When** the developer uninstalls it, **Then** the program is removed and the project list and settings stay unless the developer asks to delete the application data.
5. **Given** an older version installed, **When** the installer of a newer version runs, **Then** it replaces the older one and keeps the project list and settings.

---

### User Story 2 - Install Aurora on Linux (Priority: P1)

A Levix developer on Ubuntu or Debian installs Aurora's `.deb` with the system's package manager, which pulls WebKitGTK; on another distribution the developer runs the AppImage.

**Why this priority**: Linux is a first-class platform (TECH_SPEC); both packages are named in FEAT-005.

**Independent Test**: On Ubuntu 22.04 or later, `sudo apt install ./aurora_<version>_amd64.deb`, open Aurora from the applications menu, then `sudo apt remove aurora`; run the AppImage on the same machine.

**Acceptance Scenarios**:

1. **Given** a release candidate, **When** the developer installs its `.deb`, **Then** the package manager installs WebKitGTK 4.1 and GTK 3 if missing, and Aurora appears in the applications menu with its icon.
2. **Given** the AppImage, **When** the developer makes it executable and runs it, **Then** Aurora opens without installing anything else.
3. **Given** Aurora installed from the `.deb`, **When** the developer removes the package, **Then** the program is removed and the project list and settings stay.

---

### User Story 3 - Every release candidate carries its packages (Priority: P1)

When a release candidate tag `vX.Y.Z-rc.N` is pushed, CI builds the Windows installer and the Linux packages from that tag and publishes them as the assets of a pre-release, with a checksum file. The release gate (promote) publishes the same packages for the final version.

**Why this priority**: FEAT-005 asks for packages built in CI for release candidates; the Dev Cycle's gates 9 and 10 need an installable candidate.

**Independent Test**: Run the release workflow by hand on the feature branch (dry run) and check the four files in the run's artifacts; after the merge, the first candidate tag publishes them in a pre-release.

**Acceptance Scenarios**:

1. **Given** a tag `vX.Y.Z-rc.N` whose version equals the workspace version `X.Y.Z`, **When** it is pushed, **Then** the release workflow builds and tests on Windows and Linux, builds the packages and publishes a pre-release named after the tag with the packages and `SHA256SUMS.txt`, not marked latest.
2. **Given** a tag whose version differs from the workspace version, or with another form, **When** the workflow starts, **Then** it stops before building with a message naming both versions.
3. **Given** the user promotes an accepted candidate, **When** the promote workflow runs, **Then** it tags the candidate's commit `vX.Y.Z` and the release workflow publishes the final release with packages built from that commit.
4. **Given** the workflow is run by hand on a branch, **When** it finishes, **Then** the packages are kept as the run's artifacts and nothing is published.
5. **Given** the project sets a releases repository (ADR-0005), **When** a release is published, **Then** it goes to that repository; without it, to the source repository.

---

### Edge Cases

- A candidate tag pushed again after a failed run: the run fails on the existing release instead of mixing assets; the next candidate number is used.
- A package step fails on one platform: no release is published for the tag (publish runs only after both platforms succeed).
- Installing over a build-from-source binary: the packages install to their own locations; the README says to delete the old binary.
- The installer is not code-signed yet: Windows SmartScreen shows "Unknown publisher"; the README says how to continue, and signing is a backlog item.
- AppImage size: it carries WebKitGTK and GTK, so it is larger than the 30 MB target, which applies to the installed Aurora of the installer and the `.deb`.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Tauri bundle configuration MUST produce a Windows NSIS installer (current-user install, embedded WebView2 bootstrapper, Start menu entry, uninstaller) and Linux `.deb` and AppImage packages, with Aurora's name, identifier, version, publisher, copyright, category, descriptions and the proprietary license.
- **FR-002**: Packages MUST carry the workspace version (`Cargo.toml`); there is no second place where the version is written.
- **FR-003**: A release workflow MUST run on `v*` tags and be callable by the promote workflow; it MUST check that the tag is `vX.Y.Z` or `vX.Y.Z-rc.N` and matches the workspace version before building, with the check in the repository's own tooling (`cargo xtask`) and covered by tests.
- **FR-004**: The release workflow MUST run the tests and build the packages on Windows and Linux from the tag's commit, and publish them only after both platforms succeed: candidates as pre-releases not marked latest, final versions as releases marked latest only when newest.
- **FR-005**: Each release MUST carry `SHA256SUMS.txt` with the checksum of every package.
- **FR-006**: The release workflow MUST be runnable by hand on any branch as a dry run that keeps the packages as run artifacts and publishes nothing.
- **FR-007**: A promote workflow MUST let the user release an accepted candidate (trunk model): it tags the candidate's commit `vX.Y.Z` and calls the release workflow; it runs in the `release` environment so the user can require an approval.
- **FR-008**: The releases repository MUST be configurable (a repository variable and a token secret); without them, releases go to the source repository.
- **FR-009**: The README MUST say how to install, update and uninstall from the packages on Windows and Linux, and keep building from source as an option.

### Key Entities

- **Package**: one installable file per platform and format (`-setup.exe`, `.deb`, `.AppImage`), named with the version.
- **Release**: a tag with its packages and checksums; a pre-release for a candidate.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A release candidate tag produces a pre-release with 3 packages and `SHA256SUMS.txt`, with no manual step.
- **SC-002**: Aurora installed from the Windows installer or the `.deb` takes under 30 MB (TECH_SPEC), measured on the installed files.
- **SC-003**: A developer installs and opens Aurora from a package in under 2 minutes, without a Rust toolchain.
- **SC-004**: 100% of the automated tests pass on Windows and Linux, and a tag that does not match the workspace version never publishes a release.

## Assumptions

- Code signing (Windows Authenticode, Linux package signing) is out of scope: there is no certificate yet; it becomes a backlog item.
- Automatic updates are out of scope (TECH_SPEC open question: Tauri's updater or the shared installer core of `levix-platform`).
- macOS packages come later, with the macOS target (TECH_SPEC).
- MSI is not built: NSIS installs per user without administrator rights and needs no WiX toolset; MSI can be added for managed deployments later.
- The public releases repository of ADR-0005 does not exist yet; until Levi creates it and sets its variable and token, releases go to the private source repository, which is enough for the Levix team.
- The Tauri CLI (`tauri-cli` 2.x, MIT or Apache-2.0) is the build tool of the packages; it downloads NSIS (zlib license) and the AppImage tools (MIT) at build time. No crate is added to `Cargo.lock`.
