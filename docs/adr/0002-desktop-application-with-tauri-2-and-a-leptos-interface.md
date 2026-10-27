---
id: 2
title: Desktop application with Tauri 2 and a Leptos interface
status: accepted
date: '2026-10-27'
version: null
subsystems: [desktop, ui]
supersedes: []
superseded_by: []
amends: []
amended_by: []
---

# ADR-0002: Desktop application with Tauri 2 and a Leptos interface

## Context

Levi decided on 2026-10-27 that Aurora is a desktop application in Rust from the start, without a terminal interface. The application is a dashboard: lists, states, a decision inbox and question-and-answer with citations rendered from Markdown. It must run on Windows and Linux like the engines. The `levix-platform` design already expects Tauri desktop products to use its crates directly, and AnyContext owns the Aurora Borealis design tokens.

## Decision

- **Tauri 2** is the desktop shell: a Rust process (`aurora-desktop`) that owns the windows, calls the core and the engine adapters, and exposes one Tauri command per core use case.
- **Leptos** (client-side rendering compiled to WebAssembly) is the interface (`aurora-ui`), so the whole product is written in Rust. The interface calls the Tauri commands and receives the core's own types (serde), never engine output.
- The Aurora Borealis tokens become CSS custom properties in the interface.
- Targets: Windows x86_64 (WebView2) and Linux x86_64 (WebKitGTK), as the engines.

Chosen by Levi on 2026-10-27 among three options.

## Alternatives Considered

- **Tauri 2 with a TypeScript and React interface**: the largest ecosystem of components and charts, but the interface would not be Rust and the core types would need a second definition.
- **Iced (native, no web view)**: one native binary without WebView2 or WebKitGTK, but fewer ready components for tables, charts and Markdown, and the theme could not reuse web tooling.

## Consequences

- One language and one type definition from the engines to the screen.
- The build needs the `wasm32-unknown-unknown` target and a WebAssembly bundler (Trunk) besides Cargo.
- Linux users need WebKitGTK; the installer checks for it.
- Leptos has fewer ready-made components than React; charts and Markdown rendering may need small components of our own.
