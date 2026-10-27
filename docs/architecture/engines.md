---
title: Engine Adapters
summary: Driven adapters that read the Dev Cycle, Gedo and AnyContext through their Rust binaries' versioned JSON, and store the project list and settings.
code: [crates/aurora-engines]
---

# Engine Adapters

Driven adapters that read the Dev Cycle, Gedo and AnyContext through their Rust binaries' versioned JSON, and store the project list and settings.

## Responsibilities

- Run engine commands safely and parse their versioned JSON.
- Find each engine and report what it is and can do (`EngineProbe`).
- Keep the project list and the settings in TOML files (`ProjectRegistry`, `SettingsStore`).
- It owns no product rules: what to show and when lives in the core.

## Structure

| Module | Holds |
|---|---|
| `runner` | process execution without a shell, standard input closed, time limit with kill, standard output capped at 1 MiB, standard error kept up to 64 KiB; output still open 250 ms after the limit (a child of the engine holding the pipes) counts as a timeout |
| `contract` | the version reply (`<engine>.version`, `schema_version` MAJOR 1, `version`, `capabilities`); unknown fields are ignored |
| `probe` | `CliProbe`: locate, ask `version --json`, fall back to `--version` |
| `repo` | repository root discovery (a `.git` directory or file, without running Git), canonical paths without `\\?\` on Windows, case-insensitive comparison on Windows |
| `registry` | `TomlRegistry` over `projects.toml` and `TomlSettings` over `settings.toml`, atomic writes |
| `bin/aurora-fake-engine` | feature `fake-engine`: a test double answering as `dcp`, `gedo` or `actx` after its file name |

## Behavior

**Probing an engine.** The time limit from the settings is kept between 100 ms and 120 s. The configured location wins; otherwise the command name is looked up on the search path. A configured location that is not a file, or a command not found, is `Missing`. Then:

1. `<engine> version --json`. A valid reply with MAJOR 1 is `Ready` with its version and capabilities. A reply with another MAJOR is `Degraded` with "update Aurora" (newer) or "update the engine" (older). A wrong schema or missing fields is `Degraded`.
2. When the reply is not a JSON envelope or the command fails, `<engine> --version`. Success is `InstalledWithoutContract`, with the version read from the first line.
3. A timeout, a reply over 1 MiB, a crash or a failing `--version` is `Degraded` with the reason; the standard error is kept in `last_error`.

Nothing else is ever run or read: no scripts, interpreters or engine files (ADR-0006). Each probe logs the command, duration and outcome, never the output.

**The project list.** `projects.toml` and `settings.toml` live in the user's configuration directory (`~/.config/aurora` on Linux, `%APPDATA%\Levix\Aurora\config` on Windows), each with `version = 1`. Writes go to a temporary file renamed over the old one. Adding a folder registers its repository root and refuses folders that do not exist, are outside a repository, or are already registered through any path. A project whose root is no longer a repository is `Missing`. On load:

| Situation | Result |
|---|---|
| No file | empty list |
| Unreadable file | moved to `projects.toml.damaged-<timestamp>`, empty list, `Recovered` notice |
| File from a newer Aurora | left untouched, read-only notice |
| Directory not writable, or a write fails | changes kept for the session, read-only notice |

Unreadable or newer settings fall back to the defaults without touching the file.

**Tests.** `tests/runner.rs` and `tests/probe.rs` run the fake engine (feature `fake-engine`, so `cargo test --all-features`) in every mode: `ready`, `no-contract`, `unsupported`, `invalid`, `huge`, `hang`, `crash`, `echo` and `orphan`. The mode comes from a `<binary>.mode` file next to the binary, or `AURORA_FAKE_MODE`. The tests hard-link the binary into Cargo's target directory to avoid "text file busy" on Linux.

## Interfaces

Implements the ports of `aurora-core`. Runs the engines' binaries and reads and writes the registry and settings files.

## Decisions

- [ADR-0003](../adr/0003-engines-integrated-only-through-their-command-lines-with-ver.md): command lines with versioned JSON only.
- [ADR-0006](../adr/0006-no-stand-in-adapters-engines-plugged-in-only-through-their-r.md): no stand-in adapters; engines only through their Rust binaries.
- [ADR-0004](../adr/0004-local-first-state-derived-from-the-repositories.md): the registry and settings files are the only own state.
