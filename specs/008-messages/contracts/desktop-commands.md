# Desktop commands added by this feature

Same conventions as `specs/001-foundations/contracts/desktop-commands.md`: arguments and results are `aurora-core` types serialized with serde in `camelCase`; every command returns `Result<T, CommandError>`; a `MessagesError` comes back as `CommandError { kind: "messages", message }` with `messages` holding the serialized error.

| Command | Arguments | Result | Use case |
|---|---|---|---|
| `messages_snapshot` | none | `MessagesSnapshot`: every registered project's conversations with their messages and closing records, from memory | the messages worker (spec 010, FR-004) |
| `messages_refresh` | none | `Vec<PollResult>`: the projects whose conversations changed | the worker: `usecases::messages::sync` and `export_inbox` (spec 010, US4) |
| `messages_template` | `kind` | `Vec<TemplateSection>` (name, hint, required) | `usecases::messages::template` (FR-006) |
| `messages_act` | `project`, `action` (`UserAction`) | `Vec<MessageHeader>` sent or changed | the worker's write queue: `usecases::messages::act` (FR-004 to FR-006, spec 010 FR-005) |

- Spec 010 (IMP-007, ADR-0012) replaced `messages_overview`, `messages_conversation`, `messages_record` and `messages_poll` with `messages_snapshot` and `messages_refresh`: the view never waits for the provider to read.
- `messages_snapshot` answers at once; its first call probes the engines behind it and asks the worker for a reading. `messages_refresh` and `messages_act` wait for the worker.
- The worker uses the `dcp-integrations` report of the latest engine probe and runs at most four provider processes at once.
- Event `messages-snapshot` with `MessagesNews` (version, active conversations, whether one waits for the user, the projects that changed), emitted after each reading that changed the snapshot; it replaced `messages-news`.
- The Tauri capability file adds only these commands and the event.
- The browser preview answers them from `FixtureMessages`, which keeps the sample conversations in memory.
