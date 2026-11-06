# Desktop commands added by this feature

Same conventions as `specs/001-foundations/contracts/desktop-commands.md`: arguments and results are `aurora-core` types serialized with serde in `camelCase`; every command returns `Result<T, CommandError>`; a `MessagesError` comes back as `CommandError { kind: "messages", message }` with `messages` holding the serialized error.

| Command | Arguments | Result | Use case |
|---|---|---|---|
| `messages_overview` | none | `Vec<ProjectInbox>` for every registered project | `usecases::messages::overview` (FR-002) |
| `messages_conversation` | `project`, `thread` | `Vec<Message>` oldest first | `usecases::messages::conversation` (US3) |
| `messages_record` | `project`, `thread` | `ClosingRecord` | `usecases::messages::record` (FR-004) |
| `messages_template` | `kind` | `Vec<TemplateSection>` (name, hint, required) | `usecases::messages::template` (FR-006) |
| `messages_act` | `project`, `action` (`UserAction`) | `Vec<MessageHeader>` sent or changed | `usecases::messages::act` (FR-004 to FR-006) |
| `messages_poll` | none | `Vec<PollResult>` | `usecases::messages::poll` (FR-007) |

- They use the `dcp-integrations` report of the latest engine probe, probing first when none ran.
- They run on a blocking worker thread, at most four provider processes at once.
- New event `messages-news` with the `Vec<PollResult>` that changed, emitted by the background poll (research R5).
- The Tauri capability file adds only these commands and the event.
- The browser preview answers them from `FixtureMessages`, which keeps the sample conversations in memory.
