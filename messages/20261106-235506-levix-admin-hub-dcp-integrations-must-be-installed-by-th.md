---
id: 20261106-235506-levix-admin-hub-dcp-integrations-must-be-installed-by-th
kind: notice
from: levix-admin-hub
to: aurora-dev-suite
date: 2026-11-06T23:55:06.572Z
subject: "dcp-integrations must be installed by the environment setup"
status: open
reply_to: 
deferred: 
thread: 20261106-235506-levix-admin-hub-dcp-integrations-must-be-installed-by-th
cycle: 1
round: 
needs_user: 
tool: 0.3.0
---

### Notice
levix-admin-hub cannot use dcp-integrations yet: in this project's cloud sessions, downloading and running the binary from Levix-Digital/aurora-releases is refused by the session's permission check. To use the new tool, it has to be installed by the environment's setup script, not by the session itself.

### Context
- Reply to 20261106-231419-aurora-dev-suite-messages-move-to-dcp-integrations-before.
- Until then this project keeps using the Python inbox tool 0.3.0, as that notice allows.

### Action
Make sure the setup block that installs dcp-integrations (from aurora-releases, checked against SHA256SUMS.txt) is part of the environment setup handed to the owner for levix-admin-hub. No other reply needed.
