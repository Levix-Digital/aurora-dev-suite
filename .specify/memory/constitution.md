<!-- dev-cycle-constitution
generated-by: dev-cycle-protocol/constitution.py (do not edit; change a source and regenerate)
sources-sha256: 8942da4035d770cf0fbd95aae2031e3153d299ca441e22491a840a17656ca5ee
content-sha256: 4ef7386e57038cf858d67abe054b5c46392e847c52e6d641849d9bfe1fdaf37c
-->
# aurora-dev-suite Constitution

The non-negotiable rules every specification, plan and task of this project follows. It is generated from the project's sources of truth by `dev-cycle-protocol/constitution.py`; change a source and regenerate it, never edit it.

## Core Principles

### I. Design

*Source: software-design*

#### Modularity and separation of responsibilities
- One module, type or function has **one reason to change**. When a description needs "and", split it.
- Group code by what it is about (a domain concept or a feature), not by what kind of file it is, unless the architecture style says otherwise.
- Business rules never live in the user interface, in transport code (HTTP handlers, CLI argument parsing) or in persistence code.
- Dependencies point toward stable code: details (frameworks, databases, network, UI) depend on the core, never the other way around.

#### SOLID, in practice
| Principle | What it means when writing code | Smell that it is violated |
|---|---|---|
| **S**ingle responsibility | Each unit does one job and can be named without "and". | A 600-line file mixing parsing, rules and output. |
| **O**pen/closed | Add behavior by adding a new implementation, not by growing an `if/else` chain in stable code. | Every new provider edits the same `match` in five places. |
| **L**iskov substitution | Any implementation of an interface can replace another without surprises. | An implementation that panics or ignores half the contract. |
| **I**nterface segregation | Small interfaces shaped by what the caller needs. | Callers implementing methods they never use. |
| **D**ependency inversion | High-level code depends on abstractions it owns; adapters implement them. | Core code constructing an HTTP client or a database driver. |

#### Clean Code
- **Names** say what something is or does; no abbreviations a newcomer cannot decode; no `data`, `info`, `manager`, `util` as whole names.
- **Small functions** that do one thing at one level of abstraction; early returns instead of deep nesting.
- **No magic values**: named constants with units in the name (`TIMEOUT_MS`).
- **Comments explain why**, never what; delete commented-out code and dead code.
- **Consistency over preference**: follow the conventions already in the codebase (naming, error types, layout) unless the change is an agreed refactor.
- **Duplication**: remove it when the same rule exists twice; tolerate it when two pieces only look alike today.

#### Explicit error handling
- Never swallow an error (`except: pass`, `let _ =` on a meaningful result, an empty `catch`, a blind `.unwrap()` on input that can fail).
- Errors carry context (what was being done, with which input) and use the project's canonical error types.
- Decide at each boundary: recover, translate into the caller's error type, or propagate. Report to the user once, at the edge.
- Fallbacks are allowed only when they are explicit, logged and visible in status output; a silent fallback that changes behavior is a defect.

#### Testability
- Pure logic is separated from I/O so it can be unit-tested without mocks.
- External services sit behind an interface the tests can replace.
- A structure that is hard to test is a design problem, not a testing problem.

### II. Simplicity First

*Source: software-design*

Principles applied mechanically produce over-engineering. The rule: **the simplest solution that keeps responsibilities separate.**

- Introduce an abstraction (trait, interface, base class, factory) only when at least one of these is true:
  1. there is a **second implementation** now (not "maybe later");
  2. it creates a **test seam** you need today;
  3. it isolates an **external dependency** (framework, service, database, OS API).
- No speculative configuration, plugin systems or generic frameworks for a single use case.
- Prefer composition over inheritance; prefer plain data and functions over class hierarchies.
- Three similar lines are better than a premature helper with five parameters.

#### Design patterns: only when they solve a real problem
| Problem actually present | Pattern that fits |
|---|---|
| Several interchangeable algorithms or providers chosen at runtime | Strategy |
| A third-party or legacy interface that does not match yours | Adapter |
| Construction is complex or depends on configuration | Factory / Builder |
| Behavior changes with an explicit lifecycle state | State machine |
| Actions must be queued, logged, undone or dispatched uniformly | Command |
| Persistence must be swappable or mocked | Repository |
| Several parts react to an event without knowing each other | Observer / event channel |

If you cannot point to the problem in the left column, do not use the pattern. Name the pattern in code or docs only when it helps the reader.

### III. Architecture Style: hexagonal

*Source: software-design/styles/hexagonal.md*

##### Structure

- **Core (domain + application services)**: entities, business rules, use cases and the **ports** they need (interfaces/traits for storage, providers, clocks, file systems). No framework, UI, transport or driver code.
- **Driving adapters** (input): CLI, TUI, HTTP handlers, RPC, MCP servers. They translate requests into core calls and core results into output.
- **Driven adapters** (output): database repositories, HTTP clients, model providers, file system access. They implement the core's ports.
- **Composition root**: one place per executable that builds the adapters and wires them into the core.

##### Dependency rule

Adapters depend on the core; the core depends on nothing outside itself except plain libraries (serialization, math, date handling). List the adapter and framework modules in `architecture.forbidden_core_imports` so `check_design.py` enforces the rule.

##### Where things go

| Piece | Location |
|---|---|
| A business rule or validation | Core service or entity |
| A new user command | Core command/use case, then one thin handler per surface |
| Database schema or query | Driven adapter implementing a core repository port |
| A new LLM, payment or storage provider | Driven adapter implementing an existing port |
| Output formatting, colors, layout | Driving adapter |

