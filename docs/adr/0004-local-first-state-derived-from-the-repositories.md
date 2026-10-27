---
id: 4
title: Local-first state derived from the repositories
status: proposed
date: '2026-10-27'
version: null
subsystems: [core, engines]
supersedes: []
superseded_by: []
amends: []
amended_by: []
---

# ADR-0004: Local-first state derived from the repositories

## Context

The engines already keep their state where it belongs: the Dev Cycle in each repository's files, Gedo under `refs/gedo/*` and `.git/gedo/`, AnyContext in its workspaces. Aurora is local-first and must never become a second source of truth for any of it.

## Decision

- Aurora's only own data is the **project registry** (the repositories the user added, with a display name) and its settings, stored as a TOML file in the user's configuration directory (`aurora/` under the platform's config path).
- Everything else is derived on demand from the engines and held in memory; a refresh re-reads it.
- No database in the foundations. A local cache may be added later, rebuilt from the engines and never edited.

## Alternatives Considered

- **A local database of projects and states** (SQLite or redb): faster start-up and history, but a second copy of the engines' data that can drift.
- **A cloud service**: outside the product's scope; the code and documents never leave the machine.

## Consequences

- Removing Aurora leaves every repository and engine untouched.
- Start-up reads each registered project again; with many projects this is done in parallel and shown progressively.
- History over time (trends, past gates) needs the engines' own history, not Aurora's.
