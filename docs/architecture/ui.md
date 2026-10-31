---
title: User Interface
summary: 'The Leptos interface compiled to WebAssembly: views, navigation, the backends and the Aurora Borealis theme.'
code: [crates/aurora-ui]
---

# User Interface

The Leptos interface compiled to WebAssembly: views, navigation, the backends and the Aurora Borealis theme.

## Responsibilities

- Render the views and send the user's actions to a backend.
- The theme: Aurora Borealis tokens as CSS custom properties, light and dark.
- It holds no rules: availability, statuses and validation come from the core.

## Structure

| Path | Holds |
|---|---|
| `src/app.rs` | the shell: `View` signal, navigation, Alt shortcuts, theme, shared `AppContext`, preview banner |
| `src/backend/` | `Backend`: `Tauri` (`tauri.rs`) or `Fixture` (`fixture.rs`) |
| `src/views/` | `portfolio`, `project`, `engines`, `inbox`, `settings`, `loader` (scheduling of parallel reads), `reads` (row state and reads over the loader, shared by Portfolio and Inbox), and shared helpers |
| `src/theme_contrast.rs` | the contrast test of the tokens |
| `styles/tokens.css`, `styles/app.css` | the theme and the layout |
| `index.html`, `Trunk.toml` | the Trunk build; `trunk serve` listens on port 1420 |

## Behavior

**Backends.** At start-up the interface uses Tauri when `window.__TAURI__` exists, otherwise the fixture backend. The fixture backend answers every command of the desktop from the core's fixtures, and simulates the engine events after a short delay. Each refresh alternates Gedo between ready and degraded, so the preview shows every engine state. A banner says the preview shows sample data. A test checks that the fixture backend answers every command listed in the desktop contracts. For `project_cycle_state` the fixture backend takes `dcp` as ready with the contract, so the views can be built before `dcp` ships it, while the Engines view keeps showing the sample engines; the Portfolio calls it for every project.

**Navigation.** One `View` signal: Portfolio, Project, Engines, Inbox, Settings. The navigation is a list of buttons; the arrow keys move between them, and Alt+1 to Alt+5 open the views. Every focused element has a visible outline. There is no skip link: the sidebar and the content are `<aside>` and `<main>` landmarks, which screen readers jump between, and Alt+1 to Alt+5 also move the focus to the content.

**Views.** Portfolio lists the projects with add (folder dialog or typed path), rename, remove with confirmation, and locate for missing ones, plus the registry notices. Each row shows its project's cycle summary from the core (phase, current release, what waits for the user by kind), or reading, or the failure reason with the engine's error output; the header totals what waits, and Refresh reads every project again keeping the last state until the new one arrives. The reads go through `views::reads` over `views::loader::Loader`: at most four at once, never two for the same project, each answer applied as it arrives, and answers older than the latest request or for a project no longer listed dropped. The unavailable hint is shown once, by the Portfolio's availability line. Engines shows each engine's status, version, location, capabilities, the features it enables, and its last error output, with a Refresh button. Settings holds the theme and each engine's location and time limit. Each Portfolio row has **Open**, which sets the chosen project in the shared `AppContext` and shows the Project view; the choice lasts while Aurora runs and is not saved. The Project view reads the chosen project (the first registered one when none is chosen) through `project_cycle_state`, keeps only the answer of the latest request, and shows one card per section: overview, gates, next step, what waits, plans and milestones, roadmap, release candidate, pull requests, blocking and untriaged bugs; a section with nothing says so. It offers a project choice and Refresh, which keeps the last state until the new one arrives. Failures use `views::CycleFailure`, the same text and engine output disclosure as the Portfolio rows. The Inbox reads every registered project the same way as the Portfolio (its own reads, when it opens) and shows the core's `Inbox`: a header with the items, the projects they belong to, the projects still being read and a note when a project has findings, then one card per urgency with each item's project, kind, title, detail and **Open**, which opens the Project view on its project. Projects that failed are listed under "Not read" through `views::CycleFailure`, or in one line when every failure is the unavailable `dcp`. The Project view has two tabs kept in `AppContext`, **Cycle** (the sections above) and **Settings** (`views::project_settings`): the settings of the chosen project through `project_config`, grouped as the reply groups them, each with a control for its type (choice, check box, text, number as text, check boxes for a list with allowed values, one value per line for a free list), its default when it differs, read-only settings disabled with their reason, and the edits marked. A toolbar counts the edits; **Preview changes** sends the core's `ConfigDraft::change` through `preview_project_config` and shows each key's before and after with the diff; **Apply** sends exactly that change through `apply_project_config`, then reads the settings again and reminds the user to commit `.dev-cycle.json`. Refusals are shown under their field with the accepted values; a stale revision offers to read again, keeping the edits (`ConfigDraft::rebase`). Leaving the tab or choosing another project with edits not applied asks first. In the preview, the fixture backend keeps the sample settings in memory. Every section shows ready, degraded, unavailable with what is needed, or the release that brings it. Results and errors go to a polite live region.

**Theme.** `<html data-theme>` is `light` or `dark`, resolved from the setting and from `prefers-color-scheme` while the setting is "system". The dark tokens come from AnyContext's `AuroraTheme`, with muted text lightened to `#94A3B8` and errors to `#F87171` for contrast; the light tokens are derived. The test checks every text and background pair at 4.5:1 and focus indicators at 3:1, in both themes. Views use classes only, no inline styles, to fit the content security policy.

## Interfaces

Calls `aurora-desktop` commands and listens to its events; uses the types of `aurora-core`.

## Decisions

- [ADR-0002](../adr/0002-desktop-application-with-tauri-2-and-a-leptos-interface.md): Leptos interface, all Rust.
