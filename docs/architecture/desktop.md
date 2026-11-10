---
title: Desktop Shell
summary: 'The Tauri 2 application: composition root, commands exposed to the interface, engine events, security and logging.'
code: [crates/aurora-desktop]
---

# Desktop Shell

The Tauri 2 application: composition root, commands exposed to the interface, engine events, security and logging.

## Responsibilities

- Build the adapters and wire them into the core at start-up (the composition root).
- Expose each core use case as one Tauri command, and the engine probes as events.
- Own the main window and the log file.
- It holds no product rules and no engine knowledge beyond choosing the adapters.

## Structure

| File | Holds |
|---|---|
| `src/main.rs` | start-up: logging, plugins, state, commands |
| `src/state.rs` | `AppState`: `TomlRegistry`, `TomlSettings`, `CliProbe`, `DcpCycleSource`, `DcpConfigSource`, `ProviderMessagesSource` and the last engines overview |
| `src/commands.rs` | the commands and `CommandError { kind, message, detail, config, messages }` (`detail` holds the `CycleError` of the `cycle` kind, `config` the `ConfigError` of the `config` kind, `messages` the `MessagesError` of the `messages` kind) |
| `src/messages.rs` | the four messages commands, the provider port of the worker (`sync` then `export --cached` per project) and the adaptive timer |
| `src/worker.rs` | the messages worker: one queue of readings and writes, the snapshot in memory and in `messages-snapshot.json` |
| `tauri.release.conf.json` | merged by the release build: `bundle.externalBin` with `binaries/dcp-integrations`, and the Windows installer hooks |
| `windows/path-hooks.nsh` | Windows installer hooks: add the installation folder to the user's `PATH` on install, take it off on uninstall (HKCU, no plugins; left alone when `PATH` is near the NSIS string limit) |
| `src/events.rs` | `probe_all`: one thread per engine |
| `src/logging.rs` | the log file |
| `tauri.conf.json` | window, content security policy, frontend location, packages (`bundle`) |
| `capabilities/default.json` | what the window may call |
| `build.rs` | generates an `allow-<command>` permission per command |

The bin is `aurora`. Without the `custom-protocol` feature it loads the interface from `trunk serve` at `http://localhost:1420`; with it, it embeds `crates/aurora-ui/dist`.

## Behavior

**Commands.** `list_projects`, `add_project`, `pick_and_add_project` (the system folder dialog, opened from Rust), `rename_project`, `remove_project`, `relocate_project`, `engines_overview`, `refresh_engines`, `get_settings`, `save_settings`, `project_cycle_state` (a project's cycle state through `dcp`, using the last engine probe and probing first when none ran), `project_config`, `preview_project_config` and `apply_project_config` (a project's Dev Cycle settings through `dcp config`, the same way), and `messages_snapshot`, `messages_refresh`, `messages_template` and `messages_act` (messages between projects through `dcp-integrations`, the same way; specs/008-messages/contracts/desktop-commands.md). Each runs on Tauri's blocking pool and returns the core's types or a `CommandError`.

