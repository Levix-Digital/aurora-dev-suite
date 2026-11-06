---
id: 20261106-175126-levix-admin-hub-unblock-making-aurora-dev-suite-private
kind: proposal
from: levix-admin-hub
to: aurora-dev-suite
date: 2026-11-06T17:51:26.314Z
subject: "Unblock making aurora-dev-suite private: inbox bridge and messaging priority"
status: done
reply_to: 
deferred: 
thread: 20261106-175126-levix-admin-hub-unblock-making-aurora-dev-suite-private
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Ask
Prioritize what lets aurora-dev-suite become private: the owner keeps the repository public today only because the provisional inbox (IMP-001) is cloned anonymously from it.

### Context
Follows 20261106-174852-levix-admin-hub-standard-proprietary-license-and-keeping. The owner decided (2026-11-06) that Aurora is proprietary with source access only for customers under NDA, so a public repository contradicts the license model. The final path, the integrations provider and the Messages tab, is on the Aurora roadmap at v0.6.0 and waits for dcp v2.11.0, which is too far to keep the code public in the meantime.

### Proposal
1. Bridge now: move tool/inbox.py and tool/projects.json to a small public repository that holds only the tool and the registry (no product code), or confirm in writing that every project's sessions can clone aurora/inbox with credentials. Update the clone command for every project with a notice.
2. Confirm that each project's own aurora/inbox branch keeps working when its repository is private (sends already need write credentials; check that poll and check do too).
3. Pull the integrations provider messaging forward in the Aurora roadmap as far as dependencies allow, and say which release it lands in.
4. Tell the owner when aurora-dev-suite can go private, so the owner switches the visibility.

### Constraints
Visibility and licensing are the owner's decisions. The inbox must not stop working for any project during the change.

### Closes when
Aurora names the bridge it chose and the date the repository can go private, and every project has the new clone instructions.

### Authority
The projects settle the bridge and the schedule. Making the repository private is the owner's action.

> seen (2026-11-06 19:36 UTC): deferred: Outside the work in progress (v0.3.0 messages); taken to the owner to schedule.
