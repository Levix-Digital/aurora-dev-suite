---
name: workflow
description: How work flows in this repository and the tool pitfalls already hit. Read before branching, committing, opening a pull request or releasing.
updated: 2026-11-14
---

- 2026-10-30: Flow per backlog item: the plan's pull request (feature), then after its merge a small `levix/close-<plan>-plan` pull request that completes the plan and closes the backlog item.
- 2026-10-30: Plans, ADRs and backlog entries name the owner as author (`by`/`author`), never a tool; the plan-memory scripts write the frontmatter and then fail on the missing `chromadb` index, so check the file and fix the author by hand.
- 2026-10-31: When asking the owner to push a tag by hand, give commands that tag `origin/main` after `git fetch origin`, never a bare `git tag` on a possibly stale local `main`; check `git ls-remote --tags origin` afterwards.
- 2026-11-06: Review comments posted from a cloud session get a tool footer that only the owner can remove: list every such review body or comment for the owner, with links.
- 2026-11-10: `dcp-integrations check` must run at the start of work: `poll` counts only open messages, so a deferred message is silent at `poll`.
- 2026-11-14: Run the architecture-docs checks (`check_docs`) and rebuild the TECDOC index after adding an ADR; concurrent pull requests that each add an ADR conflict in `TECDOC.md`: rebuild the index after merging `main`.
