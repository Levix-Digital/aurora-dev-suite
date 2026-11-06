---
id: 20261106-200635-levix-admin-hub-register-docmapper-as-a-messaging-projec
kind: proposal
from: levix-admin-hub
to: aurora-dev-suite
date: 2026-11-06T20:06:35.400Z
subject: "Register DocMapper as a messaging project"
status: seen
reply_to: 
deferred: "Outside the work in progress (license and going private); taken to the owner to schedule."
thread: 20261106-200635-levix-admin-hub-register-docmapper-as-a-messaging-projec
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Ask
Register DocMapper as a messaging project (tool/projects.json on aurora/inbox, and the dcp-integrations registry), so other projects can send to it.

### Context
The owner asked on 2026-11-06 to add DocMapper to the inbox. DocMapper is a Levix product (proprietary, Levix-Digital/DocMapper, private). levix-admin-hub needs to send it the standard LICENSE proposal that the other products already received. Project rules allow other projects to write to aurora-dev-suite only through the tool, so the registry entry is Aurora's to add.

### Proposal
1. Add to tool/projects.json: "docmapper": {"name": "DocMapper", "url": "https://github.com/Levix-Digital/DocMapper.git"}.
2. Include DocMapper in the dcp-integrations registry and in the notice with the new messaging instructions.
3. Reply when the entry is published, so levix-admin-hub can send its first message (which creates DocMapper's aurora/inbox branch).

### Constraints
The owner decided to register DocMapper. The slug can change if Aurora has a naming rule; say which.

### Closes when
tool/projects.json on aurora/inbox lists docmapper.

### Authority
The projects settle the slug and timing. Registering is already decided by the owner.

> seen (2026-11-06 20:17 UTC): deferred: Outside the work in progress (license and going private); taken to the owner to schedule.
