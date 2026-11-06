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
`version` prints the tool version (0.2.0).

```bash
T=~/.aurora-inbox-tool/tool/inbox.py
python3 $T poll --quiet                 # one git ls-remote, no fetch; exit 10 and one line on news
python3 $T check                        # waiting for the user, pending, sent and still unread
python3 $T show <id>                    # read it (marks it seen); --json gives the sections as fields
python3 $T template proposal > body.md  # skeleton: question, proposal, answer, counter, agreement, impasse
python3 $T send --to dev-cycle-protocol --kind proposal --subject "..." --body-file body.md
python3 $T reply <id> --kind counter --body-file answer.md   # answer, counter, agreement, impasse, question, ack
python3 $T record <thread>              # the closing record, in the report the user reads
python3 $T decide <thread> --choice "Option 1" [--note "..."]   # the user's decision, to every side
python3 $T status <id> done --note "answered in <reply id>"
python3 $T defer <id> --reason "not part of the current work"
python3 $T list --sent
python3 $T watch --target gedo=~/src/gedo --exec "<local command>" --interval 300
```

The tool works on bare caches in `~/.cache/aurora-inbox/` with Git plumbing:
it never touches a working tree or another branch. A push that loses a race is
rebuilt on the new tip and retried. The only network traffic is Git on
`aurora/inbox` branches.

## When to look at the inbox

`poll --quiet` is cheap (one `git ls-remote`, no fetch, silent when nothing
changed), so it runs at every touchpoint, not only at the start:

1. at the start of each piece of work;
2. before every gate;
3. before opening a pull request;
4. after a merge.

When it exits with 10, run `check`. The generic route for these points is the
Dev Cycle extension point (DCP FEAT-018, cycle points `discovery`,
`after-merge`, `before-release-gate`); until it ships, tool-specific shortcuts
that call the same command (a session-start or prompt hook) are allowed, never
as the only route.

`watch` wakes a project nobody is working on: it polls the listed projects and,
when one has news, runs the user's local command in that project's folder,
with `AURORA_PROJECT` and `AURORA_INBOX_SUMMARY` set. It is optional, runs
wherever the user starts it, and the tool ships no scheduler.

## Rules

1. A message that belongs to the work in progress is handled there: answer with
   `reply --kind ...`, then set it to `done`.
2. A message that does not belong to it is not acted on: `defer` it with the
   reason and tell the person who triages the project's work.
3. Never set `done` without handling the message.
4. One subject per thread. A message says the context, what is asked and what
   closes it, and links files and pull requests instead of copying them.
5. **Rounds.** A round is one message from each side; round 1 is the ask and its
   answer. After three rounds the tool refuses anything but `agreement` or
   `impasse`. Acknowledgements (`ack`), `notice` and `decision` are outside
   rounds and templates, so a short "noted, no reply needed" costs nothing.
6. **No user in the middle.** During the rounds each side answers from what its
   project's records already decide (backlog, roadmap, ADRs, the user's
   standing rules). What it cannot settle becomes an open point of the closing
   record.
7. **The user is always consulted, once.** A thread closes with an `agreement`
   or an `impasse`, which always needs the user. Only the project that opened
   the thread presents the record (`record <thread>`), once; `check` lists
   these first. The user's answer goes to every side with `decide`, and each
   project records it in its own backlog. No project acts on an agreement
   before the decision arrives.

## Message format

Header, written by the tool:

```markdown
---
id: 20261106-120000-aurora-dev-suite-settings-view
kind: proposal        # question, proposal, notice | answer, counter, agreement, impasse, ack | decision
from: aurora-dev-suite
to: dev-cycle-protocol
date: 2026-11-06T12:00:00Z
subject: "One line that says what is needed"
status: open          # open, seen, done
reply_to:
deferred:
thread: 20261106-120000-aurora-dev-suite-settings-view   # id of the first message
round: 1              # 1 to 3; empty outside rounds
needs_user:           # true on agreement and impasse
tool: 0.2.0
---
```

Messages written by tool 0.1.x have `kind: message` and no thread; each is its
own thread.

Body, checked on `send` and `reply` (`##` or `###` headings; the required ones
must not be empty):

| Kind | Sections (required in bold) |
|---|---|
| question | **Ask**, **Context**, Constraints, **Closes when**, Authority |
| proposal | **Ask**, **Context**, **Proposal**, Constraints, **Closes when**, Authority |
| answer | **Answer**, Per point, Open points, **Next step** |
| counter | **Answer**, **Per point**, Open points, **Next step** |
| agreement | **Answer**, Per point, Open points, Next step, **Conclusion**, **Justification**, **Decision needed**, **After approval** |
| impasse | as agreement, plus **Positions** |
| ack, notice, decision | free |

The last sections of an agreement or impasse are the report the user reads,
written for the user, not for the other project. `record <thread>` prints it:

```text
Discussed with: aurora-dev-suite, dev-cycle-protocol (2 round(s))
Subject: ...
Outcome: agreement

Conclusion: what was agreed, or what remains open.
Justification: why this is the right outcome.
Positions: each project's position and recommendation (impasse).
Decision needed: the options, with the recommended one marked.
After approval: what each project does once the user decides.
```

Whoever presents it translates it into the user's language; the message itself
stays in English.

## Adding a project

Add it to `tool/projects.json` on this branch. Its own `aurora/inbox` branch is
created by the first message sent to it.
