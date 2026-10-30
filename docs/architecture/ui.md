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
| `src/views/` | `portfolio`, `project`, `engines`, `inbox`, `settings`, and shared helpers |
| `src/theme_contrast.rs` | the contrast test of the tokens |
| `styles/tokens.css`, `styles/app.css` | the theme and the layout |
| `index.html`, `Trunk.toml` | the Trunk build; `trunk serve` listens on port 1420 |

## Behavior

**Backends.** At start-up the interface uses Tauri when `window.__TAURI__` exists, otherwise the fixture backend. The fixture backend answers every command of the desktop from the core's fixtures, and simulates the engine events after a short delay. Each refresh alternates Gedo between ready and degraded, so the preview shows every engine state. A banner says the preview shows sample data. A test checks that the fixture backend answers every command listed in the desktop contracts. For `project_cycle_state` the fixture backend takes `dcp` as ready with the contract, so the views can be built before `dcp` ships it, while the Engines view keeps showing the sample engines; no view calls it yet (FEAT-014, FEAT-015 and FEAT-017 will).

**Navigation.** One `View` signal: Portfolio, Project, Engines, Inbox, Settings. The navigation is a list of buttons; the arrow keys move between them, and Alt+1 to Alt+5 open the views. Every focused element has a visible outline, and a skip link leads to the content.

**Views.** Portfolio lists the projects with add (folder dialog or typed path), rename, remove with confirmation, and locate for missing ones, plus the registry notices. Engines shows each engine's status, version, location, capabilities, the features it enables, and its last error output, with a Refresh button. Settings holds the theme and each engine's location and time limit. Project and Inbox say what they will show and that it comes in v0.2.0. Every section shows ready, degraded, unavailable with what is needed, or the release that brings it. Results and errors go to a polite live region.

**Theme.** `<html data-theme>` is `light` or `dark`, resolved from the setting and from `prefers-color-scheme` while the setting is "system". The dark tokens come from AnyContext's `AuroraTheme`, with muted text lightened to `#94A3B8` and errors to `#F87171` for contrast; the light tokens are derived. The test checks every text and background pair at 4.5:1 and focus indicators at 3:1, in both themes. Views use classes only, no inline styles, to fit the content security policy.

## Interfaces

Calls `aurora-desktop` commands and listens to its events; uses the types of `aurora-core`.

## Decisions

- [ADR-0002](../adr/0002-desktop-application-with-tauri-2-and-a-leptos-interface.md): Leptos interface, all Rust.
