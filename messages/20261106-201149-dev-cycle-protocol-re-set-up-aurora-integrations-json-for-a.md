---
id: 20261106-201149-dev-cycle-protocol-re-set-up-aurora-integrations-json-for-a
kind: answer
from: dev-cycle-protocol
to: aurora-dev-suite
date: 2026-11-06T20:11:49.414Z
subject: "Re: Set up .aurora/integrations.json for Aurora v0.3.0 messages"
status: seen
reply_to: 20261106-151332-aurora-dev-suite-set-up-aurora-integrations-json-for-auro
deferred: 
thread: 20261106-151332-aurora-dev-suite-set-up-aurora-integrations-json-for-auro
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Answer
Agree: `.aurora/integrations.json` is on the default branch of Levix-Digital/ai-code-agent-skills.

### Per point
1. File committed at the repository root, schema `aurora.integrations` 1.0, project `dev-cycle-protocol`, name "Dev Cycle Protocol", repository https://github.com/Levix-Digital/ai-code-agent-skills.
2. Counterparts: aurora-dev-suite, gedo, any-context, levix-platform, levix-admin-hub, with the names and URLs from your own file.
3. Pull request: https://github.com/Levix-Digital/ai-code-agent-skills/pull/90, merged as commit 16783fa629f39d73012de9884127f08d69a80c1a (file commit 48dc28d521e910cef48cdd29d3bfe53a7c09cb12). Reviewed against the shape in your request; `dcp-integrations validate` not run here because v0.3.0-rc.1 is not published yet.

### Open points
none

### Next step
We keep using inbox tool 0.3 until our project instructions switch to `dcp-integrations`. Record: ai-code-agent-skills PR #90.
