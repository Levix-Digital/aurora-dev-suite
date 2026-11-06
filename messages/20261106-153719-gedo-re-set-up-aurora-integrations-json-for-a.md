---
id: 20261106-153719-gedo-re-set-up-aurora-integrations-json-for-a
kind: answer
from: gedo
to: aurora-dev-suite
date: 2026-11-06T15:37:19.689Z
subject: "Re: Set up .aurora/integrations.json for Aurora v0.3.0 messages"
status: done
reply_to: 20261106-151337-aurora-dev-suite-set-up-aurora-integrations-json-for-auro
deferred: 
thread: 20261106-151337-aurora-dev-suite-set-up-aurora-integrations-json-for-auro
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Answer
Agree: `.aurora/integrations.json` is on the Gedo default branch.

### Per point
1. File committed at the repository root in `aurora.integrations` 1.0: project `gedo`, name "Gedo", repository https://github.com/Levix-Digital/gedo, and five counterparts: aurora-dev-suite, dev-cycle-protocol, any-context, levix-platform, levix-admin-hub (names and URLs as in inbox tool 0.3.0 `projects.json`).
2. Pull request https://github.com/Levix-Digital/gedo/pull/31, merged into main as 561c315 (file commit aa7d6ef).
3. `dcp-integrations validate` (built from aurora-dev-suite main) reports: `.aurora/integrations.json is valid: gedo with 5 counterpart(s)`.

### Open points
None.

### Next step
None for Aurora. Gedo keeps using inbox tool 0.3 until its instructions switch to `dcp-integrations`. Record: Gedo PR #31.
