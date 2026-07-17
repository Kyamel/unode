# unode Permissions

## Two-layer model

Permissions have two orthogonal categories that are enforced at different points.

### Layer 1 -- Core built-in permissions

Gate access to capabilities that unode provides regardless of the app:

| Permission | Gates |
|---|---|
| `http.fetch` | HTTP requests to declared origins |
| `http.fetch.any` | HTTP requests to any origin (dangerous) |
| `storage.session.read` | Read session-scoped plugin storage |
| `storage.session.write` | Write session-scoped plugin storage |
| `storage.persistent.read` | Read persistent plugin storage |
| `storage.persistent.write` | Write persistent plugin storage |
| `events.read` | Subscribe to host event bus |
| `events.write` | Emit events to host event bus |

Enforced by: the trusted host runtime before a host function is injected and
again before sensitive operations execute. Web enforcement lives in the
JavaScript/runtime bridge plus `unode-web-runtime`; TUI enforcement lives in the
Wasmtime host-call layer.

### Layer 2 -- App domain permissions

Gate access to app-specific APIs defined by the bridge. Examples for Mugen:

| Permission | Gates |
|---|---|
| `catalog.read` | `catalog.getWork`, `catalog.listChapters`, `catalog.search` |
| `library.read` | `library.isFavorited` |
| `library.write` | `library.addFavorite`, `library.removeFavorite` |
| `reader.open` | `reader.openChapter` |

Enforced by: the host bridge before delegating to the real implementation.

---

## Declaration in plugin manifest

Permissions are typed at the authoring edges and plain strings on the wire.
The core ships its builtins as constants (`permissions::builtin::*`); apps
declare their own domain permissions with `Permission::new` -- the core never
enumerates or limits them. `Permission::scoped` applies the `resource:scope`
convention.

```rust
// In the app's SDK crate (domain permissions are app-defined):
pub const CATALOG_READ: Permission = Permission::new("catalog.read");

// In plugin source:
let manifest = plugin_manifest("com.mugenx.catalog", "Catalog")
    .permission(
        perm(builtin::HTTP_FETCH)
            .reason("Fetch cover images from CDN")
            .allow_origin("https://cdn.mugenx.com"),
    )
    .permission(
        perm(CATALOG_READ)
            .required(true)
            .reason("Read works and chapters from the catalog"),
    )
    .build();
```

---

## Approval and storage

When a plugin is installed, the host presents `PermissionRequest` items to the
user. The user approves or denies each. The result is stored as a
`PermissionProfile`:

```rust
pub struct PermissionProfile {
    pub plugin_id: String,
    pub grants: Vec<PermissionGrant>,
}

pub struct PermissionGrant {
    pub permission: String,
    pub granted: bool,
    pub granted_at: String,           // ISO 8601
    pub allowed_origins: Vec<String>, // populated for http.fetch
}
```

---

## Enforcement

```rust
pub struct PermissionGuard {
    profile: PermissionProfile,
}

impl PermissionGuard {
    pub fn has(&self, permission: &str) -> bool { ... }

    pub fn assert(&self, permission: &str) -> Result<(), PermissionDeniedError> {
        if !self.has(permission) {
            return Err(PermissionDeniedError {
                plugin_id: self.profile.plugin_id.clone(),
                permission: permission.into(),
            });
        }
        Ok(())
    }

    pub fn assert_origin(&self, url: &str) -> Result<(), OriginNotAllowedError> {
        self.assert("http.fetch")?;
        let approved = self.approved_origins();
        if !approved.iter().any(|o| origin_matches(o, url)) {
            return Err(OriginNotAllowedError { ... });
        }
        Ok(())
    }
}
```

---

## Default deny

Any method or capability not explicitly declared in the manifest and not
approved in the `PermissionProfile` is denied. The plugin never sees the
method -- the host function is simply not injected into the WASM imports.

Attempting to call an un-injected import causes a WASM trap (Wasmtime) or
`WebAssembly.RuntimeError` (browser) before any permission check code runs.
This is the strongest possible enforcement: the capability does not exist in
the plugin's execution environment.

---

## HTTP origin enforcement

HTTP permissions are enforced twice:

1. **Denylist at instantiation** -- host functions for `http_fetch` are only
   injected if `http.fetch` was granted.
2. **Origin check at call time** -- when `http_fetch` is called, the URL is
   checked against `allowed_origins` before the request is made.

In the TUI runtime, the Wasmtime sandbox can also enforce network access at the
OS level -- if the Wasmtime instance has no network capability, no host function
can make a network request regardless of what the Rust code does.

---

## Bridge permission metadata

The app bridge declares which permission each API method requires:

```rust
pub fn catalog_host_functions() -> HostApiFunctions {
    HostApiFunctions {
        functions: vec![
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
        ],
    }
}
```

The host runtime iterates this list at instantiation time, checks
`guard.has(required_permission)` for each function, and only injects the
functions whose permission was granted. Functions without a granted permission
are not present in the imports object at all.
