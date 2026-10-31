# Desktop commands added by this feature

Same conventions as `specs/001-foundations/contracts/desktop-commands.md`: arguments and results are `aurora-core` types serialized with serde in `camelCase`; every command returns `Result<T, CommandError>`.

| Command | Arguments | Result | Use case |
|---|---|---|---|
| `project_config` | `id` (project id) | `ProjectConfig`; a `ConfigError` comes back as `CommandError { kind: "config", message }` with `detail` holding the serialized `ConfigError` | `usecases::config::read` (FR-002, FR-003) |
| `preview_project_config` | `id`, `change` (`ConfigChange`) | `ConfigOutcome` with `applied: false` | `usecases::config::preview` (FR-004, FR-006) |
| `apply_project_config` | `id`, `change` (`ConfigChange`) | `ConfigOutcome` with `applied: true` | `usecases::config::apply` (FR-005, FR-006) |

- The commands use the `dcp` report of the latest engine probe; when no probe ran yet they run one first, as `project_cycle_state` does.
- They run on a blocking worker thread.
- No new event. The Tauri capability file adds only these commands.
- The browser preview answers them from `FixtureConfigSource`, which keeps the sample settings in memory.
