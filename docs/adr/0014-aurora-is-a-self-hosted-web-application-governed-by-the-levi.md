---
id: 14
title: Aurora is a self-hosted web application governed by the Levix Admin Hub
status: accepted
date: '2026-11-10'
version: null
subsystems: [desktop, ui, core, integrations]
supersedes: [2]
superseded_by: []
amends: [4, 5, 8, 11, 12]
amended_by: [15]
---

# ADR-0014: Aurora is a self-hosted web application governed by the Levix Admin Hub

## Context

On 2026-11-10 Levi set Aurora's audience: it is not a developer tool but the holistic view of every project's phases, of the discussions between projects and of their governance, for leads, architects and managers. Developers work through the Dev Cycle Protocol in their own tools, which keeps working without Aurora.

The integrations coordinator agreed the same day needs people on different machines to see the same state, a real approver identity, immediate e-mail and a round table started when an approver answers. A desktop application with no server (ADR-0002, ADR-0004) could only work around these: a local approver login guarding a personal approval key, signatures to carry identity between machines, e-mail sent from a developer's session.

The Levix Admin Hub is the customer-hosted, zero-knowledge control plane of the Levix products. It already has organizations, accounts with password and TOTP, roles, a hash-chained audit, a Leptos web console on axum, products registered by signed manifests (Hub ADR-0008) and planned approvals. It never stores or processes content (Hub ADR-0004).

The interface is Leptos compiled to WebAssembly behind a backend abstraction and already runs in a browser (the preview with fixtures); the desktop shell is about 1,500 of 25,000 lines.

## Decision

Levi decided on 2026-11-10:

- **Aurora is a self-hosted web application**: one server per organization, inside its own network, never a Levix-hosted service. The core, the engine adapters, the integrations provider and the Leptos interface are kept; an axum server replaces the desktop shell and exposes the same commands over HTTP, with a push channel for events and the messages worker of ADR-0012 running on the server. The interface gets an HTTP backend. Project state stays derived from the repositories (ADR-0004); the server keeps only settings and caches.
- **The Admin Hub governs, Aurora shows and coordinates.** Aurora is a product server enrolled and governed by the Hub (Hub FEAT-017) and registered by its manifest (Hub FEAT-042). Access is granted only in the Hub, by the company's governance people. Aurora's manifest declares the roles **approver** (per project) and **reader**; they reach Aurora in the sign-in token (Hub FEAT-047, the Hub as OpenID Connect provider). Aurora has no user table and no access screens of its own.
- **Aurora keeps every piece of content** (plans, decisions, messages between projects), so the Hub stays zero knowledge.
- **Approvals are signed by the Hub.** Aurora sends the Hub an approval request with metadata only (request id, content digest, projects, approver group per project). The approver reads the decision in Aurora and approves in the Hub with a TOTP step-up. The Hub records it in its audit and returns an attestation signed with the Hub key (Hub FEAT-048), which Aurora writes to the inboxes. The required check at merge verifies it offline against the Hub key fingerprint. This replaces the personal approval keys of the coordinator design. E-mail to approvers is sent by the Hub (Hub FEAT-049).
- **Without a Hub, Aurora is a read-only view**: no approvals and no governance actions. The Dev Cycle Protocol keeps working with neither Aurora nor a Hub.
- **Timing.** v0.3.x ships as desktop. Hub sign-in, approvals and e-mail arrive in Admin Hub v0.4.0 (agreed with the Admin Hub on 2026-11-10); until then the Aurora server uses a single local development account.

## Alternatives Considered

- **Keep the desktop application** (ADR-0002): works offline and needs no hosting, but cannot give several people one shared view, a real login or server-side notifications, which is what leads and approvers need.
- **Desktop plus a small synchronization server**: keeps both the installers and a server, the most work for the least gain.
- **Aurora with its own accounts and approvals**: duplicates what the Admin Hub already does for every Levix product and splits governance in two places.
- **Aurora inside the Admin Hub**: breaks the Hub's zero-knowledge rule, because Aurora reads and shows repository content.
- **A Levix-hosted service**: breaks the promise that content never leaves the organization.

## Consequences

- Supersedes ADR-0002. Amends ADR-0004 (a server inside the organization instead of each person's machine; still no copy of the engines' data), ADR-0005 (the license is carried by the Admin Hub), ADR-0008 (installers give way to a container image and a Linux binary), ADR-0011 and ADR-0012 (local inbox copies and the messages worker live on the server).
- The desktop shell and installers stay until the server ships; no new desktop-only features after v0.3.x.
- New work is the server, the HTTP backend, sign-in, the manifest, approvals through the Hub and server packaging (EPIC-011). Security of a network service (sessions, CSRF, rate limits) becomes part of every review.
- The server needs its own read access to the repositories and write access to `aurora/inbox` only, through plain Git, so it stays agnostic to the Git host.
- Aurora depends on Admin Hub v0.4.0 for governance; the attestation format and the client registration are settled in the Hub's v0.4.0 planning, with Aurora consulted before the contract is frozen.
- The decision holds while Aurora stores no identities of its own and the Hub receives no content: both are checked in review.
