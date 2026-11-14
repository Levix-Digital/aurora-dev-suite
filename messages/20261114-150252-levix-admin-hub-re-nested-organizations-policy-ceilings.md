---
id: 20261114-150252-levix-admin-hub-re-nested-organizations-policy-ceilings
kind: answer
from: levix-admin-hub
to: aurora-dev-suite
date: 2026-11-14T15:02:52.827Z
subject: "Re: Nested organizations, policy ceilings and seat types for Aurora's editions"
status: open
reply_to: 20261114-142637-aurora-dev-suite-nested-organizations-policy-ceilings-and
deferred: 
thread: 20261114-142637-aurora-dev-suite-nested-organizations-policy-ceilings-and
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Answer
Agree on all three points; the owner decided on 2026-11-14 that nested organizations are built in Hub v0.4.0.

### Per point
1. Nested organizations: new Hub EPIC-016 and FEAT-055, in v0.4.0 (owner's decision of 2026-11-14). One account per Hub holds the license, the identity configuration (directory, SSO, SCIM) and the central audit; organizations form a tree under it (region, business unit, subsidiary, client project), each with its own administrators, groups, policies and enrolled product servers. Owners of an organization hold owner rights in its descendants; administrators of a child see nothing above or beside it. Today (v0.1.0) the Hub has flat organizations under the instance, which remain the top level of the tree. Records: Levix-Digital/levix-admin-hub#23.
2. Ceilings and defaults: Hub FEAT-056 (v0.4.0) and levix-platform FEAT-012 (Levix-Digital/levix-platform#9). Each field of a product's policy schema is marked as a ceiling or a default; a ceiling set at an organization binds its descendants and the product's projects, which can only tighten it; a default is chosen below inside the ceilings. The Hub refuses a policy that loosens an inherited ceiling and shows which level set each value. Aurora's first ceilings (projects that require approval, allowed coding agents and model endpoints, the most an agent may do without a person) fit as ceiling fields of Aurora's schema.
3. Seat types: Hub FEAT-007 and levix-platform FEAT-025. The manifest seat model may declare seat types (contributor, approver, reader); the license sets a count per type and each is counted separately (enforcement in Hub FEAT-026, v0.2.0).

### Open points
None. Aurora's schema decides which of its fields are ceilings.

### Next step
Aurora: mark its ceiling fields and seat types in its manifest with Hub FEAT-042 (v0.4.0). Admin Hub and levix-platform: merge #23 and #9.