##### Common mistakes

- Business logic in a CLI handler or TUI event loop "because it was quicker".
- The core constructing its own HTTP client, database connection or UI widget.
- Ports shaped like the first adapter's API (leaking driver types into the core).
- A port for something with one implementation and no test need (see "Simplicity First").

### IV. Development Cycle

*Source: dev-cycle-protocol*

1. **Design**: modular code with clear responsibilities, SOLID, Clean Code, explicit errors, patterns only when needed, in the project's architecture style (`software-design`).
2. **Evidence**: every change is proven by automated tests; manual tests cover only what cannot be automated, in one curated suite (`quality-assurance`).
3. **Documentation describes today**: technical docs and README are edited in place; history lives in Git and release notes (`architecture-docs`, `user-docs`).
4. **Plans are shared and complete**: approved plans are versioned files in the project (`plan-memory`).
5. **Safe delivery**: short-lived branches on the project's branching model, CI on every pull request, release candidates, the user's release gate (`devops-release`).
6. **Security**: no secrets in Git, backward-compatible data migrations, a rollback path for every release.
7. **Bugs first**: open `critical`, `high` and `medium` bugs are planned and fixed before new features, improvements and adjustments; a `low` bug goes into the next release at the latest; no known bug stays untriaged (a severity, and a release for the ones that do not block). Only the user can explicitly accept the risk. Nothing found is lost, it is fixed now or captured in the backlog.
8. **Follow-through**: the agent follows CI and releases to the end and fixes what fails; the user's decisions (plan approval, candidate acceptance, the release gate) are never taken by the agent.
9. **No AI attribution, anywhere**: no artifact names an AI, a model or a coding agent as author, co-author, reviewer or source: commits and their trailers, tags, branches, pull and merge requests, reviews and review comments, issues, plans, documentation, release notes and code comments. Everything is authored by the repository's Git identity (`devops-release`). Configuration a project needs for its tools (an agent's settings folder, an integration name) is technical, not attribution.

## Product Requirements and Constraints

*Source: docs/product/TECH_SPEC.md*

### Non-Functional Requirements

#### Performance

- Cold start to the first painted window: under 1.5 s on a mid-range laptop (4 cores, SSD).
- Portfolio with 20 registered projects: every project's summary shown within 3 s of start, each project appearing as soon as it is read.
- One engine call: timeout 10 s by default (configurable per command); a timeout marks only that section as degraded.
- Installed size of Aurora itself (without the engines): under 30 MB.

#### Security and Privacy

- No network access of Aurora's own in the foundations; the engines decide their own network use.
- Code, documents and engine output never leave the machine; no telemetry.
- Engine commands run without a shell, with fixed arguments and the project directory as working directory; paths from the registry are validated (existing directory, inside a Git repository).
- The web view gets no file system, shell or HTTP access: Tauri capabilities allow only Aurora's own commands, with a strict content security policy.
- Engine output is untrusted input: size-limited, parsed with typed structs, rendered as text (Markdown rendered without raw HTML).

#### Availability and Recovery

A desktop application without a service. Recovery means: a missing or broken engine never blocks the rest of the application, and a corrupt registry file is moved aside and recreated empty with a message.

#### Accessibility and Localization

- Keyboard navigation for every view, visible focus, contrast of WCAG 2.2 AA in light and dark themes.
- Interface in English first; text kept in one catalog so Portuguese (pt-BR) can follow.

### Constraints

- Proprietary, binary-only distribution; every crate `publish = false`; dependencies limited to permissive licenses (MIT, Apache-2.0, BSD, ISC, Zlib, Unicode-3.0), checked with `cargo-deny` (ADR-0005).
- No engine crate is linked (ADR-0003); the engines' schedules decide when each feature can ship.
- One small team (Levi and coding agents): the foundations stay small, and every feature is testable with fixtures before its engine exists.
- Windows and Linux are both first-class: CI builds and tests on both.

## Architecture Decisions

Accepted decisions; a plan that contradicts one needs a new ADR that supersedes it.

- ADR-0001: Hexagonal architecture with the engines behind ports (`docs/adr/0001-hexagonal-architecture-with-the-engines-behind-ports.md`)
- ADR-0002: Desktop application with Tauri 2 and a Leptos interface (`docs/adr/0002-desktop-application-with-tauri-2-and-a-leptos-interface.md`)
- ADR-0003: Engines integrated only through their command lines with versioned JSON (`docs/adr/0003-engines-integrated-only-through-their-command-lines-with-ver.md`)
- ADR-0004: Local-first state derived from the repositories (`docs/adr/0004-local-first-state-derived-from-the-repositories.md`)
- ADR-0005: Proprietary binary-only distribution with Levix licenses (`docs/adr/0005-proprietary-binary-only-distribution-with-levix-licenses.md`)
- ADR-0006: No stand-in adapters: engines plugged in only through their Rust binaries (`docs/adr/0006-no-stand-in-adapters-engines-plugged-in-only-through-their-r.md`)

## Governance

- This constitution supersedes other practices for specifications, plans and tasks. A plan passes the Constitution Check or records each justified violation in its Complexity Tracking table; a lasting exception becomes an ADR.
- Amendments change the source, not this file: an ADR for decisions, the technical specification for requirements and constraints, `.dev-cycle.json` for the architecture style, the skills for principles. Then run `constitution.py generate` and commit the result with the change.
- Gate 3 of the development cycle blocks a plan while this file is stale or edited by hand.

**Version**: 1.2.0 | **Ratified**: 2026-10-27 | **Last Amended**: 2026-10-27
