---
id: 5
title: Proprietary binary-only distribution with Levix licenses
status: proposed
date: '2026-10-27'
version: null
subsystems: [desktop]
supersedes: []
superseded_by: []
amends: []
amended_by: []
---

# ADR-0005: Proprietary binary-only distribution with Levix licenses

## Context

Levi decided on 2026-10-27 that every Levix product is proprietary and distributed only as binaries, freemium, with Aurora sold as the suite. AnyContext already publishes binaries from a private source repository to a public releases repository. `levix-platform` plans the licensing component (one license per plan, signed by Levix, verified offline) and the verified catalog of downloads.

## Decision

- The source of Aurora stays private; signed binaries and installers are published to a separate public releases repository.
- Every crate has `publish = false` and the proprietary license notice; dependencies stay permissive (MIT, Apache-2.0, BSD, ISC, Zlib, Unicode-3.0), checked with `cargo-deny`.
- Licenses and downloads use the `levix-platform` components by version tag once they exist (its FEAT-019 and FEAT-020). Until then the foundations have no license check, for internal use by the Levix team.

## Alternatives Considered

- **Open core** (the earlier plan of Gedo and the Dev Cycle): wider adoption, but gives the research behind the engines away and makes compliance gates easy to bypass.
- **A license scheme of Aurora's own**: faster to start, but a fourth scheme beside the products' and the Admin Hub's.

## Consequences

- No public crates; the engines are installed as binaries, never through `cargo install`.
- Aurora's licensing waits for `levix-platform`, and a suite plan needs the change proposed to it on 2026-10-27.
