# Aurora Dev Suite

Aurora Dev Suite is the Levix Digital desktop application that brings the Dev Cycle Protocol (`dcp`), Gedo (`gedo`) and AnyContext (`actx`) together in one window, for developers who build software with coding agents.

This first release lays the foundations. You open Aurora on Windows or Linux, register the repositories you follow, and see which engines are installed on your machine and which Aurora features each one enables. The project views fill in as each engine ships its machine-readable contract: the Dev Cycle Protocol first, then Gedo and AnyContext.

## Features

- **Portfolio** of the Git repositories you follow: add them from a folder dialog or a path, rename, remove and locate moved ones; the list is kept between sessions.
- **Engines** view: whether `dcp`, `gedo` and `actx` are installed, their version and capabilities, what each one enables, what to install or update, and the last error output of an engine that failed.
- **Light and dark** Aurora Borealis themes that follow your system, or your choice.
- **Keyboard first**: every view and control works without the mouse.
- **Local only**: Aurora keeps a project list and its settings on your machine, never touches your repositories, and sends nothing over the network.

## Requirements

- Windows 10 22H2 or later, or Windows 11, with Microsoft Edge WebView2 (included in Windows 11 and current Windows 10)
- Linux x86_64 (Ubuntu 22.04 or later, or equivalent) with WebKitGTK 4.1
- To build from source: the Rust toolchain of `rust-toolchain.toml` (installed by `rustup`) and Trunk 0.22

Aurora works without any engine; each one it finds unlocks its features.

## Installation

Download the package for your system from the assets of an Aurora release (release candidates are marked "Pre-release"), and check it against `SHA256SUMS.txt` in the same release (`sha256sum -c SHA256SUMS.txt --ignore-missing` on Linux, `Get-FileHash <file>` in PowerShell).

**Windows**: run `Aurora_<version>_x64-setup.exe`. It installs Aurora for your user, without administrator rights, adds it to the Start menu and installs the WebView2 runtime if it is missing. The installer is not signed yet: when Windows SmartScreen says "Windows protected your PC", choose **More info** and then **Run anyway**.

**Debian and Ubuntu**: install the `.deb` with its dependencies (WebKitGTK 4.1 and GTK 3):

```bash
sudo apt install ./Aurora_<version>_amd64.deb
```

**Other Linux distributions**: make the AppImage executable and run it; it carries its own libraries:

```bash
chmod +x Aurora_<version>_amd64.AppImage
./Aurora_<version>_amd64.AppImage
```

### Building from source

