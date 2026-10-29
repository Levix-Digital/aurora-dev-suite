# aurora/inbox

This branch is the inbox of Aurora Dev Suite for messages from other projects
(ADR-0007). It has no history in common with the code and is never merged. Do
not open pull requests from it, and keep it out of CI and branch rules.

It also carries the provisional messaging tool (IMP-001) used by every Levix
project until the integrations provider exists: `tool/inbox.py` and the
project registry `tool/projects.json`.

## Layout

| Path | Content |
|---|---|
| `messages/<id>.md` | Messages received by this project |
| `tool/inbox.py` | The provisional tool (Python standard library and `git` only) |
| `tool/projects.json` | Every project that can send and receive, with its repository |

Each project's own `aurora/inbox` branch holds only `README.md` and
`messages/`; the tool and the registry live here only.

## Getting the tool

```bash
git clone --quiet --depth 1 --single-branch --branch aurora/inbox \
  https://github.com/Levix-Digital/aurora-dev-suite.git ~/.aurora-inbox-tool
```

## Commands

The current project is `--project`, else `AURORA_PROJECT`, else the file
`~/.aurora-project`, else the registered project whose repository is the
`origin` of the current directory. Every command takes `--json`.

```bash
python3 ~/.aurora-inbox-tool/tool/inbox.py check            # what is pending for this project
python3 ~/.aurora-inbox-tool/tool/inbox.py show <id>        # read it (marks it seen)
python3 ~/.aurora-inbox-tool/tool/inbox.py send --to dev-cycle-protocol --subject "..." --body-file body.md
python3 ~/.aurora-inbox-tool/tool/inbox.py reply <id> --body-file answer.md
python3 ~/.aurora-inbox-tool/tool/inbox.py status <id> done --note "answered in <reply id>"
python3 ~/.aurora-inbox-tool/tool/inbox.py defer <id> --reason "not part of the current work"
python3 ~/.aurora-inbox-tool/tool/inbox.py list --sent
```

The tool works on bare caches in `~/.cache/aurora-inbox/` with Git plumbing:
it never touches a working tree or another branch. A push that loses a race is
rebuilt on the new tip and retried.

## Rules

1. Run `check` at the start of each piece of work in a project.
2. A message that belongs to the work in progress is handled there: answer with
   `reply`, then set it to `done`.
3. A message that does not belong to it is not acted on: `defer` it with the
   reason and tell the person who triages the project's work.
4. Never set `done` without handling the message.
5. One subject per message. A message says the context, what is asked and what
   closes it, and links files and pull requests instead of copying them.

## Message format

```markdown
---
id: 20261029-213000-aurora-dev-suite-settings-view
kind: message
from: aurora-dev-suite
to: dev-cycle-protocol
date: 2026-10-29T21:30:00Z
subject: "One line that says what is needed"
status: open
reply_to:
deferred:
---

## Context
## What is asked
## What closes it
```

## Adding a project

Add it to `tool/projects.json` on this branch. Its own `aurora/inbox` branch is
created by the first message sent to it.
