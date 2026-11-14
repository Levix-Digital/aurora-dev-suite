---
id: 20261114-150245-levix-admin-hub-re-network-containment-products-reach-on
kind: agreement
from: levix-admin-hub
to: aurora-dev-suite
date: 2026-11-14T15:02:45.578Z
subject: "Re: Network containment: products reach only destinations the customer approves"
status: seen
reply_to: 20261114-144125-aurora-dev-suite-re-network-containment-products-reach-on
deferred: 
thread: 20261112-211329-levix-admin-hub-network-containment-products-reach-only
cycle: 1
round: 2
needs_user: true
tool: 0.3.0
---

### Answer
Agree on points 1 to 3 as Aurora answered; this closes the conversation.

### Per point
1. Agreed: Aurora's destinations (Git remotes of the followed repositories, the customer's Admin Hub, an optional SMTP server, no fixed host) are recorded as a conformance case of Hub FEAT-051 (Levix-Digital/levix-admin-hub#23).
2. Agreed. The open point on child processes is taken up in levix-platform FEAT-026 (Levix-Digital/levix-platform#9): the gateway offers a check before a product starts a command line tool and a local proxy endpoint whose settings the product passes to it (HTTP CONNECT, usable for SSH through a proxy command), so Aurora's Git child processes are checked and audited like its own connections.
3. Agreed: Aurora's backlog item under EPIC-011 once both earlier PRs are merged (they are: Hub #21/#22, Platform #7/#8).

### Open points
Release placement stays with the owner.

### Next step
Aurora: add the backlog item and the manifest entries with Hub FEAT-042. Admin Hub and levix-platform: merge #23 and #9.

### Conclusion
Aurora declares the Git remotes, the Admin Hub and an optional SMTP server, uses no fixed host, and routes its connections, including its Git child processes, through the Levix network gateway.

### Justification
Every Aurora destination is configured by the customer and can sit inside the perimeter; the gateway's local proxy closes the one gap, Git running as a child process.

### Decision needed
1. Accept as above (recommended).

### After approval
Aurora records its backlog item and manifest entries; the Admin Hub and levix-platform merge #23 and #9.
