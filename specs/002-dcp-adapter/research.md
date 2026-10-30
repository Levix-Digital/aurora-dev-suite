# Research: dcp adapter

## R1. Where the contract comes from

- **Decision**: Aurora writes the contract (`contracts/dcp-diagnose.schema.json`) and proposes it to the Dev Cycle Protocol project through the cross-project inbox, asking for `dcp diagnose --json` with schema `dcp.diagnose` 1.x and the `diagnose` capability in `dcp version --json`.
- **Rationale**: ADR-0006 builds features against fixtures that follow the agreed contract, and the suggestion file of 2026-10-27 for ai-code-agent-skills already lists `dcp diagnose --json` as the base of the portfolio. As of 2026-10-29 the protocol's backlog has no item for it (DCP FEAT-017 covers only `config` and `metrics`, DCP FEAT-018 the extension point), and its Rust engine has neither `diagnose` nor `version --json`. Writing the schema first gives both sides one source of truth; the capability catalog of v0.1.0 already names `diagnose` for the Portfolio, Project and Inbox features.
- **Alternatives considered**: waiting for the protocol to define it (blocks all of v0.2.0); several commands (`plan list`, `roadmap show`, `pr status`...) composed by Aurora (one process per command per project, and Aurora would rebuild the protocol's rules about what is pending).

## R2. Shape of the reply

- **Decision**: one reply per project carrying everything the three views need: `root`, `summary`, `next` (ask, recommended, options), `release` with open items, `plans`, `release_candidate`, `pull_requests`, `bugs` (blocking, untriaged), `findings`. Items are objects (`id`, `rank`, `title`), not preformatted strings.
- **Rationale**: it is the information the protocol's diagnosis already collects (`onboard.py diagnose --json`, `state`), so the engine can produce it without new analysis; typed fields let Aurora sort and count without parsing text. Setup-only parts of today's diagnosis (configuration problems, ignored files, Spec Kit and agent settings) stay out of version 1: they are for whoever sets up a repository, not for the portfolio, and can come as optional fields in a MINOR.
- **Alternatives considered**: copying today's `state` object as it is (strings such as `"FEAT-016 (high) dcp adapter"`, mixed setup data; a contract would freeze those accidents).

## R3. Checking the reply belongs to the project

- **Decision**: the reply carries `root`; Aurora compares it with the project's root using the same canonical comparison as the registry (case-insensitive on Windows) and refuses a mismatch as invalid.
- **Rationale**: `--dir` given a sub-folder or a linked path could make the engine answer for another repository; a silent mix-up would show one project's state under another's name.

## R4. Where the gate on capability and presence lives

- **Decision**: in the core use case `cycle_state`, before the port is called: it takes the latest `dcp` report and the project, and returns `Unavailable` or `ProjectMissing` without running anything.
- **Rationale**: the rule is product logic shared by the desktop and the preview (hexagonal style); the adapter stays a thin translation of process output.

## R5. Location of the binary

- **Decision**: the adapter runs the `location` of the latest `dcp` engine report, with the `dcp` time limit from the settings.
- **Rationale**: the probe already resolved the configured path or the search path; using its result keeps one lookup rule and makes a replaced binary visible after the next refresh (spec edge case).

## R6. Sample engine

- **Decision**: the fake engine (`aurora-fake-engine`, feature `fake-engine`) learns `diagnose --json --dir <root>` with the modes it already has (`ready`, `hang`, `crash`, `huge`, `invalid`, `unsupported`) and replies with a sample that echoes the given root, so the adapter's integration tests cover every failure.
- **Rationale**: same pattern as the probe tests of v0.1.0; no new test tool.
