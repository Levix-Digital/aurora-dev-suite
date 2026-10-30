# Research: Desktop Packages

## R1. Packages built by the Tauri bundler

**Decision**: build the packages with the Tauri CLI (`cargo tauri build`), from the bundle section of `crates/aurora-desktop/tauri.conf.json`: `nsis` on Windows, `deb` and `appimage` on Linux.

**Rationale**: Aurora is a Tauri 2 application (ADR-0002); the bundler already knows its binary, icons, identifier, WebView2 and WebKitGTK needs, and writes the `.deb` dependencies and desktop entry. The Tauri CLI is a build tool, not linked into Aurora; it must stay on the Tauri 2 line of the `tauri` crate.

**Alternatives considered**: `cargo-deb`, `cargo-wix` and a hand-written NSIS script (three tools and three configurations for what the bundler does from one); a zip of the binary (no Start menu entry, no WebView2, no dependencies on Linux).

## R2. NSIS, not MSI, on Windows

**Decision**: one Windows installer, NSIS (`Aurora_<version>_x64-setup.exe`), installed for the current user, with the WebView2 bootstrapper embedded (`embedBootstrapper`).

**Rationale**: FEAT-005 asks for NSIS or MSI. NSIS installs per user without administrator rights, which fits developers' machines; it updates an older version in place; the bundler downloads NSIS itself, while MSI needs the WiX toolset and installs per machine. The embedded bootstrapper is what TECH_SPEC names ("bundled bootstrapper") and works where the download-on-demand bootstrapper is blocked by a proxy.

**Alternatives considered**: MSI (for managed deployments through group policy; added later if a customer needs it); both (two installers to test in every candidate); the offline WebView2 installer (about 150 MB more).

## R3. AppImage and deb on Linux

**Decision**: both, as FEAT-005 says: the `.deb` for Debian and Ubuntu (the reference platform of TECH_SPEC), declaring `libwebkit2gtk-4.1-0` and `libgtk-3-0`; the AppImage for the rest, carrying its libraries.

**Rationale**: the `.deb` stays small and gets WebKitGTK updates from the system; the AppImage runs anywhere without root. The 30 MB installed-size target applies to the installer and the `.deb`; the AppImage is larger because it carries WebKitGTK and GTK, and its size is reported, not gated.

**Alternatives considered**: RPM (Fedora is not a target yet); Flatpak (a runtime and a store review for an internal tool).

## R4. Release and promote workflows of the Dev Cycle (trunk)

**Decision**: add `.github/workflows/release.yml` and `promote.yml` from the Dev Cycle's `devops-release` templates, adapted: the release workflow checks the tag, builds and tests on Windows and Linux, uploads the packages per platform, and one publish job (after both) creates the release with `SHA256SUMS.txt`. The promote workflow reads the version from `Cargo.toml`. The release is published with the GitHub CLI preinstalled on the runners (`gh release create`), not with a third-party action.

**Rationale**: the project uses trunk branching with release candidates on `main` and the promote workflow as the release gate (setup of 2026-10-27), and there is no release workflow yet. Publishing in one job after both builds means a failed platform never leaves a half-published release. `gh` avoids a new third-party action in the supply chain.

**Alternatives considered**: publishing from each build job (a release with only one platform when the other fails); `softprops/action-gh-release` (used by AnyContext; one more third-party dependency); packages on every pull request (several minutes per pull request; the hand-run dry run covers changes to packaging).

## R5. The tag must match the workspace version

**Decision**: `cargo xtask release-check <tag>` accepts `vX.Y.Z` and `vX.Y.Z-rc.N` only when `X.Y.Z` equals the workspace version, and says which kind the tag is; the release workflow runs it first. The rule lives in a pure function with unit tests.

**Rationale**: the packages take their version from `Cargo.toml` (FR-002); a tag that does not match would publish packages whose names and "About" version differ from the release. The repository's own tooling is already `cargo xtask` (`arch-check`).

**Alternatives considered**: a shell check in the workflow (no tests, duplicated in promote); writing the version from the tag at build time (the version would then live in two places).

## R6. Where releases are published

**Decision**: the publish job uses the repository in the `RELEASES_REPOSITORY` variable with the `RELEASES_TOKEN` secret when both are set, and the source repository with the workflow's token otherwise.

**Rationale**: ADR-0005 puts installers in a separate public releases repository, which does not exist yet and needs Levi to create it and a token; until then, the Levix team gets the candidates from the private source repository. Switching later is a setting, not a code change.

**Alternatives considered**: hard-coding `Levix-Digital/aurora-releases` (the workflow fails until it exists); only the source repository (the ADR's public repository would need a workflow change).

## R7. No signing and no updater yet

**Decision**: packages are not signed in v0.2.0; a backlog item (code signing of the Windows installer and the Linux packages) is captured. The updater stays an open question of TECH_SPEC.

**Rationale**: there is no certificate; ADR-0005's "signed binaries" applies to the public releases. The checksum file lets the team verify downloads meanwhile.
