# Data Model: Foundations

All types live in `aurora-core::domain`, derive `serde::{Serialize, Deserialize}`, and are shared with the interface. Ids are strings.

## Project

| Field | Type | Rules |
|---|---|---|
| `id` | `ProjectId` (UUID v4 as string) | Unique; assigned by the registry on add |
| `name` | `String` | 1 to 80 characters after trimming; defaults to the root folder's name |
| `root` | `PathBuf` | Canonical repository root (FR-005, FR-007) |
| `added_at` | RFC 3339 date-time string | Set on add |
| `presence` | `Presence` | Derived on every load, never stored |

`Presence`: `Present` or `Missing` (the root no longer exists or is no longer a repository, FR-009).

Validation on add (`ProjectError`): `NotFound`, `NotARepository`, `AlreadyRegistered { name }`, `InvalidName`. A path inside a repository is replaced by the repository root, and the result says so (`AddOutcome { project, used_repository_root: bool }`).

## Registry document (`projects.toml`)

```toml
version = 1

[[projects]]
id = "6f1c…"
name = "gedo"
root = "/home/levi/src/gedo"
added_at = "2026-10-27T18:00:00Z"
```

Load outcomes (`RegistryLoad`): `Loaded(Vec<Project>)`, `Empty` (no file yet), `Recovered { moved_to: PathBuf }` (the file could not be read and was renamed to `projects.toml.damaged-<timestamp>`, FR-010), `ReadOnly { reason }` (the directory cannot be written; changes last for this session).

## Settings (`settings.toml`)

| Field | Type | Default |
|---|---|---|
| `version` | `u32` | `1` |
| `theme` | `ThemeChoice` (`system`, `light`, `dark`) | `system` |
| `engines.<kind>.path` | optional path | none: search path |
| `engines.<kind>.timeout_ms` | `u64` | `10000` |

## Engine

`EngineKind`: `DevCycle`, `Gedo`, `AnyContext`, with the command names `dcp`, `gedo`, `actx`.

`EngineReport` (one per engine, the result of a probe):

| Field | Type | Notes |
|---|---|---|
| `kind` | `EngineKind` | |
| `status` | `EngineStatus` | see below |
| `location` | optional path | Where the binary was found |
| `version` | optional `String` | From the contract or from `--version` |
| `capabilities` | `Vec<Capability>` | Empty without the contract |
| `last_error` | optional `String` | Standard error and the reason of the last failed request, up to 64 KiB |
| `checked_at` | RFC 3339 string | |

`EngineStatus`:

| Status | Meaning |
|---|---|
| `Ready` | Answered the version contract with a supported MAJOR |
| `InstalledWithoutContract` | Found and answered `--version`, but not the contract |
| `Degraded { reason }` | Found, but timed out, crashed, replied too much or invalid output, or reported an unsupported MAJOR (`reason` says which, and whether Aurora or the engine must be updated) |
| `Missing` | Not found on the search path nor at the configured location |

`Capability`: `name: String`, `schema_major: u32`.

## Feature requirements

`Feature`: `Portfolio`, `ProjectView`, `Inbox`, `Agents`, `Knowledge`. The catalog (`aurora-core::capabilities`) maps each feature to the engine and capabilities it needs and the message when they are missing:

| Feature | Engine | Needs | Without it |
|---|---|---|---|
| Portfolio, Project view, Inbox | Dev Cycle | `dcp` capability `diagnose` (schema 1) | "Install the Dev Cycle Protocol (dcp)" |
| Agents | Gedo | `radar`, `scope`, `stream` | "Install Gedo 0.5 or later" |
| Knowledge | AnyContext | `ask`, `workspaces` | "Update AnyContext to a version with --json output" |

`FeatureAvailability { feature, state }` with `state`: `Available`, `Unavailable { missing: Vec<String>, hint }`, `NotInThisRelease { release }`.

## Ports (`aurora-core::ports`)

| Trait | Methods | Implemented by |
|---|---|---|
| `ProjectRegistry` | `load() -> RegistryLoad`, `add(path, name?) -> Result<AddOutcome, ProjectError>`, `rename(id, name)`, `remove(id)`, `relocate(id, path)` | `aurora-engines::registry` (TOML), fixtures |
| `SettingsStore` | `load() -> Settings`, `save(&Settings)` | `aurora-engines::registry`, fixtures |
| `EngineProbe` | `probe(kind, &EngineSettings) -> EngineReport` | `aurora-engines::probe`, fixtures |

`CycleSource`, `CollaborationSource` and `KnowledgeSource` are not added in this release: they would have no caller yet (Simplicity First); v0.2.0 adds `CycleSource` with its first use.
