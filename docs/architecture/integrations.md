---
title: Integrations Provider
summary: 'dcp-integrations: messages between projects over each repository''s aurora/inbox branch, with its format, conversation rules, Git transport and command line.'
code: [crates/aurora-integrations]
---

# Integrations Provider

dcp-integrations: messages between projects over each repository's aurora/inbox branch, with its format, conversation rules, Git transport and command line.

## Responsibilities

- Read and write the messages between projects on each repository's orphan `aurora/inbox` branch, with Git plumbing on bare caches, never touching a working tree.
- Hold the conversation rules (ADR-0009): kinds, templates, rounds, cycles, the closing record and who presents it to the user.
- Read the project's own registry, `.aurora/integrations.json`, and write it only for `init` and `add`.
- Answer every command in text and as versioned JSON (`--json`) for Aurora and other tools.
- It owns no user interface and makes no decision: the user decides, through Aurora or the command line.

## Structure

| Module | Holds |
|---|---|
| `format` | the message file: front matter (`id`, `kind`, `from`, `to`, `date`, `subject`, `status`, `reply_to`, `deferred`, `thread`, `cycle`, `round`, `needs_user`, `tool`), the body in `##` or `###` sections, parse and render; reads the files of the provisional tool 0.1 to 0.3 |
| `rules` | kinds that open and reply, the templates and their required sections, the current cycle, the next round, the round limit, the opener of a thread, the records waiting for the user, the closing record and its numbered options |
| `registry` | `.aurora/integrations.json` (`aurora.integrations` 1.0): the project's slug and repository and its counterparts; `init`, `add`, `load`, atomic `save` |
| `transport` | `ls-remote` without fetching, fetch into `<cache>/<project>.git`, `inbox.json` MAJOR 1 checked before reading or writing, writes through a temporary index and `commit-tree`, pushed and rebuilt on the new tip up to 8 times when a push loses a race |
| `commands` | every command and its `--json` document (`dcp-integrations.<name>` 1.0), refusals with the missing sections, `watch` once per pass |
| `bin/dcp-integrations` | argument parsing by hand, `--dir`, `--json`, `--verbose`, exit codes |

```mermaid
flowchart LR
    CLI["dcp-integrations (bin)"] --> CMD[commands]
    CMD --> R[rules]
    CMD --> REG["registry (.aurora/integrations.json)"]
    CMD --> T[transport]
    T --> F[format]
    T --> G["git (bare caches)"]
    G --> B["aurora/inbox of each repository"]
```

## Behavior

**The branch.** Each repository's `aurora/inbox` branch holds only what that project received: `README.md`, `inbox.json` (`{"schema": "aurora.inbox", "schema_version": "1.0", "writer": ...}`) and `messages/<id>.md`. A branch written only by the provisional tool has no `inbox.json` and reads as 1.0; the provider adds it on its next write. Another MAJOR is refused for reading and for writing, with "update dcp-integrations".

**Reading.** A project sees what it received (its own inbox) and what it sent (its counterparts' inboxes, filtered by `from`). Messages are ordered by date, then id. Dates carry milliseconds; ids are `<YYYYMMDD-HHMMSS>-<from>-<slug>`, with a 4-hex suffix when taken.

**Writing.** Commits use the project's Git identity (`user.name`, `user.email` of the folder), or "dcp-integrations". A refusal writes nothing: an opening kind that is not question, proposal or notice, a reply kind that is not one, empty required sections, a message to itself, an unknown project, a fourth round. Reading a message (`show`) marks it seen.

**Rounds and cycles.** A round is one message from each side; `ack`, `notice`, `decision` and `guidance` are outside the rounds. After three rounds of the current cycle only an agreement or an impasse is accepted. A `guidance` (the user's request, from `guide` or `decide --revise`) starts the next cycle and closes the open records; the rounds count again from one.

**The user's turn.** An agreement or impasse sets `needs_user`. It waits for the user in the inbox of the project that opened the thread until a `decision` or a `guidance` follows it. `record` builds the closing record (projects, rounds, cycle, conclusion, justification, positions, open points, decision needed with its numbered options, after approval, the user's requests); `decide --choice` sends a `decision` to every other side and closes the record.

**Poll.** `poll` compares the inbox branch's remote tip (`ls-remote`) with the last tip seen, kept in the cache, and fetches only when it changed; it exits with 10 when there is news. `check` records the tip too, and lists the records waiting for the user, the pending messages, the sent messages still unread after the stale time and every conversation, most recent first.

**Logs.** With `--verbose`, Git's error output goes to standard error. Bodies are never logged.

**Tests.** Unit tests cover the format (samples of tool 0.1 to 0.3), the rules and the registry. `tests/scenarios.rs` runs on bare repositories in a temporary folder: the transport (missing branch, first write, identity, a lost race rebuilt, `inbox.json` MAJOR 2 refused, an unreachable remote), a whole conversation from question to decision through a revision in cycle 2, `guide`, `check` with unread sent messages, `init`, `add`, `validate`, `watch` once, and the binary's exit codes. Every `--json` reply and every written file is validated against the contracts in `specs/008-messages/contracts`.

## Interfaces

The command line of [`contracts/cli.md`](../../specs/008-messages/contracts/cli.md), with the JSON of [`dcp-integrations.schema.json`](../../specs/008-messages/contracts/dcp-integrations.schema.json). Aurora runs it through `ProviderMessagesSource` (engines) and finds it next to its own executable first. It calls `git` only.

## Decisions

- [ADR-0007](../adr/0007-cross-project-integrations-through-git-on-an-orphan-inbox-br.md): messages through Git on an orphan inbox branch per repository.
- [ADR-0009](../adr/0009-conversations-between-projects-in-rounds-that-close-with-a-r.md): conversations in rounds and cycles that close with a record for the user.
- [ADR-0010](../adr/0010-the-integrations-provider-ships-with-aurora-and-the-inbox-br.md): the provider ships with Aurora and alone; each project lists its counterparts; the branch holds only data and its format version.
- [ADR-0003](../adr/0003-engines-integrated-only-through-their-command-lines-with-ver.md): Aurora uses it only through its command line with versioned JSON.
