---
title: Introduction
description: What Unode is, what it is not, and the ideas behind a plugin-first semantic UI protocol.
---

Unode is a renderer-agnostic **semantic UI protocol** for plugin-based
applications. Instead of shipping DOM, CSS, or terminal layout, a plugin
describes a screen as serializable *intent*. The host renders that intent in
whatever shape fits the environment it runs in.

## What it is

- A canonical, serializable AST that describes plugin UI as **intent, not
  presentation**.
- A runtime contract for routes, actions, local state, permissions, and plugin
  isolation.
- A **WASM-first** execution model where plugins compile to `.wasm` and run
  sandboxed on any host.
- A foundation that app-specific **bridges** extend with domain APIs.

## What it is not

- **Not a renderer.** Renderers are platform-specific implementations (a
  React/Svelte web adapter, a Ratatui TUI).
- **Not a design system.** Themes, spacing, and color live in the renderer.
- **Not a domain model.** Concepts like works, chapters, or users belong in the
  app bridge, never in the core.
- **Not tied to a language.** The protocol is JSON; the reference
  implementation happens to be Rust.

## Core principles

**Intent over presentation.** A plugin declares that something *is* a heading or
a danger action — never how many pixels it occupies or what color it should be.

**Protocol first.** The canonical AST is JSON. Every component — plugin, host,
renderer — communicates through this JSON protocol. No component ever receives a
live object from another; only serialized data crosses boundaries.

**WASM as the execution boundary.** Plugins compile to `.wasm`. The host
instantiates the module, provides host functions, and receives the AST as a JSON
string. Isolation is enforced without a separate process.

**Renderer as trust boundary.** The renderer owns sandboxing, theming, focus,
keyboard behavior, and permission enforcement. The plugin only declares intent.

**Domain isolation.** The core knows nothing about any app's concepts. Domain
knowledge lives entirely in the app bridge.

## How the pieces fit

```text
Plugin (Rust → .wasm)
  └── uses unode-plugin-sdk (DSL builders → CanonicalScreen JSON, host calls)

unode core (Rust → .wasm / native)
  ├── AST types + normalization
  ├── StateStore + ExprResolver (reactive binding tracking)
  ├── PermissionGuard
  └── Transport (JSON serialization)

App Bridge (Rust → .wasm)
  ├── Domain host API + models
  └── Permission metadata per method

Host + Renderer (per platform)
  ├── Web: loads plugin.wasm + unode_web_host.wasm; React/Svelte adapters
  └── TUI: loads plugin.wasm via Wasmtime; Ratatui renderer
```

Continue to [Installation](/getting-started/installation/) to set up a local
build, or jump to the [Quickstart](/getting-started/quickstart/) to write a
plugin.
