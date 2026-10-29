# Desktop commands (interface ↔ desktop)

The Tauri commands of `aurora-desktop`, one per core use case. Arguments and results are `aurora-core` types (see [data-model.md](../data-model.md)), serialized with serde in `camelCase`. Every command returns `Result<T, CommandError>`, where `CommandError { kind, message }` carries a user-facing message.

| Command | Arguments | Result | Use case |
|---|---|---|---|
| `list_projects` | none | `ProjectList { projects, notice }` (`notice` reports `Recovered` or `ReadOnly`) | List projects (US2) |
| `add_project` | `path`, optional `name` | `AddOutcome` | Add a project (FR-005, FR-007) |
| `pick_and_add_project` | none | optional `AddOutcome` (none when the dialog is cancelled) | Folder dialog, then add |
| `rename_project` | `id`, `name` | `Project` | FR-006 |
| `remove_project` | `id` | nothing | FR-006 |
| `relocate_project` | `id`, `path` | `Project` | FR-009 |
| `engines_overview` | none | `EnginesOverview { reports, features }` from the last probe; starts a probe when none ran yet | US3 |
| `refresh_engines` | none | nothing; emits events | FR-016 |
| `get_settings` | none | `Settings` | FR-003, FR-011 |
| `save_settings` | `Settings` | `Settings` | FR-003, FR-011 |

## Events (desktop → interface)

| Event | Payload | When |
|---|---|---|
| `engine-report` | `EngineReport` | Each engine's probe finishes (in any order) |
| `engines-ready` | `EnginesOverview` | All three probes finished |

## Security

The Tauri capability file allows only these commands and the dialog plugin's `open`; no file system, shell or HTTP plugin is enabled, and the content security policy allows no remote origin.
