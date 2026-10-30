# Desktop commands added by this feature

Same conventions as `specs/001-foundations/contracts/desktop-commands.md`: arguments and results are `aurora-core` types serialized with serde in `camelCase`; every command returns `Result<T, CommandError>`.

| Command | Arguments | Result | Use case |
|---|---|---|---|
| `project_cycle_state` | `id` (project id) | `CycleState`; a `CycleError` comes back as `CommandError { kind: "cycle", message }` with `detail` holding the serialized `CycleError` | `usecases::cycle::cycle_state` (FR-004 to FR-009) |

- The command uses the `dcp` report of the latest engine probe; when no probe ran yet it runs one first, as `engines_overview` does.
- It runs on a blocking worker thread; several calls (one per project, from the Portfolio in FEAT-014) run in parallel.
- No new event. The Tauri capability file adds only this command.
- The browser preview answers it from `FixtureCycleSource`.
