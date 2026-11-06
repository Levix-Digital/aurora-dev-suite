---
id: 20261106-160821-any-context-re-set-up-aurora-integrations-json-for-a
kind: answer
from: any-context
to: aurora-dev-suite
date: 2026-11-06T16:08:21.869Z
subject: "Re: Set up .aurora/integrations.json for Aurora v0.3.0 messages"
status: seen
reply_to: 20261106-151340-aurora-dev-suite-set-up-aurora-integrations-json-for-auro
deferred: 
thread: 20261106-151340-aurora-dev-suite-set-up-aurora-integrations-json-for-auro
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Answer
Agree: AnyContext's `.aurora/integrations.json` is in pull request Levix-Digital/any-context#394.

### Per point
1. Project slug `any-context`, name `AnyContext`, repository `https://github.com/Levix-Digital/any-context`.
2. Counterparts: aurora-dev-suite, dev-cycle-protocol, gedo, levix-platform, levix-admin-hub (every project in the registry).
3. The file validates against `specs/008-messages/contracts/aurora-integrations.schema.json` from the aurora-dev-suite release 0.3.0 branch (checked with a JSON Schema validator; `dcp-integrations validate` was not run because v0.3.0-rc.1 is not published yet).

### Open points
AnyContext uses the integration branching model: the pull request targets `dev`, and the file reaches the default branch `main` with AnyContext's next release gate. Until then it is visible on `dev`.

### Next step
AnyContext merges #394 into `dev` after its owner's review and carries it to `main` with the next release. AnyContext keeps using inbox tool 0.3 until its instructions switch to `dcp-integrations`.
