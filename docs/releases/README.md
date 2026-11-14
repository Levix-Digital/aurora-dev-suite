# Release notes

One file per release, named after its tag: `vX.Y.Z.md` for a final release, and for release candidates either `vX.Y.Z-rc.md` (shared by every candidate of the version, updated as work lands) or `vX.Y.Z-rc.N.md` (one candidate only, takes precedence). The release workflow publishes the file as the body of the release in the public releases repository (ADR-0005, ADR-0019) and refuses to publish a release without it. It reads the file from `main`, so notes can be fixed after the tag is pushed and before the workflow runs again.

The Levix Digital website emails a version's first release candidate to the people who signed up to test it and quotes the first 2000 characters of its notes, so put what testers need first. Plain Markdown, in English:

```markdown
## What changed
- ...

## What to try
- ...

## Known issues
- ...
```

Final release notes summarize the changes since the previous final release for people who download the product; they are not emailed.

## Releasing

1. Keep the version of the next release in `Cargo.toml` (`[workspace.package]`) and its candidate notes in `docs/releases/vX.Y.Z-rc.md` on `main`. Adding the notes file is the signal that the version is ready for testers.
2. After every green CI run on `main`, the candidate workflow (`.github/workflows/candidate.yml`) tags the head `vX.Y.Z-rc.N` (next N) and publishes the pre-release, never marked latest. It skips, with a note in its run summary, when `vX.Y.Z` is already released, when the commit already has a candidate, when `main` has moved on, or when there are no candidate notes, so merges of work in progress cut no candidate. A candidate tag pushed by hand still works.
3. When a candidate is accepted, add `docs/releases/vX.Y.Z.md` on `main` and run the promote workflow with the candidate tag (`gh workflow run promote.yml -f candidate=vX.Y.Z-rc.N`): after the approval of the `release` environment, it tags the candidate's commit `vX.Y.Z` and publishes it as the latest release.
4. Bump the version in `Cargo.toml` for the next release; until then no new candidate is cut.
