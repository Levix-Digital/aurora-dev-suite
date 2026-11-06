# Research: Messages between projects

Decisions taken while planning, with the alternatives considered. ADR-0009 and ADR-0010 hold the architectural ones; these are the feature-level choices.

## R1. The provider is a new crate, not part of `aurora-core`

**Decision**: `crates/aurora-integrations`, a library (format, rules, Git transport, commands) and a binary `dcp-integrations`. Aurora reaches it only through its command line and versioned JSON, like every engine (ADR-0003); `aurora-core` does not depend on it.

**Why**: the provider is an engine used from terminals and work sessions as much as from Aurora (ADR-0010); keeping it behind its command line keeps the desktop and the provider releasable on the same tag without coupling their types.

**Alternatives**: the rules inside `aurora-core` with Aurora writing the branches itself (breaks ADR-0007: Aurora never fetches or pushes); a separate repository (the provider ships with Aurora, so it would need a second release train).

## R2. Git through the `git` command, as the provisional tool does

**Decision**: the provider runs `git` (no shell) on bare caches under the user's cache directory (`directories` crate): `ls-remote`, `fetch` of the one branch, `read-tree`, `hash-object`, `update-index`, `write-tree`, `commit-tree`, `push`, with the retry on a lost race of the provisional tool.

**Why**: the same behaviour already proven on six projects; uses the user's Git credentials, proxies and hosts with no new code; no native dependency.

**Alternatives**: `gix` or `git2` (large dependency trees, credential helpers to reimplement, licences to review under `deny.toml`); a hosting API (breaks ADR-0007).

## R3. Identity and counterparts from `.aurora/integrations.json`

**Decision**: the file names the project's slug, its own repository and its counterparts (`slug`, `name`, `url`, optional `consumes` and `follows` for v0.6.0). `init`, `add` and `validate` manage it; the provider never writes it without one of these commands. The commit author of inbox writes is Git's configured identity in that project (`git config user.name/user.email` read in the project folder), falling back to the provider's name.

**Why**: ADR-0010 (no central registry). Reading the identity from the project keeps "LeviGuilherme" for Levix and the customer's identity for a customer, instead of the provisional tool's hard-coded default.

**Alternatives**: keep `projects.json` (central, ADR-0010 refused it); environment variables only (invisible, easy to get wrong).

## R4. A conversation may span inboxes the project cannot list

**Decision**: a project sees what it received (its own inbox) plus what it sent (its counterparts' inboxes, filtered by `from`), as the provisional tool does. Aurora lists a conversation under its opening project when that project is registered, otherwise under the registered side.

**Why**: same model as ADR-0007 (a conversation is the union of what each side received); no extra index to keep in sync.

## R5. Aurora polls with `poll --json`, at most four projects at once

**Decision**: a timer in the desktop process runs `dcp-integrations poll --json` per project using messages, every N minutes (Settings, default 5, "off" allowed), with the existing limit of four engines at once, and emits an event to the interface only when a poll reports news. Reading a project again (`check --json`, `list --json`) happens only after news, on open and on refresh.

**Why**: `poll` is one `git ls-remote` and fetches nothing when nothing changed (SC-004); the interface stays quiet otherwise.

**Alternatives**: `watch` inside Aurora (it runs a user command; Aurora needs an event instead); a file watcher on the caches (misses remote changes).

## R6. Bodies go through a temporary file

**Decision**: `send` and `reply` take `--body-file`; the adapter writes the body to a file in a new temporary directory, passes its path, and removes the directory afterwards. Bodies are limited to 256 KiB.

**Why**: no shell and no argument length limits on Windows; the body is never in a process list or a log.

## R7. One schema file for every reply

**Decision**: `contracts/dcp-integrations.schema.json` (JSON Schema 2020-12) defines every reply document (`dcp-integrations.version`, `.poll`, `.check`, `.list`, `.show`, `.template`, `.sent`, `.record`, `.status`, `.error`), each with `schema` and `schema_version` "1.0", following `dcp.diagnose`. `aurora-inbox.schema.json` defines `inbox.json`; `aurora-integrations.schema.json` defines `.aurora/integrations.json`. Text output (without `--json`) stays the same as the provisional tool.

## R8. Shipping the provider with Aurora

**Decision**: Tauri's `bundle.externalBin` with `binaries/dcp-integrations`; the release workflow builds `aurora-integrations --release` for the job's target, copies it as `dcp-integrations-<target triple>[.exe]` into `crates/aurora-desktop/binaries/`, then builds the bundles; the same binary is uploaded to the release as `dcp-integrations-<os>-x86_64[.exe]` with its checksum in `SHA256SUMS`. Aurora finds it next to its own executable first, then on `PATH`, then at the location set in Settings (the engine discovery of ADR-0003).

**Why**: the bundler already builds the three packages (ADR-0008) and supports sidecar binaries on every target; one tag ships both.

**Alternatives**: a second installer (two things to install); downloading the provider at first run (breaks the air-gapped desktop).

## R9. Retiring the provisional tool

**Decision**: after the release, the Levix projects run `dcp-integrations init` and `add`, commit `.aurora/integrations.json`, and switch their instructions from `inbox.py` to `dcp-integrations`; the Python tool and `projects.json` are then removed from this repository's `aurora/inbox` branch, keeping its messages. Done after v0.3.0 is released, not in this feature's pull request.
