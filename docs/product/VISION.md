# Aurora Dev Suite

Aurora is the Levix Digital suite that lets a company run its own governed software factory: the process, the collaboration, the knowledge, the governance and the view of every project, all inside the company's perimeter, over the coding agents and models the company already chose. "Aurora" names the whole suite; its modules are the Dev Cycle Protocol (`dcp`, the process), Gedo (`gedo`, the collaboration on code), AnyContext (`actx`, the knowledge), the Levix Admin Hub (the governance) and the Aurora application itself (the console: a self-hosted web application where leads, architects and managers follow every project, the decisions between projects and the approvals).

The software factory you own: your agents, your models, your servers, your control plane, your records.

## Problem

Coding agents made writing code several times faster, and companies are now asked to turn that speed into a "software factory": a continuous line from a request to verified, shipped and monitored software, with people governing instead of typing. The offers on the market sell this as a hosted platform built around the vendor's own agent:

- **The control plane belongs to the vendor.** Sessions, policies, identity and analytics pass through the vendor's cloud, even in the "private" deployments, where only the agents' execution moves into the customer's network. For organizations whose data cannot leave the perimeter, this is the first objection.
- **The more isolated the deployment, the less product the customer gets.** In fully isolated networks the hosted accounts, usage limits, readiness reports and analytics are switched off, and isolation becomes the customer's own job.
- **The process lives inside the agent.** Plans, validation and approvals are the platform's behaviour and records, not files the company owns; leaving the vendor leaves the code and little else, and an approval is a hook, not evidence an auditor can verify.
- **Cost is metered.** Usage is charged in credits and rate limits, and a second product is needed to keep the bill under control.
- **Leads cannot see across projects.** The state of each project (phase, plans, gates, release candidates, pull requests), what parallel agents are changing and what was decided between projects is scattered over files, hosts and command lines, one repository at a time.

Inside Levix the same problems appear on a smaller scale: several products built in parallel with coding agents, each with its own cycle, and decisions between projects that used to depend on one person relaying messages.

## Target Users

- **Leads, architects and managers (primary users of the Aurora application).** They follow each project's phase, the discussions and decisions between projects and the governance trail, and approve what needs their approval. They do not write the code. Their access is granted in the Admin Hub, with the roles approver (per project) and reader.
- **IT and security administrators (buyers, with the leads).** They need every product inside the perimeter, under the company's identity, policies and audit, including in networks with no internet. They work in the Admin Hub.
- **Developers (users of the modules, not of the application).** They work in their own tools with any coding agent, through `dcp`, Gedo and AnyContext. The Dev Cycle Protocol works fully without Aurora; a developer's cycle depends on Aurora only when the project requires approvals.
- **The Levix Digital team (first customer).** Levix builds its own products through the suite and uses that experience as the first case.
- **Organizations whose data cannot leave the perimeter (market):** legal and accounting offices, health, finance, public sector, defence, critical infrastructure and logistics, and the consultancies and integrators that serve them.

## Value Proposition

- **Governance that survives the air gap.** Identity, approvals, policies, metrics and audit run with no outside connection. In the most demanding deployment the customer gets the whole product, not a reduced one.
- **Your control plane.** Nothing is hosted by Levix. The Admin Hub runs at the customer and never sees content; Aurora keeps all content at the customer.
- **Any agent, any model.** The suite is the factory around the agents, not another agent. It works with the coding agents and the models, local or contracted, that the company already uses, and with any Git host.
- **The process and its records are the company's.** Plans, decisions, metrics and approvals are files in the company's Git repositories. Uninstalling leaves plain repositories with their whole history.
- **Evidence, not trust.** Deterministic gates in the cycle, approvals signed by the Hub and verified offline at merge, and a metadata-only audit that an auditor can check without trusting the vendor or the agent.
- **One view for leads.** Every project's phase, plans, gates and pull requests; the round tables and decisions between projects; what waits for each person's approval; and the outcome numbers (lead time, rework, escaped bugs, where the work went).
- **Collaboration below Git text and across projects.** Gedo merges by meaning and shows parallel agents to each other; the integrations coordinator settles between projects what a change in one means for the others.
- **No meter.** A license per seat. No credits, tokens or usage limits to manage.

