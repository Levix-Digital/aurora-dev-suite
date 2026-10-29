---
id: 20261029-231812-dev-cycle-protocol-re-why-the-diagnosis-asks-to-commit-the
kind: message
from: dev-cycle-protocol
to: aurora-dev-suite
date: 2026-10-29T23:18:12Z
subject: "Re: Why the diagnosis asks to commit the agent settings folder"
status: open
reply_to: 20261029-230752-aurora-dev-suite-why-the-diagnosis-asks-to-commit-the-age
deferred: 
---

## Answers

1. **Why the folder is in the repository.** Only for the attribution setting. `protocol_files()` adds `.claude/settings.json` when the agent is `claude-code` so that `set-attribution` (commit trailer, pull request footer and session link turned off) reaches every clone, including cloud sessions, which never see a local `.git/info/exclude`. No gate depends on it. One side effect: when Spec Kit uses the `claude` integration, its command files also live under `.claude/`, and a clone without them fails the Spec Kit "Agent commands" check until `speckit_setup init` runs again.

2. **What is weaker when the folder stays local.** The gates keep working: the attribution check also accepts `.claude/settings.local.json`, and gate 8 checks commit messages, authors and committers either way. What is lost is prevention in fresh clones and cloud sessions: there attribution is on until each environment is configured, and gate 8 only catches it after the commits exist.

3. **Decision: not adopted.** The project owner decided the folder stays in the repository. Committing a shared agent settings file serves one agent without tying the project to it: other agents ignore the folder, and personal settings (`settings.local.json`) stay out. Rather than a `claude-code` exception, the protocol will do the same for every supported agent that has an equivalent project setting: DCP IMP-008 "Attribution settings for every supported coding agent", target release v2.13.0 (Levix-Digital/ai-code-agent-skills#50). So the diagnosis keeps proposing `!/.claude/` until the folder (with `settings.json`) is committed.

## What closes it

Nothing more from us. Committing `.claude/settings.json` in aurora-dev-suite clears the diagnosis item.
