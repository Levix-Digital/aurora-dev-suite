# Command line of `dcp-integrations` (MAJOR 1)

Same commands, arguments and text output as the provisional tool 0.3.0 (`aurora/inbox` of aurora-dev-suite, 479da57), plus `init`, `add` and `validate` for `.aurora/integrations.json`. Every command takes `--json` and then answers one document of `dcp-integrations.schema.json` on standard output. The current project is the one whose `.aurora/integrations.json` is in the current directory's repository root, or `--dir <root>`.

| Command | `--json` reply (`schema`) | Exit codes |
|---|---|---|
| `version` | `dcp-integrations.version` (version, schema_major per document, capabilities `messages`) | 0 |
| `init --project <slug> [--name <name>]` | `dcp-integrations.status` | 0; 2 file exists |
| `add <slug> --url <repository> [--name <name>]` | `dcp-integrations.status` | 0; 2 slug exists |
| `validate` | `dcp-integrations.status` | 0; 3 invalid |
| `projects` | `dcp-integrations.list` of counterparts | 0 |
| `poll` | `dcp-integrations.poll` | 0 nothing new; 10 news (`--json` still answers) |
| `sync [--max-age S]` | `dcp-integrations.sync` (schema 1.1, capability `cache`): fetches the project's inbox and its counterparts' at once, each unless fetched less than S seconds ago (default 10), and indexes them | 0 |
| `check [--no-sent] [--stale-hours H] [--quiet]` | `dcp-integrations.check` (with `conversations`, most recent first) | 0 |
| `list [--sent] [--all]` | `dcp-integrations.list` | 0 |
| `show <id>` | `dcp-integrations.show` (header, sections, body); marks it seen | 0; 4 unknown id |
| `thread <thread>` | `dcp-integrations.thread` (every message of the thread with its sections, oldest first) | 0; 4 unknown thread |
| `template <kind>` | `dcp-integrations.template` | 0 |
| `send --to <slug> --kind <kind> --subject <s> --body-file <f>` | `dcp-integrations.sent` | 0; 5 refused |
| `reply <id> --kind <kind> [--subject <s>] --body-file <f>` | `dcp-integrations.sent` | 0; 5 refused |
| `record <thread>` | `dcp-integrations.record` | 0; 4 no closing record |
| `decide <thread> --choice <text> [--note <text>]` | `dcp-integrations.sent` | 0; 4 |
| `decide <thread> --revise <text>` | `dcp-integrations.sent` (guidance) | 0; 4 |
| `guide <thread> --text <text>` | `dcp-integrations.sent` (guidance) | 0; 4 |
| `status <id> open\|seen\|done [--note <text>]` | `dcp-integrations.status` | 0; 4 |
| `defer <id> --reason <text>` | `dcp-integrations.status` | 0; 4 |
| `watch --target <slug>=<dir> --exec <cmd> [--interval S] [--once]` | none (text only) | 0 |

The reading commands (`projects`, `check`, `list`, `show`, `thread`, `record`) take `--cached` (capability `cache`, ADR-0011): they read the local copies and fetch only an inbox never fetched before; `sync` keeps the copies current. Writes always fetch first.

Any failure with `--json` answers `dcp-integrations.error` (`code`, `message`, `missing_sections`, `git_stderr`) and exit code 1 for Git or I/O, 2 to 5 as above. Refusals name what is missing; nothing is written on a refusal.
