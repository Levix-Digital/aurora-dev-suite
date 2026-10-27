---
title: User Interface
summary: 'The Leptos interface compiled to WebAssembly: views, components and the Aurora Borealis theme.'
code: [crates/aurora-ui]
---

# User Interface

The Leptos interface compiled to WebAssembly: views, components and the Aurora Borealis theme. Not built yet: this page describes the design of the foundations.

## Responsibilities

- Render the views and send the user's actions as Tauri commands.
- The theme: Aurora Borealis tokens as CSS custom properties, light and dark.
- It holds no rules: filtering, ordering and statuses come from the core.

## Structure

`crates/aurora-ui` (Leptos, client-side rendering, built with Trunk). Views of the foundations: **Portfolio** (registered projects and their phase), **Project** (cycle state of one project), **Engines** (health and capabilities) and **Inbox** (what waits for the user). **Agents** and **Knowledge** come with Gedo and AnyContext.

## Behavior

Each view loads its data with one command and shows a per-section status (ready, degraded with the reason, unavailable). Without the desktop shell (in a browser during development) the interface uses the core's fixture data.

## Interfaces

Calls `aurora-desktop` commands; uses the types of `aurora-core`.

## Decisions

- [ADR-0002](../adr/0002-desktop-application-with-tauri-2-and-a-leptos-interface.md): Leptos interface, all Rust.
