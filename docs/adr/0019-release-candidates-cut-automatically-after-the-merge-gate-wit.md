---
id: 19
title: Release candidates are cut automatically after the merge gate, with notes from the repository
status: accepted
date: '2026-11-14'
version: null
subsystems: []
supersedes: []
superseded_by: []
amends: [8]
amended_by: []
---

# ADR-0019: Release candidates are cut automatically after the merge gate, with notes from the repository

## Context

ADR-0008 publishes Aurora's packages from a release candidate tag pushed by hand on `main`, with notes generated from the pull requests. On 2026-11-13 the owner decided that release candidates are published automatically once the human gate (the review and merge of the pull request) has passed, as DocMapper does (DocMapper ADR-0015, pull requests #16 and #17), and DocMapper proposed the same to every project (message 20261113-215443). The Levix Digital website reads each product's public releases repository and emails a version's first release candidate, quoting its release body, to the people who signed up to test it (messages 20261113-153953 and 20261113-155353 from levix-website).

## Decision

- **A candidate workflow** runs after every successful CI run of a push to `main`. It tags the head `vX.Y.Z-rc.N`, with X.Y.Z the workspace version and N the next candidate number of that version, and calls the release workflow for that tag. It cuts nothing, and says why in its run summary, when `vX.Y.Z` is already released, when the commit already has a candidate, when `main` has moved on, or when the candidate notes are missing.
- **Release notes live in the repository**, in `docs/releases` (`vX.Y.Z.md`, `vX.Y.Z-rc.md` or `vX.Y.Z-rc.N.md`), and become the body of the release. The candidate notes file is the gate that a version is ready for testers. A tag without notes is not published.
- **Human gates are unchanged:** the merge for a candidate, the `release` environment of the promote workflow for a final release. Candidates stay pre-releases never marked latest; final releases are marked latest. A candidate tag pushed by hand still works.
- **Releases stay in the public releases repository** named by the `RELEASES_REPOSITORY` variable (ADR-0005).

## Alternatives Considered

- **Keep tagging candidates by hand:** one more manual step per candidate, and generated notes that do not tell testers what to try.
- **A candidate on every green merge, without a notes gate:** merges of work in progress would email testers.

## Consequences

- Every version needs its notes file before its first candidate; bumping the version and adding the notes is the release preparation.
- The website can announce candidates and finals from the public releases repository with no token in this project.
- The same flow carries over to the server packaging of ADR-0014 (FEAT-046), which changes the assets, not the gates.
