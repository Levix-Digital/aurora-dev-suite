# Data model: Messages between projects

## On disk and on Git (written only by `dcp-integrations`)

### `.aurora/integrations.json` (project work branch, committed by the user)

| Field | Type | Notes |
|---|---|---|
| `schema` | `"aurora.integrations"` | |
| `schema_version` | `"1.0"` | |
| `project` | slug | `[a-z0-9-]{2,40}`, this project's name in messages |
| `repository` | URL | this project's own repository (where its inbox lives) |
| `counterparts[]` | list | `slug`, `name`, `url`; `consumes[]` and `follows[]` optional, read in v0.6.0 (FEAT-030) |

### `aurora/inbox` branch (orphan, per repository)

| Path | Content |
|---|---|
| `README.md` | what the branch is |
| `inbox.json` | `{"schema": "aurora.inbox", "schema_version": "1.0"}`; absent on branches written only by the provisional tool (read as 1.0) |
| `messages/<id>.md` | one message received by this project |

### Message (`messages/<id>.md`)

Front matter, then the body in sections (`##` or `###`).

| Field | Values |
|---|---|
| `id` | `<YYYYMMDD-HHMMSS>-<from>-<slug>[-<4 hex>]` |
| `kind` | `question`, `proposal`, `notice`, `answer`, `counter`, `agreement`, `impasse`, `ack`, `decision`, `guidance`; `message` for tool 0.1 |
| `from`, `to` | project slugs |
| `date` | ISO 8601 UTC with milliseconds (seconds for tool 0.1 and 0.2) |
| `subject` | one line |
| `status` | `open`, `seen`, `done` |
| `reply_to` | id or empty |
| `deferred` | reason or empty |
| `thread` | id of the first message; empty for tool 0.1 (the id is the thread) |
| `cycle` | 1 + the user's requests so far; empty means 1 |
| `round` | 1 to 3 within the cycle; empty outside rounds |
| `needs_user` | `true` on agreement and impasse |
| `tool` | version of the writer |

Rules (ADR-0009): a round is one message from each side; after three rounds of the current cycle only `agreement` or `impasse`; `ack`, `notice`, `decision` and `guidance` are outside rounds; a `guidance` starts a new cycle; a closing record waits for the user when it is not `done`, no `decision` or `guidance` follows it, and the thread was opened by this project.

### Templates (required in bold)

| Kind | Sections |
|---|---|
| question | **Ask**, **Context**, Constraints, **Closes when**, Authority |
| proposal | **Ask**, **Context**, **Proposal**, Constraints, **Closes when**, Authority |
| answer | **Answer**, Per point, Open points, **Next step** |
| counter | **Answer**, **Per point**, Open points, **Next step** |
| agreement | **Answer**, Per point, Open points, Next step, **Conclusion**, **Justification**, **Decision needed**, **After approval** |
| impasse | as agreement, plus **Positions** |
| ack, notice, decision, guidance | free |

## In Aurora (`aurora-core::domain::messages`, new)

| Type | Fields |
|---|---|
| `MessageKind` | the kinds above, plus `Other(String)` for kinds Aurora does not know (shown as text) |
| `MessageHeader` | the header fields above, typed |
| `Message` | header, `sections: Vec<(String, String)>` in order, `body` |
| `Conversation` | `thread`, `projects`, `subject`, `cycle`, `round`, `last`, `waiting_for_user: bool`, `no_answer_yet: bool`, `user_requests: Vec<String>`, `listed_under: ProjectId` |
| `ClosingRecord` | `thread`, `projects`, `rounds`, `cycle`, `subject`, `outcome` (agreement, impasse), `conclusion`, `justification`, `positions`, `open_points`, `decision_needed`, `options: Vec<String>` (numbered lines of decision needed), `after_approval`, `user_requests` |
| `ProjectInbox` | `project: ProjectId`, `slug`, `counterparts`, `setup: Ready \| NotSetUp { command } \| Unavailable(reason)`, `conversations` |
| `Draft` | `from`, `to`, `kind`, `subject`, `sections` (from the template) |
| `UserAction` | `Decide { thread, choice }`, `Revise { thread, text }`, `Guide { thread, text }`, `Send(Draft)`, `Reply { id, kind, sections }`, `MarkSeen { id }` |
| `MessagesError` | `ProviderMissing`, `Unsupported { found }`, `NotSetUp`, `Refused { message, missing_sections }`, `Git { message, stderr }`, `Timeout`, `InvalidReply` |
| `PollResult` | `project`, `changed`, `summary` |

Checks in the core before any command runs: an empty required section, a kind not allowed for a reply at this point (round limit), an empty user text, a body over 256 KiB.

Settings (`domain/settings.rs`): `messages_poll_minutes: u32` (default 5, 0 = off, kept in `settings.toml` as `messagesPollMinutes`), and the provider's location under the existing engine settings.

Feature (`domain/feature.rs`): `Feature::Messages`, needing engine `EngineKind::Integrations` (`dcp-integrations`, version schema `dcp-integrations.version`) MAJOR 1, release v0.3.0.
