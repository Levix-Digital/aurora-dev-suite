---
id: 17
title: Each project declares the repositories it writes and reads in .aurora/integrations.json
status: accepted
date: '2026-11-13'
version: null
subsystems: [integrations]
supersedes: []
superseded_by: []
amends: [15]
amended_by: []
---

# ADR-0017: Each project declares the repositories it writes and reads in .aurora/integrations.json

## Context

Which repositories a project may change, and which ones it must read to work, lived only in hand-written instructions per project, and they drifted: Aurora's listed levix-admin-hub as a shared library it writes to, while the Admin Hub is a product with its own backlog, decisions and releases that depends on levix-platform (Admin Hub README, VISION, ADR-0005, ADR-0008). ADR-0015 already gives shared libraries no seat at a round table and asks the user to mark them with a checkbox when a project is added, but nothing in the repositories says which projects are libraries. On 2026-11-12 Levi proposed declaring, per project, the repository it writes to and the repositories it reads, as the source of the context needed for development, with messages remaining the integration proposals themselves.

## Decision

Levi decided on 2026-11-13 that the declaration lives in each project's `.aurora/integrations.json` (schema 1.1, additive):

- **`kind`**: `project` (the default when missing) or `library`. A library declares it in its own file; `dcp-integrations add` copies it into the consumer's counterpart entry.
- **`writes`**: slugs; the first is the project itself, and any other entry must be a counterpart of kind `library`. A project never lists another project: what it needs there is asked for by message.
- **`reads`**: the context sources, each a counterpart slug or a plain `url`, with an optional `ref` (default: that repository's production branch), optional `paths` to read first and a one-line `why`. Everything in `writes` is readable.
- **dcp-integrations** validates the declaration on `init`, `add` and `access --check`, returns it with `access --json`, opens `check` with a "Writes" and a "Reads" line, and refuses `send` to a library ("open a pull request there from this project"). A round-table seat reads its own repository plus its `reads`; a change to a library names as informed every project that reads or writes it.
- **Aurora** shows the declaration (project view, portfolio dependency map, the library flag of the add-project form read from `kind`) and never edits the file.
- `counterparts` stays the list of projects the inbox talks to; `consumes` and `follows` stay where they are until FEAT-030 is planned.
- levix-platform is a library; levix-admin-hub is a project.

## Alternatives Considered

- **Only in Aurora's settings for each registered project**: a session without Aurora would not see it, and dcp-integrations could not validate it.
- **A separate file next to integrations.json**: one more file to keep in step with the counterparts it refers to.
- **Moving `consumes` and `follows` into `reads` now**: one place for the same information, but a MAJOR 2 of a file every project already committed.

## Consequences

- The declaration is not a permission: Git access is still what the credentials allow. A warning in `verify` for a pull request writing outside the declaration can come later.
- Libraries have no inbox branch; messages already sent to one are closed with a note.
- Each project adds its own declaration in its own repository; Aurora proposes it by message once dcp-integrations supports schema 1.1.