Linux (Debian and Ubuntu), the system libraries first:

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libsoup-3.0-dev librsvg2-dev libayatana-appindicator3-dev
```

Then, on Linux and on Windows, from the repository's root:

```bash
cargo install trunk --locked --version 0.22.1
cd crates/aurora-ui && trunk build --release && cd ../..
cargo build -p aurora-desktop --release --features custom-protocol
```

The application is `target/release/aurora` (`target\release\aurora.exe` on Windows). Copy it wherever you like. To build the packages instead, install the Tauri CLI (`cargo install tauri-cli --locked --version "^2"`) and run `cargo tauri build` in `crates/aurora-desktop`; they are written to `target/release/bundle`.

## Quick Start

1. Start `aurora`. The Portfolio is empty the first time.
2. Click **Add a project…** and choose a Git repository (or a folder inside one), or type its path and press **Add**.
3. Press **Alt+3** to open **Engines** and see which engines Aurora found and what they enable.

## Usage

### Portfolio (Alt+1)

The repositories you follow, each with its development cycle at a glance: the phase (release candidate, implementation, review, planning or idle), the current release and how many things wait for you (a release candidate, pull requests that need action, blocking or untriaged bugs). The states are read through the Dev Cycle Protocol (`dcp`), several projects at a time, and each row fills in as its answer arrives; **Refresh** reads them again. Without a `dcp` that answers its JSON contract, the Portfolio says what to install and the rows say the cycle state is unavailable.

A folder inside a repository registers the repository's root. A repository already in the list is refused, even through another path. **Rename** changes only the name Aurora shows. **Remove** stops following the repository and never changes it on disk. A project whose folder was moved or deleted is shown as missing; **Locate…** points it to the new path.

### Project (Alt+2)

One project's development cycle, opened with **Open** on its Portfolio row or chosen in the view: the phase and what waits for you (the same as its Portfolio row), the gates that wait for your decision (a choice the protocol asks for, a release candidate to accept, the release gate), the recommended next step and its options, the plans in progress with their milestones, the current release and its open items, the release candidate, the open pull requests, and the blocking and untriaged bugs. **Refresh** reads it again. It reads through the Dev Cycle Protocol (`dcp`) and, without a `dcp` that answers its JSON contract, says what to install.

Its **Settings** tab shows the project's Dev Cycle settings, grouped, each with its value, its default and whether it can be changed. Change what you need, press **Preview changes** to see each value before and after and the change to `.dev-cycle.json`, then **Apply**. The change is written by the Dev Cycle Protocol, never by Aurora, and Aurora does not commit it: commit `.dev-cycle.json` yourself. It needs a `dcp` with the `config` capability; without it the tab says what to install.

### Inbox (Alt+4)

Everything that waits for you across all projects, the most urgent first: what blocks a release (blocking bugs, release candidates with failed tests), what waits for your acceptance (release candidates to test or to release), what waits for your review (pull requests with review threads for you) and the bugs to triage. Each item shows its project and detail; **Open** shows its project. Projects that could not be read are listed under "Not read" with the reason, and **Refresh** reads them all again. It reads through the Dev Cycle Protocol (`dcp`) and, without a `dcp` that answers its JSON contract, says what to install.

### Engines (Alt+3)

Each engine is shown as one of:

| Status | Meaning |
|---|---|
| Ready | It answered `<engine> version --json`; its version and capabilities are listed |
| Installed without contract | It answered `--version` but not the JSON contract yet; update it when a version with the contract is out |
| Degraded | It was found but did not answer correctly (timed out, crashed, too much output, or a contract version this Aurora does not know); the reason says whether to update Aurora or the engine, and **Last error output** shows what it printed |
| Missing | It is not on the search path nor at the location set in Settings |

**Refresh** checks again after you install or update an engine; there is no need to restart Aurora.

### Settings (Alt+5)

The theme (follow the system, light or dark), and for each engine a location that wins over the search path and a time limit.

### Keyboard

Tab moves between controls, and the focused one is always outlined. In the navigation, the Up and Down arrows move and Enter opens. Alt+1 to Alt+5 open the views from anywhere.

## Configuration

| Setting | Where | Default | Meaning |
|---|---|---|---|
| Theme | Settings view, `settings.toml` | Follow the system | Light, dark or the system's appearance |
| Engine location | Settings view, `settings.toml` | Search path | The `dcp`, `gedo` or `actx` binary to use |
| Engine time limit | Settings view, `settings.toml` | 10 seconds | How long Aurora waits for an engine's answer |
| Log level | `AURORA_LOG` environment variable | `info` | `debug` adds each engine command and its duration |

Aurora's files are in your configuration folder: `~/.config/aurora` on Linux, `%APPDATA%\Levix\Aurora\config` on Windows. `projects.toml` is the project list and `settings.toml` the settings. Both are plain TOML you can read, back up or delete.

## Updating and Uninstalling

To update, install the package of the new version over the old one: the Windows installer replaces the installed version, `sudo apt install ./Aurora_<version>_amd64.deb` upgrades the package, and a new AppImage replaces the old file. Your project list and settings stay. If you used a binary built from source before, delete it.

To uninstall, use **Settings > Apps > Installed apps > Aurora > Uninstall** on Windows, `sudo apt remove aurora` on Debian and Ubuntu, or delete the AppImage (or the binary built from source). Your project list and settings stay; to remove them too, delete Aurora's configuration folder (above) and its data folder (`~/.local/share/aurora` on Linux, `%LOCALAPPDATA%\Levix\Aurora` on Windows). Your repositories are never changed.

## Troubleshooting

| Problem | Fix |
|---|---|
| On Linux, `aurora` stops with `libwebkit2gtk-4.1.so.0: cannot open shared object file` | Install WebKitGTK 4.1: `sudo apt-get install libwebkit2gtk-4.1-0` (Debian, Ubuntu) or `sudo dnf install webkit2gtk4.1` (Fedora) |
| On Windows, the window does not open | Install the Microsoft Edge WebView2 Runtime from Microsoft's website |
| Windows SmartScreen blocks the installer | The installer is not signed yet: choose **More info**, then **Run anyway** |
| The AppImage does not start and mentions FUSE | Install FUSE 2 (`sudo apt install libfuse2` on Ubuntu 22.04, `libfuse2t64` on 24.04), or run it with `--appimage-extract-and-run` |
| "The saved project list could not be read" | The old file was kept next to the new one as `projects.toml.damaged-<timestamp>`; fix it and rename it back to `projects.toml` with Aurora closed, or start a new list |
| "Changes to the project list will not be kept" | Aurora cannot write its configuration folder, or the list was written by a newer Aurora; check the folder's permissions, or update Aurora |
| An engine is Missing although it is installed | Make sure its folder is on the search path of the session that starts Aurora, or set its location in Settings, then **Refresh** |
| An engine is Degraded with "no answer" | Raise its time limit in Settings, or run `<engine> version --json` in a terminal to see what it does |

Logs are in `~/.local/share/aurora/logs` on Linux and `%LOCALAPPDATA%\Levix\Aurora\data\logs` on Windows, one file per day, the last five kept.

## Documentation

- [Technical documentation](TECDOC.md) for maintainers
- [Product vision](docs/product/VISION.md) and [roadmap](docs/ROADMAP.md)
- [Manual tests](tests/MANUAL_TESTS.md)

## Contributing

Aurora is developed by the Levix Digital team. Work follows the Dev Cycle Protocol: report a problem or an idea in [docs/BACKLOG.md](docs/BACKLOG.md) or as an issue in this repository. While working on the interface, `trunk serve` in `crates/aurora-ui` opens a preview at `http://127.0.0.1:1420` that runs with sample data, without the desktop or any engine.

## License

Proprietary software of Levix Digital - see [LICENSE](LICENSE). All rights reserved.
