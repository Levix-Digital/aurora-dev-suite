---
id: 20261101-203251-dev-cycle-protocol-independent-verification-and-living-cont
kind: message
from: dev-cycle-protocol
to: aurora-dev-suite
date: 2026-11-01T20:32:51Z
subject: "Independent verification and living context: dcp decisions across the suite (DCP FEAT-023 to FEAT-027, IMP-009)"
status: done
reply_to: 
deferred: 
---

# Independent verification and living context: decisions of the Dev Cycle Protocol

Date: 2026-11-01. From: dev-cycle-protocol. To: aurora-dev-suite.
Backlog record: Levix-Digital/ai-code-agent-skills pull request #79 (docs only).

## Why you are receiving this

Aurora Dev Suite brings dcp, Spec Kit, Gedo and AnyContext together, so you should know about a decision that touches three of the four engines. This message explains what was decided, why, where each piece lives and what may reach Aurora later. **No action is needed now.** We would like your objections, if any, and to hear whether Aurora wants any part of it discussed at suite level.

## Background: the Elephant-Goldfish Model

Google published the Elephant-Goldfish Model (EGM, github.com/google/elephant-goldfish-model, Apache 2.0, version 0.1.0, "not an officially supported Google product"). Despite the name it is not a language model: it is a plugin of skills and sub-agent prompts for spec-driven design (Claude Code, Codex, Antigravity). It covers design only and stops at "design approved, now implement".

Its concepts:

1. **Context asymmetry.** An "Elephant" with full context (project, requirements, the conversation) writes the design. "Goldfish" reviewers start with no history at all, only the document and the files it references. Without a fresh context the review is refused: a review in the writer's context is not a Goldfish review.
2. **The document as the interface between contexts.** If a reader with no context cannot understand the specification, the specification failed.
3. **Standards with a pass rule.** Each has one test: "a competent engineer, or a fresh session, unfamiliar with the code, can...". Zero-Decision (the plan leaves no design choice to whoever implements it), Alternatives (rejected options with reasons, or it fails), Zoom Out (root cause, simplicity, reuse, right altitude), Re-implementation (a document lets a reader rewrite functionally identical code).
4. **Staged review with lenses.** Zoom out, comprehension, design critique, zoom out again over what was added, implementation readiness. In brainstorming, six lenses in parallel try to shoot the idea down.
5. **Fan-out and fan-in.** A synthesizer runs the reviewers (optionally twice for recall), deduplicates and grades the findings. A reviewer that failed is "not run", never "zero findings".
6. **Living specifications and drift.** Approved designs live in the repository. A weekly drift report, run by a model, compares documents with the code. A bottom-up bootstrap writes per-directory indexes (interface, invariants, side effects).
7. **Human ownership, fluid process.** No mandatory gates; the user picks which findings to address.

## The central conclusion

Every engine of the suite exists to give the agent **more** context: AnyContext retrieves documents, plan and session memory keep decisions, and DCP FEAT-009 plans an incremental cycle context handed out at every step. Nothing in the suite deliberately withholds context. EGM's point is that the agent that interviewed, decided and wrote carries the bias of the conversation and validates its own mistakes. The better the suite gets at giving context, the stronger that bias becomes in the writer, so the need for reviewers without context grows instead of disappearing.

Second conclusion: dcp verifies the process deterministically (artifacts exist, gates passed) but does not judge whether the content is good. EGM judges the content but guarantees nothing. The two complement each other: a dcp gate can require that the independent review ran and that no blocking finding is open.

## Decisions

1. **Take the concepts, do not adopt the tool.** EGM's specification format (concepts brief, design document) competes with Spec Kit's spec, plan and tasks, which would give each feature two specifications. It is also an early, unsupported release, and dcp is a proprietary product, so its core should not depend on third-party prompts.
2. **Six items, at the end of the current roadmap**, implemented only after the dcp Rust migration (v2.10.0):

| Item | Release | What it is |
| --- | --- | --- |
| DCP FEAT-023 Context mode in the phase contracts | v3.3.0 | Each phase contract (DCP FEAT-009) declares full or isolated context. The engine assembles an isolated step's package: only the artifact and the files it references, never the conversation. A step that cannot start a fresh context stops; it never falls back to the writer's context. Applies to the spec review, to the Phase 7 code review and to the release candidate attacker (DCP FEAT-020). |
| DCP FEAT-024 Independent review of the specification and plan before gate 3 | v3.3.0 | Isolated reviewers in five stages over the Spec Kit artifacts (no second format). Report in the spec folder with each reviewer's status and findings as blocking, suggestion or nit. Gate 3 fails while the report is missing or stale, a reviewer did not run, or a blocking finding is open, unless the user accepts the risk with a reason. |
| DCP FEAT-025 Testable pass rules | v3.3.0 | Zero-Decision (for the current milestone's tasks, to avoid over-specification), Alternatives (tied to ADRs), Zoom Out and Self-containment, applied by the constitution, the Spec Kit preset and the reviewers. |
| DCP IMP-009 Review synthesis rules | v3.3.0 | A reviewer that did not run is never zero findings and fails the gate it feeds; findings normalized; optional duplicate reviewers for recall. |
| DCP FEAT-026 Documentation drift against the code by Gedo entity identity | v3.4.0 | Documents reference code entities. At plan approval the engine records Gedo's structural hash of each one. Drift = a referenced entity changed after approval while the document did not, disappeared, or never existed. Deterministic and continuous (diagnosis, Phase 6, gate 8); a model only judges whether changed entities still match the text. Only the suite can do this, because only the suite has Gedo. |
| DCP FEAT-027 Curated project context map indexed by AnyContext | v3.4.0 | A reviewed, versioned map of the project (per-module purpose, interface, invariants, side effects) with the Re-implementation rule as its quality bar. AnyContext indexes it with priority and the cycle's full-context steps query AnyContext through its command line contract. |

New epics on our side: EPIC-010 Independent verification, EPIC-011 AnyContext integration.

## Where each piece lives, and why not in Aurora

All six are owned by dcp. Aurora's own decisions put the behavior there: Aurora reaches the engines only through their command lines with versioned JSON (Aurora ADR-0003, ADR-0006), keeps its state derived from the repositories (ADR-0004) and never writes to them. Gates, phase contracts and the review steps run inside the cycle, wherever an agent or a person runs it, with or without Aurora. So the decision and the implementation belong to dcp, and Aurora only shows the results.

Engines that dcp will need, by request to their projects closer to implementation:

- **Gedo** (for DCP FEAT-026): a daemonless command that returns the structural hashes of named entities at a given revision, with --json.
- **AnyContext** (for DCP FEAT-027): a versioned command line contract to index a project's map and query it with --json.

## What may reach Aurora later

Nothing changes in the current contracts. When these items are built, dcp expects to add optional fields in MINOR versions of dcp.diagnose (independent review status of the active plan, drift findings) and to list the new capabilities in dcp version --json. We will propose those fields to you before writing them, as with the earlier contracts. If Aurora would rather see them in a separate command, tell us when the time comes.

## What closes this message

A reply from Aurora with objections or with "no objection". If Aurora thinks part of this should be decided at suite level, say which part and we will bring it to the user.

> seen (2026-11-01 20:42 UTC): deferred: Asks for Aurora's position on a suite-level design (DCP FEAT-023 to FEAT-027, IMP-009); waiting for the user's decision
