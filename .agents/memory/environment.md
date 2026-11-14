---
name: environment
description: Machines, cloud and CI: versions, blocked hosts, setup quirks. Read when a command, install or network call fails or behaves differently between machines.
updated: 2026-11-14
---

- 2026-10-30: Fresh Linux container for the desktop crate: run `apt-get update` first, then install `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev libssl-dev pkg-config`.
- 2026-10-30: Install Trunk with `cargo install trunk --version 0.22.1 --locked` (downloading its release tarball may be blocked by a sandbox).
- 2026-10-30: Do not run `pkill -f "trunk serve"` in the same shell command that started it: it kills the calling shell. Stop the preview by its process id.
- 2026-11-06: Cloud sessions install `dcp-integrations` from the public `Levix-Digital/aurora-releases` repository and resolve the latest tag with `git ls-remote --tags` (the GitHub API refuses repositories not attached to the session; anonymous release downloads work).
- 2026-10-30: Cloud sessions cannot push tags or create or edit GitHub releases (refused by the Git proxy and the API); release candidates come from the candidate workflow, finals from the promote workflow run by the owner.
- 2026-11-14: A cloud session's Git proxy refuses repositories not attached to the session: a renamed counterpart repository (for example `Levix-Digital/dev-cycle-protocol`) must be attached before `dcp-integrations` can write to its inbox.
- 2026-10-29: Hidden refs such as `refs/aurora/*` are refused by the cloud Git proxy; messages use the `aurora/inbox` branch.