## Business Model

- **One name, modules by job.** The offer is Aurora; the modules are Process (Dev Cycle Protocol), Collaboration (Gedo), Knowledge (AnyContext), Governance (Admin Hub) and the Aurora application. Each module stays usable on its own.
- **Editions by deployment and governance, never by usage:**
  - **Community** (free): the Dev Cycle Protocol, which is open source, the free tiers of the other engines and the readiness report.
  - **Team:** the Aurora application for one team, on up to five machines, with a Levix-signed license file verified offline.
  - **Enterprise:** Aurora with the Admin Hub: company sign-in, approvers per project, signed approvals, audit, policies, outcome metrics and initiatives across projects.
  - **Sovereign:** Enterprise for networks with no internet: update mirror, local model serving, network containment, source access for the customer, compliance evidence and priority support.
- **Pricing per seat:** contributor seats (developers whose cycle is governed) and lower-priced approver and reader seats (people in the Aurora application), under the annual license of the Hub instance. Never per usage, token or document. Exact prices follow the first pilots.
- **Licensing:** Aurora and the Admin Hub are proprietary and distributed as signed binaries, with licenses signed by Levix and verified offline through the licensing component of `levix-platform` ([ADR-0005](../adr/0005-proprietary-binary-only-distribution-with-levix-licenses.md)). The Dev Cycle Protocol is open source by its own project's decision; each other engine's license is that product's decision.
- **Sales method:** a maturity model of governed autonomy (from an ad hoc use of agents, to a guided cycle, enforced gates, signed approvals with integration between projects, and measured delivery). A free readiness report places a company's repositories on it; a fixed-scope pilot moves one team up a level and shows the outcome numbers before and after.
- **Services and partners:** installation, training and customer-specific integrations priced as projects; consultancies and integrators as the main channel for on-premises customers.
- **No Levix-hosted service.** Every edition runs at the customer.

The decisions behind this section are recorded in [ADR-0018](../adr/0018-aurora-names-the-whole-suite-sold-in-editions-by-deployment.md).

## Success Metrics

- **Time to a governed project:** a company goes from nothing to its first project under the cycle, visible in Aurora, in one working day, following the published guide.
- **A lead's question answered in seconds:** the state of any project and what waits for the person's approval are found in under 10 seconds.
- **Outcome numbers per project:** lead time from an approved plan to the merge, rework after gates and escaped bugs are shown for every project that runs the cycle, with their trend across releases.
- **Decisions between projects without a relay:** every integration decision reaches its approvers with its record, and its time to close is measured.
- **Complete in an isolated network:** every feature of the Enterprise edition works with no outside connection.
- **Works with what is installed:** Aurora works with any engine version that reports the capabilities it needs, and degrades with a clear message when one is missing.
- **Pilots that convert:** share of pilots that become annual contracts.

## Scope

The complete product, as capability areas:

