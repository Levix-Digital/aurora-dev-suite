---
id: 18
title: Aurora names the whole suite, sold in editions by deployment and governance with a license per seat
status: accepted
date: '2026-11-14'
version: null
subsystems: []
supersedes: []
superseded_by: []
amends: [5]
amended_by: [20]
---

# ADR-0018: Aurora names the whole suite, sold in editions by deployment and governance with a license per seat

## Context

The product vision still described Aurora as a desktop application for developers, with every Levix product proprietary and a freemium bundle of three engines. Since then Aurora became a self-hosted web application for leads, governed by the Levix Admin Hub ([ADR-0014](0014-aurora-is-a-self-hosted-web-application-governed-by-the-levi.md)), and the Dev Cycle Protocol became open source by its own project's decision (2026-11-06).

On 2026-11-14 the user asked for a business proposition like the hosted "software factory" platforms on the market, but on-premises, and an analysis compared one of them (Factory) with the planned end state of the Levix development line: Aurora, the Dev Cycle Protocol, Gedo, AnyContext and the Admin Hub. Its findings: those platforms sell their own agent and metered inference; their control plane stays at the vendor, even in the private deployment; in a fully isolated network their accounts, limits, readiness reports and analytics are switched off; the process and its records live in the platform. The Levix roadmaps, read to the end, already cover the process, the collaboration, the knowledge, the governance and the view, all at the customer. The one missing block is the managed execution of agents.

The user decided the six points below on 2026-11-14.

## Decision

- **Aurora is the name of the whole suite.** No additional name is created. The modules are presented by their job: Process (Dev Cycle Protocol), Collaboration (Gedo), Knowledge (AnyContext), Governance (Admin Hub) and the Aurora application (the console). Each module stays usable on its own.
- **Positioning: the software factory you own.** The customer's agents, models, servers, control plane and records. Aurora is the factory around the agents, not an agent, and nothing is hosted by Levix.
- **Aurora observes and governs first.** Agents run wherever the customer runs them. Managed runs inside the perimeter (a runner enrolled in the Admin Hub that runs any agent under the cycle) are a later epic, after the integrations release, in two steps: one registered runner, then a pool.
- **Editions by deployment and governance, never by usage:** Community (the open source Dev Cycle Protocol, free tiers of the other engines, the readiness report), Team (the Aurora application for one team, up to five machines, with a Levix-signed license file), Enterprise (Aurora with the Admin Hub) and Sovereign (Enterprise for networks with no internet).
- **A license per seat:** contributor seats and lower-priced approver and reader seats, under the annual license of the Hub instance. Never per usage, token or document.
- **Outcome metrics and readiness come first** after the web application: the cycle metrics get their own release, and a readiness report with levels is asked of the Dev Cycle Protocol.
- **A Levix maturity model** of governed autonomy is the base of the sales material and of the pilot.
- **This amends ADR-0005:** Aurora and the Admin Hub stay proprietary, binary-only, with licenses verified offline. "Every Levix product is proprietary" no longer holds: the Dev Cycle Protocol is open source, and each other engine's license is that product's decision. The freemium bundle "priced below the sum of the separate plans" is replaced by the editions above.

## Alternatives Considered

- **A new umbrella name** (for example around "software factory"): rejected by the user; one more name over a suite of named products would confuse the customer.
- **Building managed execution now:** it would put Levix in the race between agents before the governance and the view are delivered; every building block it needs (role runners in the Dev Cycle Protocol, product servers, model endpoints, quotas and network containment in the Admin Hub) is planned for later anyway.
- **Pricing by usage or credits:** it is the most criticised point of the hosted platforms and contradicts the Admin Hub's business model.
- **A Levix-hosted edition:** excluded by the Admin Hub's non-goals and by the market the suite is for.

## Consequences

- `docs/product/VISION.md` is rewritten: users (leads first), value proposition, business model, scope and non-goals.
- The backlog gains EPIC-012 (Outcomes and readiness) and EPIC-013 (Managed runs inside the perimeter), and items for initiatives, entry points, the recipe catalogue and the policy view; the roadmap gains a release for the outcome metrics right after the web application.
- Requests go to other projects by message: to the Dev Cycle Protocol (keep the metrics commands in dcp v2.12.0; a scored readiness report) and to the Admin Hub (nested organizations).
- The license texts and notices of the Aurora repository do not change.
- Analysis: `2026-11-14-factory-as-a-model-for-aurora.md` in the project's shared files.
