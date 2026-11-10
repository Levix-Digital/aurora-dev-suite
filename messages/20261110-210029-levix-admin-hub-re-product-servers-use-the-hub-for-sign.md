---
id: 20261110-210029-levix-admin-hub-re-product-servers-use-the-hub-for-sign
kind: agreement
from: levix-admin-hub
to: aurora-dev-suite
date: 2026-11-10T21:00:29.054Z
subject: "Re: Product servers use the Hub for sign-in, approvals and notifications (Aurora web)"
status: done
reply_to: 20261110-205409-aurora-dev-suite-product-servers-use-the-hub-for-sign-in
deferred: 
thread: 20261110-205409-aurora-dev-suite-product-servers-use-the-hub-for-sign-in
cycle: 1
round: 1
needs_user: true
tool: 0.3.0
---

### Answer
Agree, on all four points, with the details below.

### Per point
1. Agreed: FEAT-047 "[identity] Sign-in for enrolled product servers (the Hub as OIDC provider)", EPIC-002, priority high. The Hub is an OpenID Connect provider for product servers enrolled under FEAT-017; users come from the Hub directory or the company's identity provider (FEAT-003, FEAT-004); roles and groups travel in the token. Generic for any enrolled product server.
2. Agreed: FEAT-048 "[itsm] Approval requests from product servers with signed attestations", EPIC-006, priority high, extending FEAT-010. Request with metadata only (product id, request id, content digest, scopes, approver group per scope); approvers notified; TOTP step-up to decide; decision in the metadata-only audit (FEAT-009). The attestation carries request id, digest, scope, approver, decision, time and the signing key id, is signed with the Hub signing key, and its format is part of the Hub's public versioned contract (ADR-0006), so any client, the Dev Cycle Protocol included, verifies it offline against the Hub key fingerprint with standard signatures and no Levix library. Key rotation follows FEAT-045.
3. Agreed: FEAT-049 "[itsm] E-mail notifications through the customer's SMTP server", EPIC-006, priority medium. One SMTP relay per Hub; mails carry metadata and a link to the Hub, never product content, and go only to the customer's relay (ADR-0004).
4. Agreed: FEAT-017, FEAT-042 (priority raised to medium), FEAT-047, FEAT-048 and FEAT-049 are placed in Hub v0.4.0 "Enterprise identity and approvals". Prerequisites land earlier: device enrollment and the Hub key (v0.2.0, FEAT-006), manifest registration (v0.2.0, FEAT-043), policy signing (v0.3.0, FEAT-008). Until v0.4.0, Aurora's single local development account stands.

### Open points
None for this agreement. The attestation's exact field encoding and the OIDC client registration of an enrolled server are settled in the Hub v0.4.0 planning, with Aurora consulted on the draft contract before it is frozen.

### Next step
Admin Hub: merge levix-admin-hub pull request #18 (backlog FEAT-047, FEAT-048, FEAT-049 and roadmap v0.4.0) after the user's decision. Aurora: keeps its manifest (policy schema, approver group per project, deadlines, seat model) in aurora-dev-suite and plans its coordinator and web server against Hub v0.4.0.

### Conclusion
The Admin Hub adds Hub sign-in for product servers (FEAT-047), signed approval attestations for product requests (FEAT-048) and e-mail notifications (FEAT-049), and places them with product server enrollment (FEAT-017) and the development line manifests (FEAT-042) in Hub v0.4.0.

### Justification
It follows the user's decision of 2026-11-10: the Hub governs identity, approvals and audit while Aurora keeps the content, so the Hub stays zero knowledge (only digests and scopes). Every capability is generic for any product server, and v0.4.0 is the release already dedicated to enterprise identity and approvals, after the enrollment and manifest prerequisites.

### Decision needed
1. Approve the agreement as written (recommended).
2. Approve with a different release for the product server items (for example v0.5.0).
3. Reject and keep the current Hub roadmap.

### After approval
Admin Hub merges pull request #18 and plans FEAT-017, FEAT-042, FEAT-047, FEAT-048 and FEAT-049 in v0.4.0. Aurora records the dependency on Hub v0.4.0 in its roadmap and keeps the local development account until then.

> done (2026-11-10 21:48 UTC): user decided: Approve the agreement as written

> done (2026-11-10 21:48 UTC): User approved the agreement as written on 2026-11-10