1. **Web application and packaging:** one self-hosted server per organization with the web interface, packaged as a container image and a Linux binary ([ADR-0014](../adr/0014-aurora-is-a-self-hosted-web-application-governed-by-the-levi.md)).
2. **Governance through the Admin Hub:** sign-in, the approver and reader roles, approvals with signed attestations, policies and license come from the Hub; without a Hub, Aurora is a read-only view.
3. **Project portfolio:** the organization's repositories and, for each, the cycle state from `dcp`: phase, plans and milestones, backlog, roadmap, release candidates, pull requests, gates and blocking bugs.
4. **Decisions:** everything waiting for a person across projects (gates, release candidates, review threads, integration decisions), first as a view and then with actions.
5. **Integrations between projects:** phase checks, round tables, holds and approvals by every involved project, with the messages shown as tables ([ADR-0015](../adr/0015-integrations-coordinator-round-tables-approvals-by-every-inv.md), [ADR-0016](../adr/0016-the-integrations-coordinator-runs-in-dcp-integrations-dcp-gi.md)).
6. **Outcomes and readiness:** cycle metrics per project, release and portfolio; where the work went against the roadmap; attribution of delivered work to backlog items and pull requests; the readiness score of each repository and its trend.
7. **Initiatives:** a named goal that spans several projects, with each project's plan, phase, round tables and approvals on one page.
8. **Agent work view:** the agents running on each repository, their Gedo streams and scopes, and the collisions and contract breaks Gedo reports.
9. **Knowledge:** AnyContext questions and answers with citations over each project's sources.
10. **Policies shown per project:** the ceilings the organization set (which agents and models, what an agent may do without a person) and what each project chose inside them.
11. **Entry points and recipes:** a ticket, a chat message, an alert or a schedule becomes a request of the cycle through optional adapters, with a catalogue of ready recipes.
12. **Managed runs inside the perimeter (later):** a runner enrolled in the Hub that runs any coding agent under the cycle on the customer's capacity, first one runner and then a pool.
13. **Suite installer, updates and licensing:** one installer for the application and the engines, release channels, signed packages and license verification.
14. **Engine integration contract:** Aurora reads the engines only through their command lines with versioned `--json` output and capability discovery ([ADR-0003](../adr/0003-engines-integrated-only-through-their-command-lines-with-ver.md)).

Order: the web application first, then the outcome metrics and readiness, then the integrations with approvals, the agent and knowledge views, and the suite. The roadmap (`docs/ROADMAP.md`) is the only place that assigns items to releases.

## Non-Goals

- A coding agent or a model: Aurora governs and shows the work of the agents the company chose.
- A service hosted by Levix, or any call home.
- Re-implementing what an engine does, or linking the engines' code: Aurora shows and drives the engines through their command lines.
- Making a developer's cycle depend on Aurora: the Dev Cycle Protocol works fully without it, unless the project requires approvals.
- Users, devices, policies and audit of its own: that is the Admin Hub.
- An editor, an IDE or a Git hosting service.
- Metering usage, tokens or documents.
- Tying the suite to one agent, one model provider or one Git host.

## Risks and Assumptions

- **The modules are not ready yet.** Gedo's command line arrives with its v0.4.0, `dcp` is finishing its move to one native binary, AnyContext has no versioned `--json` output, and the Admin Hub is at its foundations. Reduced by versioned contracts agreed with each project, and by Aurora degrading clearly when a capability is missing.
- **Dependence on the Admin Hub's schedule.** Sign-in and approvals wait for Admin Hub v0.4.0. Reduced by the read-only view without a Hub and a single local account for development until then.
- **The outcome numbers and the readiness score depend on `dcp`** (DCP FEAT-015 and FEAT-022 for the metrics, DCP FEAT-033 for the readiness report). Reduced by the agreement of 2026-11-14 with the Dev Cycle Protocol that puts the three in dcp v2.12.0, with Aurora reviewing both JSON contracts before they are frozen.
- **The market is being taught by larger vendors.** Their category ("software factory") and their proof (customers, certifications, partners) are ahead. Assumption: organizations that cannot accept a vendor-hosted control plane are a market those offers serve poorly. Validated with the first pilots.
- **No managed execution at first.** Competing offers run agents by themselves from a ticket; Aurora starts by governing and showing. Reduced by the entry points (optional adapters) and by the managed runs planned for later.
- **Many modules, one name.** A buyer must see one product. Reduced by one installer, one license and one console, and by presenting the modules by their job.
- **One small team.** Reduced by keeping Aurora a view over the engines' contracts and by plugging each engine in as it becomes ready.

## Open Questions

- Exact prices per edition and seat type; after the first pilots.
- What the Team edition includes without a Hub beyond the read-only view.
- The levels of the maturity model, and how they map to the levels and criteria of the readiness report that the Dev Cycle Protocol defines (DCP FEAT-033).
- Which Aurora release takes initiatives, the entry points and the policy view.
- Whether the Admin Hub's organizations can be nested (holding, region, business unit, client project), asked to the Admin Hub project.
