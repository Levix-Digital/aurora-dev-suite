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
| `runner` | process execution without a shell, standard input closed, time limit with kill, standard output capped at 1 MiB, standard error kept up to 64 KiB; output still open 250 ms after the limit (a child of the engine holding the pipes) counts as a timeout; on Windows the process starts without a console window (`CREATE_NO_WINDOW`) |
| `dcp` | running `dcp` (exit code and runner errors as `CycleError`), checking a reply's `schema` and MAJOR, and its `root`; shared by the cycle and settings adapters |
| `config` | `DcpConfigSource`: runs `dcp config get --json --dir <root>` and `dcp config set --json [--dry-run] --dir <root> --revision <r> --value <key>=<JSON>...`, and reads the `dcp.config` and `dcp.config-set` replies (MAJOR 1) |
| `cycle` | `DcpCycleSource`: runs `dcp diagnose --json --dir <root>` and reads the `dcp.diagnose` reply (MAJOR 1) into the core's `CycleState` |
| `contract` | the version reply (`<engine>.version`, `schema_version` MAJOR 1, `version`, `capabilities`); unknown fields are ignored |
| `probe` | `CliProbe`: locate, ask `version --json`, fall back to `--version` |
| `repo` | repository root discovery (a `.git` directory or file, without running Git), canonical paths without `\\?\` on Windows, case-insensitive comparison on Windows |
| `registry` | `TomlRegistry` over `projects.toml` and `TomlSettings` over `settings.toml`, atomic writes |
| `bin/aurora-fake-engine` | feature `fake-engine`: a test double answering as `dcp`, `gedo` or `actx` after its file name; as `dcp` it also answers `diagnose` |

## Behavior

**Probing an engine.** The time limit from the settings is kept between 100 ms and 120 s. The configured location wins; otherwise the command name is looked up on the search path. A configured location that is not a file, or a command not found, is `Missing`. Then:

1. `<engine> version --json`. A valid reply with MAJOR 1 is `Ready` with its version and capabilities. A reply with another MAJOR is `Degraded` with "update Aurora" (newer) or "update the engine" (older). A wrong schema or missing fields is `Degraded`.
2. When the reply is not a JSON envelope or the command fails, `<engine> --version`. Success is `InstalledWithoutContract`, with the version read from the first line.
3. A timeout, a reply over 1 MiB, a crash or a failing `--version` is `Degraded` with the reason; the standard error is kept in `last_error`.

Nothing else is ever run or read: no scripts, interpreters or engine files (ADR-0006). Each probe logs the command, duration and outcome, never the output.

**Reading a cycle state.** The contract is [`dcp-diagnose.schema.json`](../../specs/002-dcp-adapter/contracts/dcp-diagnose.schema.json), written by Aurora and proposed to the Dev Cycle Protocol; no `dcp` release answers it yet. `DcpCycleSource` runs the `location` of the latest `dcp` report with the `dcp` time limit, passing the project's root as one argument. Then:

| Situation | Result |
|---|---|
| Exit 0 and a valid reply about the same root (canonical comparison, case-insensitive on Windows) | the cycle state, stamped with the time it was read; unknown fields ignored; warnings on standard error ignored |
| Not JSON, not a `dcp.diagnose` envelope, a missing field or a reply about another root | `Invalid`, with the standard error kept |
| Schema MAJOR other than 1 | `Unsupported`, "update Aurora" when newer, "update the engine" when older |
| A timeout, a reply over 1 MiB, a failure to start or a non-zero exit | `TimedOut`, `TooLarge` or `Failed`, with the standard error kept |

Each request logs the root, the duration and the outcome, never the reply.

**The project list.** `projects.toml` and `settings.toml` live in the user's configuration directory (`~/.config/aurora` on Linux, `%APPDATA%\Levix\Aurora\config` on Windows), each with `version = 1`. Writes go to a temporary file renamed over the old one. Adding a folder registers its repository root and refuses folders that do not exist, are outside a repository, or are already registered through any path. A project whose root is no longer a repository is `Missing`. On load:

| Situation | Result |
|---|---|
| No file | empty list |
| Unreadable file | moved to `projects.toml.damaged-<timestamp>`, empty list, `Recovered` notice |
| File from a newer Aurora | left untouched, read-only notice |
| Directory not writable, or a write fails | changes kept for the session, read-only notice |

Unreadable or newer settings fall back to the defaults without touching the file.

**Reading and changing project settings.** The contracts are [`dcp-config.schema.json`](../../specs/007-project-settings/contracts/dcp-config.schema.json) and [`dcp-config-set.schema.json`](../../specs/007-project-settings/contracts/dcp-config-set.schema.json), written by Aurora for the Dev Cycle Protocol (DCP FEAT-017); no `dcp` release answers them yet. `DcpConfigSource` runs `dcp` as the cycle adapter does. A `config get` reply whose setting value does not fit its own type or allowed values is invalid. A `config set` reply must match the dry run asked for; its `errors` become `ConfigError::Rejected`, or `Stale` for `stale_revision`. Each command is logged with the root, the number of keys, dry run or not, applied, the duration and the outcome; values are never logged.

**Tests.** `tests/runner.rs` and `tests/probe.rs` run the fake engine (feature `fake-engine`, so `cargo test --all-features`) in every mode: `ready`, `no-contract`, `unsupported`, `invalid`, `huge`, `hang`, `crash`, `echo` and `orphan`. `tests/cycle.rs` runs it as `dcp diagnose` in those modes and in `idle`, `warn`, `other-root` and `old`. `tests/config.rs` runs it as `dcp config get` and `config set` (it writes the arguments it received next to the binary) in `ready`, `refused`, `stale`, `invalid`, `unsupported`, `hang` and `crash`. `tests/contract_samples.rs` checks the sample replies in `tests/fixtures/dcp-diagnose` and `tests/fixtures/dcp-config` against the schema with `jsonschema` and that they read into the core's sample cycle states, so the fake engine, the fixtures and the preview follow the same contract. The mode comes from a `<binary>.mode` file next to the binary, or `AURORA_FAKE_MODE`. The tests hard-link the binary into Cargo's target directory to avoid "text file busy" on Linux.

## Interfaces

Implements the ports of `aurora-core`. Runs the engines' binaries and reads and writes the registry and settings files.

## Decisions

- [ADR-0003](../adr/0003-engines-integrated-only-through-their-command-lines-with-ver.md): command lines with versioned JSON only.
- [ADR-0006](../adr/0006-no-stand-in-adapters-engines-plugged-in-only-through-their-r.md): no stand-in adapters; engines only through their Rust binaries.
- [ADR-0007](../adr/0007-cross-project-integrations-through-git-on-an-orphan-inbox-br.md): the integrations provider ships with Aurora and plugs into `dcp` through its extension point.
- [ADR-0004](../adr/0004-local-first-state-derived-from-the-repositories.md): the registry and settings files are the only own state.