**Engines.** `engines_overview` returns the last overview, probing and waiting the first time; concurrent first calls (the Portfolio's parallel reads at start-up) wait for a single probe. `refresh_engines` returns at once and probes in the background. Each engine is probed on its own thread and emitted as `engine-report` when done; `engines-ready` carries the overview once all three finished, so a hanging engine delays only its own report.

**Messages worker.** At start-up a thread (`worker.rs`) loads `messages-snapshot.json` from the configuration folder and becomes the only caller of `dcp-integrations` for messages. It takes jobs from one queue: readings are coalesced, writes (`messages_act`) run in the order they were made and answer their caller, and one reading follows a burst of writes once the queue has been quiet for 300 ms. A reading runs `sync` and then `export --cached` for every registered project (or `check`, `thread` and `record` with a provider without `export`), completes the setups (BUG-009) and builds the next `MessagesSnapshot`: a project that cannot be read keeps its last good inbox and is listed as stale. The snapshot is kept in memory, written to a temporary file and renamed, and when something changed the worker emits `messages-snapshot` with its version, the counts and the projects that changed. `messages_snapshot` answers from memory at once (its first call also probes the provider when no probe ran yet, then asks for a reading); `messages_refresh` waits for a reading. A 1 s timer asks for a reading every `messages_poll_seconds` while the window has the focus and every 60 s otherwise; getting the focus asks for one at once, at most every 20 s (window `Focused` events set `AppState.focused`). With the setting at 0 only the user's actions read ([ADR-0012](../adr/0012-a-messages-worker-in-aurora-owns-every-provider-call-and-ser.md)).

**Security.** The capability file grants only Aurora's commands and event listening; no file system, shell, HTTP or dialog permission reaches the web view. The content security policy allows only the application's own scripts (plus `wasm-unsafe-eval`), styles and the IPC channel, and no remote origin.

**Logging.** `tracing` writes a daily file `aurora.<date>.log` in `logs` under the user's data directory (`~/.local/share/aurora` on Linux, `%LOCALAPPDATA%\Levix\Aurora\data` on Windows), keeping five files. The level comes from `AURORA_LOG` (default `info`). When the window cannot start, the error goes to the log and standard error, with the WebKitGTK package names on Linux. A missing WebKitGTK library stops the binary before `main`, so the loader's message names the library; packages declare the dependency from v0.2.0.

**Packages.** `cargo tauri build` in this crate builds the release interface (`beforeBuildCommand`), the binary with `custom-protocol` and the packages of the `bundle` section in `target/release/bundle`: `nsis/Aurora_<version>_x64-setup.exe` (installed for the current user in `%LOCALAPPDATA%\Aurora`, with a Start menu entry, an uninstaller and the WebView2 bootstrapper embedded), `deb/` (`/usr/bin/aurora`, a desktop entry and icons; depends on WebKitGTK 4.1 and GTK 3) and `appimage/` (the binary with its libraries). The release build first builds `dcp-integrations`, copies it as `binaries/dcp-integrations-<target triple>` and passes `--config tauri.release.conf.json`, so the packages put it next to `aurora` (ADR-0010) and the Windows installer adds its folder to the user's `PATH`, as the `.deb` does with `/usr/bin` (only the release build has these; a plain `cargo tauri build` does not); a plain `cargo build` needs no provider. The version is the workspace version. Packages leave the user's configuration and data folders alone on uninstall. The release workflow builds and publishes them (ADR-0008).

## Interfaces

Called by `aurora-ui` through `window.__TAURI__` (`invoke` and `event.listen`). Calls `aurora-core` and builds `aurora-engines`.

## Decisions

- [ADR-0001](../adr/0001-hexagonal-architecture-with-the-engines-behind-ports.md): the composition root lives here.
- [ADR-0002](../adr/0002-desktop-application-with-tauri-2-and-a-leptos-interface.md): Tauri 2 shell.
- [ADR-0005](../adr/0005-proprietary-binary-only-distribution-with-levix-licenses.md): proprietary binaries; licensing through `levix-platform` later.
- [ADR-0011](../adr/0011-messages-are-read-from-local-inbox-copies-kept-current-by-a.md): the Messages view answers from the provider's copies and a background sync follows every poll.
- [ADR-0012](../adr/0012-a-messages-worker-in-aurora-owns-every-provider-call-and-ser.md): a worker owns every provider call and the view reads a saved snapshot.
- [ADR-0008](../adr/0008-installers-built-by-the-tauri-bundler-and-published-by-the-r.md): packages built by the Tauri bundler and published by the release workflow.
- [ADR-0014](../adr/0014-aurora-is-a-self-hosted-web-application-governed-by-the-levi.md): Aurora becomes a self-hosted web application governed by the Admin Hub; this shell stays until the web server replaces it.
