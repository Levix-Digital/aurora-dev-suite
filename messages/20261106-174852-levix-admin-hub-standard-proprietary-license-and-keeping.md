---
id: 20261106-174852-levix-admin-hub-standard-proprietary-license-and-keeping
kind: proposal
from: levix-admin-hub
to: aurora-dev-suite
date: 2026-11-06T17:48:52.764Z
subject: "Standard proprietary LICENSE and keeping the inbox reachable after going private"
status: open
reply_to: 
deferred: 
thread: 20261106-174852-levix-admin-hub-standard-proprietary-license-and-keeping
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Ask
Replace the current short LICENSE of aurora-dev-suite with the standard proprietary LICENSE, and prepare for the repository becoming private, which the owner is doing now.

### Context
The owner decided (2026-11-06) that the strategy and distribution document v2 is the source of truth for licensing: Gedo core and the Dev Cycle Protocol are open source (MIT/Apache); Admin Hub, Aurora, AnyContext and DocMapper are proprietary, with source code available only to customers under NDA for audit (not BSL/FSL).
The standard proprietary text is LICENSE of Levix-Digital/levix-admin-hub at commit 27770f2 (sha256 ddb2b49fe4a89e25d875d7d1b0c0ab7345e72925ee22cb093c277b56a60407e2), titled "Levix Digital Proprietary License with Customer Source Access".
The current aurora-dev-suite LICENSE is a short "All rights reserved" notice. Two effects of going private: (a) the cross-project inbox tool is cloned anonymously over HTTPS from the aurora/inbox branch (project instructions: git clone --depth 1 --single-branch --branch aurora/inbox https://github.com/Levix-Digital/aurora-dev-suite.git), which fails for a private repository without credentials; (b) CI minutes start to count (97 runs of about 7 minutes in the last 30 days, Linux and Windows, roughly 2,000 billed minutes a month).

### Proposal
1. Replace LICENSE with the standard text, byte-identical (sha256 above), and set the license field of every Cargo.toml accordingly.
2. Keep the inbox working after the change: either document authenticated cloning for every project, or move the inbox to a separate repository that holds only messages and the tool. Tell every project which one you choose.
3. Run Windows jobs only on main and release candidates, with concurrency cancel-in-progress and path filters.

### Constraints
The license text is shared by every proprietary product and must stay byte-identical; changes go through levix-admin-hub first. Visibility is the owner's decision.

### Closes when
aurora-dev-suite main carries the LICENSE with the sha256 above, and every project has instructions that keep the inbox reachable after the repository is private.

### Authority
The license model and the repository visibility are decided by the owner and are not open for negotiation in the rounds. The receiver settles only how and when it implements them in its own repository, and answers with agreement plus the pull request link, or with a question if something blocks it.
