---
title: Desktop Shell
summary: 'The Tauri 2 application: composition root, commands exposed to the interface, windows, packaging and updates.'
code: [crates/aurora-desktop]
---

# Desktop Shell

The Tauri 2 application: composition root, commands exposed to the interface, windows, packaging and updates. Not built yet: this page describes the design of the foundations.

## Responsibilities

- Build the adapters and wire them into the core at start-up (the composition root).
- Expose each core use case as one Tauri command, and long operations as events.
- Own the main window, the menu and the system integration.
- Package the application for Windows and Linux.
- It holds no product rules and no engine knowledge beyond choosing the adapters.

## Structure

`crates/aurora-desktop` is the Tauri crate (`src/main.rs`, `src/commands.rs`, `tauri.conf.json`); the interface's build output is its frontend.

## Behavior

Each command deserializes its input, calls the use case and returns the core's types. Results that take long (a refresh of many projects, an answer from AnyContext) are streamed as events. Tauri capabilities allow only Aurora's own commands; the web view gets no direct file system or shell access.

## Interfaces

Called by `aurora-ui` through Tauri's `invoke` and events. Calls `aurora-core` and builds `aurora-engines`.

## Decisions

- [ADR-0001](../adr/0001-hexagonal-architecture-with-the-engines-behind-ports.md): the composition root lives here.
- [ADR-0002](../adr/0002-desktop-application-with-tauri-2-and-a-leptos-interface.md): Tauri 2 shell.
- [ADR-0005](../adr/0005-proprietary-binary-only-distribution-with-levix-licenses.md): proprietary binaries; licensing through `levix-platform` later.
