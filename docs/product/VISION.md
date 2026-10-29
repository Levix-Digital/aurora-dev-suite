# Aurora Dev Suite

Aurora Dev Suite is the Levix Digital desktop application, written in Rust, that brings the Dev Cycle Protocol (`dcp`), Gedo (`gedo`) and AnyContext (`actx`) together into one AI development platform for developers who build software with coding agents. It installs, licenses and updates the three engines as one suite, and shows in one window where every project stands, what the agents are changing and what the project's documents say.

## Problem

Levix builds software with several coding agents working in parallel, driven by the Dev Cycle Protocol. The tools that make this work are separate engines, each with its own command line, installer and update path:

- **The state of a project is scattered.** The phase, the plans and their milestones, the backlog, the roadmap, the release candidates, the pull requests and the pending approval gates live in Markdown files, `.dev-cycle.json`, GitHub and the output of a dozen commands. Knowing "where does project X stand and what is waiting for me?" means opening files and running commands in each repository.
- **Parallel agents are invisible.** When several agents work on one repository, nobody sees which entities each one is changing or where they are about to collide until a merge fails. Gedo will find these collisions (`gedo radar`, agent streams), but only in a terminal, one repository at a time.
- **Knowledge is a separate tool.** AnyContext answers questions about code, documents and documentation sites, but it lives in its own terminal interface, away from the plans and the code it should inform.
- **Three products, three installs.** Each engine is installed, updated and (soon) licensed on its own, on Windows and Linux, and the Dev Cycle Protocol still needs Python and its packages on every machine and cloud session.

Today the team copes with terminal commands, reading the Markdown files directly and keeping the picture in their heads.

## Target Users

- **Levix Digital development team (primary).** Levi and the coding agents working for him on several products in parallel (Gedo, AnyContext, the Dev Cycle Protocol, the platform). They want one place that shows every project's state and what needs their decision, sees the agents' work, and answers questions from the projects' knowledge, without leaving the desktop or managing three tools by hand. Aurora is built and dogfooded by this team first.
- **Individual developers using coding agents (later).** Developers who run coding agents and want structure (phases, gates, reviews), safe parallel work and grounded knowledge. They try the suite through a free tier.
- **Teams and companies (later).** Engineering teams that want the same on shared repositories, with team licenses.

## Value Proposition

- **One window for the whole cycle:** every project's phase, plans, gates, release candidates and pull requests, with what is waiting for the user's decision at the top.
- **Parallel agents made visible:** which agent works on what, and the collisions Gedo finds, before they become broken merges.
- **Knowledge next to the work:** AnyContext questions over the project's code, plans, ADRs and documents from inside the same window.
- **One install, one license, one update** for the three engines, native on Windows and Linux, with no Python, Node or other runtime.
- **Local-first:** the code and documents never leave the user's machine and Git repositories; uninstalling leaves every repository a plain Git repository.

## Business Model

- **Proprietary, binary-only distribution.** Aurora, Gedo, `dcp` and AnyContext are closed source: private source repositories publish signed binaries and installers to public releases repositories (the pattern AnyContext already uses with `any-context-releases`).
- **Freemium.** Each engine keeps a free tier and paid plans of its own; Aurora is the bundle that joins the three in one suite plan (individual and team), priced below the sum of the separate plans.
- **Licenses** are signed by Levix and verified offline, through the licensing component of `levix-platform`.
- Exact plans, prices and what each tier includes are open (see Open Questions).

## Success Metrics

- A new machine (Windows or Linux) goes from nothing to Aurora with the three engines installed and licensed in under 5 minutes.
- The Levix team runs its daily development of every active product from Aurora: the state of each project and its pending gates are found in under 10 seconds, without opening a terminal.
- Every collision Gedo reports between parallel agents is visible in Aurora within 30 seconds of being detected.
- Aurora works with any engine version that reports the capabilities it needs, and degrades with a clear message (never a crash) when a capability is missing.
- No runtime other than the installed binaries is needed on the user's machine.

## Scope

The complete product, as capability areas:

1. **Suite installer and updates:** one installer for Aurora and the three engines, with verified downloads, release channels and atomic self-update on Windows and Linux.
2. **Licensing and plans:** suite and per-engine licenses, verified offline, with a free tier.
3. **Engine health:** the version and capabilities of each installed engine, and what each Aurora feature needs.
4. **Project portfolio:** the user's repositories and, for each, the cycle state from `dcp`: phase, plans and milestones, backlog, roadmap, release candidates, pull requests, gates and blocking bugs.
5. **Decision inbox:** everything waiting for the user across projects (gates to approve, release candidates to test, review threads, untriaged bugs), first as a view and later with actions.
6. **Agent work view:** the agents running on each repository, their Gedo streams and scopes, and the collisions and contract breaks Gedo reports.
7. **Knowledge:** AnyContext questions and answers with citations over each project's sources, from inside Aurora.
8. **Engine integration contract:** Aurora reads the engines only through their command lines with versioned `--json` output and capability discovery, each engine behind a port, with fixtures until its Rust binary offers the contract (ADR-0006).

The foundations are built first, while the engines mature: the desktop shell, the engine ports, engine health, and every view built and tested against fixtures that follow the engines' contracts. Each engine's real adapter, and the features that depend on it, arrive with the engine's Rust binary (ADR-0006).

## Non-Goals

- Re-implementing what an engine does: Aurora shows and drives the engines, it never duplicates their logic.
- Linking the engines' code: integration is through their command lines and `--json` contracts only.
- A terminal interface: Aurora is a desktop application from the start.
- An editor or IDE, or a Git hosting service.
- A cloud service in the first versions: Aurora runs on the user's machine.
- Corporate governance of installations (users, devices, policies): that is the Levix Admin Hub.
- Making the engines depend on Aurora: each engine stays usable and sold on its own.

## Risks and Assumptions

- **The engines are not ready yet.** Gedo is a library until its v0.4.0 brings the command line; `dcp` is still Python scripts until its migration to Rust; AnyContext has no `--json` output. Reduced by agreeing on the contracts now (suggestions to each repository) and by building Aurora's foundations and views first against fixtures that follow those contracts; each engine is plugged in only through its Rust binary when it ships the contract (ADR-0006), never through Python or its files.
- **The contracts may drift.** Reduced by versioned `--json` schemas and capability discovery in every engine, following Gedo's ADR-0013.
- **The license change.** Gedo and the Dev Cycle Protocol documents still describe an open-core model with MIT or Apache licenses; the move to proprietary must be reflected in each repository before anything is published.
- **Desktop UI in Rust.** Leptos has fewer ready components than web frameworks in other languages, and the web view differs between Windows (WebView2) and Linux (WebKitGTK); reduced by a first release that exercises the shell, the theme and CI on both systems (ADR-0002).
- **One small team.** Aurora competes for time with the engines themselves; reduced by keeping the foundations small (the shell, the engine ports and the first views) and plugging each engine in as it becomes ready.

## Open Questions

- Plans, prices and tier contents of the suite and of each engine.
- The final product name: "Aurora Dev Suite" replaces "Mission Control" (Gedo's planned team dashboard); is "Levix DevOS" retired?
