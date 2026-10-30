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

Aurora is built from source for now; there are no installers yet.

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

The application is `target/release/aurora` (`target\release\aurora.exe` on Windows). Copy it wherever you like.

## Quick Start

1. Start `aurora`. The Portfolio is empty the first time.
2. Click **Add a project…** and choose a Git repository (or a folder inside one), or type its path and press **Add**.
3. Press **Alt+3** to open **Engines** and see which engines Aurora found and what they enable.

## Usage

### Portfolio (Alt+1)

The repositories you follow, each with its development cycle at a glance: the phase (release candidate, implementation, review, planning or idle), the current release and how many things wait for you (a release candidate, pull requests that need action, blocking or untriaged bugs). The states are read through the Dev Cycle Protocol (`dcp`), several projects at a time, and each row fills in as its answer arrives; **Refresh** reads them again. Without a `dcp` that answers its JSON contract, the Portfolio says what to install and the rows say the cycle state is unavailable.

A folder inside a repository registers the repository's root. A repository already in the list is refused, even through another path. **Rename** changes only the name Aurora shows. **Remove** stops following the repository and never changes it on disk. A project whose folder was moved or deleted is shown as missing; **Locate…** points it to the new path.

### Project (Alt+2) and Inbox (Alt+4)

One project's development cycle, and the decisions waiting for you across all projects. Both need the Dev Cycle Protocol and are not available yet; until then they say what they will show and which release brings them.

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

To update, build the new version and replace the binary; your project list and settings stay. To uninstall, delete the binary, and to remove Aurora's data too, delete its configuration folder (above) and its data folder (`~/.local/share/aurora` on Linux, `%LOCALAPPDATA%\Levix\Aurora` on Windows). Your repositories are never changed.

## Troubleshooting

| Problem | Fix |
|---|---|
| On Linux, `aurora` stops with `libwebkit2gtk-4.1.so.0: cannot open shared object file` | Install WebKitGTK 4.1: `sudo apt-get install libwebkit2gtk-4.1-0` (Debian, Ubuntu) or `sudo dnf install webkit2gtk4.1` (Fedora) |
| On Windows, the window does not open | Install the Microsoft Edge WebView2 Runtime from Microsoft's website |
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
