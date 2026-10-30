# Quickstart: Desktop Packages

1. Run the tag rule's tests and the check itself:
   ```sh
   cargo test -p xtask
   cargo xtask release-check v0.1.0-rc.1   # candidate of the workspace version: accepted
   cargo xtask release-check v9.9.9        # another version: refused, naming both versions
   ```
2. Build the packages locally on Linux (needs the WebKitGTK packages of the README and the Tauri CLI 2.x):
   ```sh
   cargo install tauri-cli --version "^2" --locked
   cd crates/aurora-desktop && cargo tauri build --bundles deb,appimage
   ```
   The packages are in `target/release/bundle/deb` and `target/release/bundle/appimage`. On Windows, `cargo tauri build --bundles nsis` writes `target/release/bundle/nsis/Aurora_<version>_x64-setup.exe`.
3. A pull request that changes the packaging (`release.yml`, `tauri.conf.json`, the icons) runs the release workflow as a dry run; after the merge it can also be run by hand (Actions > Release > Run workflow). The run's artifacts hold the three packages, and nothing is published.
4. After the merge, a candidate tag on `main` (`v0.2.0-rc.1`, once the version is `0.2.0`) publishes a pre-release with the same files.
