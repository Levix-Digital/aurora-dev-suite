# Command line of `dcp-integrations` (MAJOR 1)

Same commands, arguments and text output as the provisional tool 0.3.0 (`aurora/inbox` of aurora-dev-suite, 479da57), plus `init`, `add` and `validate` for `.aurora/integrations.json`. Every command takes `--json` and then answers one document of `dcp-integrations.schema.json` on standard output. The current project is the one whose `.aurora/integrations.json` is in the current directory's repository root, or `--dir <root>`.

| Command | `--json` reply (`schema`) | Exit codes |
|---|---|---|
| `version` | `dcp-integrations.version` (version, schema_major per document, capabilities `messages`, `cache`, `export`, `summary`) | 0 |
| `init --project <slug> [--name <name>]` | `dcp-integrations.status` | 0; 2 file exists |
| `add <slug> --url <repository> [--name <name>]` | `dcp-integrations.status` | 0; 2 slug exists |
| `validate` | `dcp-integrations.status` | 0; 3 invalid |
| `projects` | `dcp-integrations.list` of counterparts | 0 |
| `poll` | `dcp-integrations.poll` | 0 nothing new; 10 news (`--json` still answers) |
| `sync [--max-age S]` | `dcp-integrations.sync` (schema 1.1, capability `cache`): fetches the project's inbox and its counterparts' at once, each unless fetched less than S seconds ago (default 10), and indexes them | 0 |
| `check [--no-sent] [--stale-hours H] [--quiet]` | `dcp-integrations.check` (with `conversations`, most recent first) | 0 |
| `list [--sent] [--all]` | `dcp-integrations.list` | 0 |
| `show <id>` | `dcp-integrations.show` (header, sections, body); marks it seen | 0; 4 unknown id |
| `thread <thread>` | `dcp-integrations.thread` (every message of the thread with its sections, oldest first; from 1.2 each message also has `text`, the body without the status notes, and `notes`, and its sections come from `text`) | 0; 4 unknown thread |
| `export` | `dcp-integrations.export` (schema 1.2, capability `export`): the project's counterparts and every conversation as `check` lists it, with `unread`, its messages as in `thread` and its closing record (`null` without one) | 0 |
| `template <kind>` | `dcp-integrations.template` | 0 |
| `send --to <slug> --kind <kind> --subject <s> [--summary <s>] --body-file <f>` | `dcp-integrations.sent` | 0; 5 refused |
| `reply <id> --kind <kind> [--subject <s>] [--summary <s>] --body-file <f>` | `dcp-integrations.sent` | 0; 5 refused |
| `record <thread>` | `dcp-integrations.record` | 0; 4 no closing record |
| `decide <thread> --choice <text> [--note <text>]` | `dcp-integrations.sent` | 0; 4 |
| `decide <thread> --revise <text>` | `dcp-integrations.sent` (guidance) | 0; 4 |
| `guide <thread> --text <text>` | `dcp-integrations.sent` (guidance) | 0; 4 |
| `status <id> open\|seen\|done [--note <text>]` | `dcp-integrations.status` | 0; 4 |
| `defer <id> --reason <text>` | `dcp-integrations.status` | 0; 4 |
| `watch --target <slug>=<dir> --exec <cmd> [--interval S] [--once]` | none (text only) | 0 |

The reading commands (`projects`, `check`, `export`, `list`, `show`, `thread`, `record`) take `--cached` (capability `cache`, ADR-0011): they read the local copies and fetch only an inbox never fetched before; `sync` keeps the copies current. Writes always fetch first.

Any failure with `--json` answers `dcp-integrations.error` (`code`, `message`, `missing_sections`, `git_stderr`) and exit code 1 for Git or I/O, 2 to 5 as above. Refusals name what is missing; nothing is written on a refusal.

A closing record has `recommended` (1.2): the recommended option, 1-based, from a `Recommended: <n>` line of Decision needed or the first option that ends with "(recommended)"; `null` when none is marked.

A message has `summary` (1.3, capability `summary`, FEAT-039): one or two sentences in the first person, as the sender speaking to the recipient, at most 280 characters, given with `--summary` on `send` and `reply` and written in the front matter as `summary: "<text>"` after the fixed fields (a message without one is written exactly as before). A longer summary is refused, never cut. `decide` and `guide` write one themselves: "The user decided: <choice>" and "The user asks: <first sentence>". Every message header in the replies has `summary`, empty when none was written; readers show the subject then.
