---
title: Permissions
description: Unode's two-layer permission model, manifest declaration, enforcement, and default-deny guarantees.
---

Every capability a plugin can use is gated by a permission. Unode uses a
two-layer model enforced at different points, with a hard default-deny at the
sandbox boundary.

## Two-layer model

### Layer 1 — Core built-in permissions

Gate capabilities Unode provides regardless of the app. Enforced by the renderer
(a JS or Rust `PermissionGuard`) before each host function runs.

| Permission                 | Gates                                        |
| -------------------------- | -------------------------------------------- |
| `http.fetch`               | HTTP requests to declared origins            |
| `http.fetch.any`           | HTTP requests to any origin (dangerous)      |
| `storage.session.read`     | Read session-scoped plugin storage           |
| `storage.session.write`    | Write session-scoped plugin storage          |
| `storage.persistent.read`  | Read persistent plugin storage               |
| `storage.persistent.write` | Write persistent plugin storage              |
| `events.read`              | Subscribe to the host event bus              |
| `events.write`             | Emit events to the host event bus            |

### Layer 2 — App domain permissions

Gate app-specific APIs defined by the bridge, enforced by the bridge before it
delegates to the real implementation. For example:

| Permission      | Gates                                         |
| --------------- | --------------------------------------------- |
| `catalog.read`  | `catalog.getWork`, `catalog.search`, …        |
| `library.read`  | `library.isFavorited`                         |
| `library.write` | `library.addFavorite`, `library.removeFavorite` |
| `reader.open`   | `reader.openChapter`                          |

## Declaration in the manifest

A plugin declares each permission it needs, with a human-readable reason:

```rust
let manifest = PluginManifest {
    id: "com.example.catalog".into(),
    permissions: vec![
        PermissionRequest {
            permission: "http.fetch".into(),
            reason: "Fetch cover images from CDN".into(),
            required: false,
            allowed_origins: vec!["https://cdn.example.com".into()],
        },
        PermissionRequest {
            permission: "catalog.read".into(),
            reason: "Read works and chapters from the catalog".into(),
            required: true,
            allowed_origins: vec![],
        },
    ],
    ..Default::default()
};
```

## Approval and storage

At install time the host presents each `PermissionRequest` to the user, who
approves or denies it. The result is stored as a `PermissionProfile` of grants —
each recording the permission, whether it was granted, a timestamp, and (for
`http.fetch`) the approved origins.

## Enforcement

A `PermissionGuard` wraps the profile and exposes `has(permission)`,
`assert(permission)`, and `assert_origin(url)`. HTTP is enforced **twice**: host
functions for `http_fetch` are only injected when `http.fetch` was granted, and
the URL is checked against `allowed_origins` at call time. In the TUI, Wasmtime
can additionally deny network access at the OS level.

## Default deny

Any capability not declared in the manifest and not approved in the profile is
denied — and the plugin **never sees the method**. The host function is simply
not injected into the WASM imports.

Calling an un-injected import causes a WASM trap (Wasmtime) or
`WebAssembly.RuntimeError` (browser) before any permission-check code runs. This
is the strongest possible enforcement: the capability does not exist in the
plugin's execution environment.

## Bridge permission metadata

The bridge declares which permission each API method requires:

```rust
HostFn {
    name: "catalog_get_work",
    required_permission: "catalog.read",
    handler: Box::new(catalog_get_work_handler),
},
HostFn {
    name: "library_add_favorite",
    required_permission: "library.write",
    handler: Box::new(library_add_favorite_handler),
},
```

At instantiation the renderer iterates this list, checks
`guard.has(required_permission)` for each, and injects only the functions whose
permission was granted. The rest are absent from the imports object entirely.
