# unode WASM Execution Model

## Why WASM

WASM linear memory is isolated by the runtime. A plugin cannot read or write
host memory without an explicit host function that grants access. This is
enforced at the machine level, not by trusting the plugin to behave. No amount
of clever JavaScript or Rust can break out of the WASM sandbox without a host
function that enables it.

This is strictly stronger than iframe sandboxing (which only gates DOM access)
and strictly stronger than Deno Worker permissions (which gate syscalls but not
in-process memory).

---

## Plugin lifecycle

```
1. Host reads plugin.wasm
2. Host validates manifest (embedded in WASM exports or a sidecar JSON)
3. Host loads PermissionProfile from storage
4. Host checks: all required permissions granted?
   → No: reject, do not instantiate
5. Host builds host function set filtered by PermissionProfile
6. Host instantiates WASM module with the filtered host functions
7. Plugin's init() export called (optional — for one-time setup)
8. For each route match: host calls render_route(route_json) → screen_json
9. For each action:     host calls dispatch_action(action_json) → state_delta_json
10. On unmount: host drops the WASM instance
```

---

## Host functions

Host functions are the only way a plugin can access anything outside its own
memory. Each host function is gated by a permission check before executing.

### Core host functions (every renderer implements these)

```
// State
state_get(path_ptr, path_len) → value_ptr
state_set(path_ptr, path_len, value_ptr, value_len)
state_get_snapshot() → json_ptr

// Navigation
navigate(to_ptr, to_len, mode_ptr, mode_len)
navigate_back()

// HTTP (requires "http.fetch" permission)
http_fetch(url_ptr, url_len, opts_ptr, opts_len) → response_ptr

// Storage (requires "storage.*" permissions)
storage_get(scope_ptr, scope_len, key_ptr, key_len) → value_ptr
storage_set(scope_ptr, scope_len, key_ptr, key_len, value_ptr, value_len)

// Events (requires "events.*" permissions)
events_emit(type_ptr, type_len, payload_ptr, payload_len)
events_on(type_ptr, type_len) → subscription_id

// Locale
locale_get() → locale_ptr  // returns BCP 47 string, e.g. "pt-BR"
```

### Domain host functions (app bridge implements these)

Each function in `MugenHostApi` is registered as a host function with its
required permission declared in `HostApiMeta`. The host function is only
injected into the WASM module if the permission was granted.

```
// Example: catalog host functions
catalog_get_work(work_id_ptr, work_id_len) → work_json_ptr
catalog_list_chapters(work_id_ptr, work_id_len) → chapters_json_ptr
catalog_search_works(query_ptr, query_len) → results_json_ptr

// Example: library host functions
library_is_favorited(work_id_ptr, work_id_len) → bool
library_add_favorite(work_id_ptr, work_id_len)
library_remove_favorite(work_id_ptr, work_id_len)
```

---

## Memory protocol

WASM modules use linear memory. Strings and JSON are passed as `(ptr, len)` pairs.
The host reads from WASM memory at the given offset and length.

Responses (host → plugin) use a simple allocation protocol:

```rust
// Inside the plugin SDK — allocates a buffer in WASM memory
// and returns its pointer to the host
#[no_mangle]
pub extern "C" fn unode_alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub extern "C" fn unode_dealloc(ptr: *mut u8, len: usize) {
    unsafe { drop(Vec::from_raw_parts(ptr, len, len)) }
}
```

The host calls `unode_alloc`, writes the response JSON into WASM memory, and
passes the pointer back to the plugin. The plugin reads the JSON and calls
`unode_dealloc` when done. This avoids any copying through a shared buffer.

---

## Plugin exports

Every plugin module must export:

```rust
// Called once after instantiation
// Returns manifest JSON
#[no_mangle]
pub extern "C" fn plugin_manifest() -> *const u8;
pub extern "C" fn plugin_manifest_len() -> usize;

// Called for each route match
// route_json: { pattern, params, query, state_snapshot }
// Returns CanonicalScreen JSON
#[no_mangle]
pub extern "C" fn plugin_render(
    route_ptr: *const u8, route_len: usize,
    data_ptr:  *const u8, data_len:  usize,
) -> *const u8;
pub extern "C" fn plugin_render_result_len() -> usize;

// Called for load phase
// Returns JSON data to merge into StateStore
#[no_mangle]
pub extern "C" fn plugin_load(
    route_ptr: *const u8, route_len: usize,
) -> *const u8;
pub extern "C" fn plugin_load_result_len() -> usize;

// Called for action dispatch
// action_json: ActionRef
#[no_mangle]
pub extern "C" fn plugin_dispatch(
    action_ptr: *const u8, action_len: usize,
);
```

---

## Sandboxing in the Web renderer

In the Web renderer, plugins run as WASM modules instantiated via
`WebAssembly.instantiate()`. The browser's WASM runtime enforces memory
isolation. Host functions are JavaScript closures that check permissions
before executing:

```typescript
const imports = {
  unode: {
    state_set: (pathPtr, pathLen, valuePtr, valueLen) => {
      const path = readString(memory, pathPtr, pathLen);
      const value = readJson(memory, valuePtr, valueLen);
      stateStore.set(path, value);
      // No permission check needed — setState is always allowed
    },

    http_fetch: (urlPtr, urlLen, optsPtr, optsLen) => {
      const url = readString(memory, urlPtr, urlLen);
      guard.assertOrigin(url);  // throws PermissionDeniedError if not approved
      return fetch(url).then(r => r.json()).then(data => writeJson(memory, data));
    },

    catalog_get_work: (idPtr, idLen) => {
      guard.assert("catalog.read");  // throws if not granted
      const id = readString(memory, idPtr, idLen);
      return catalogApi.getWork(id).then(work => writeJson(memory, work));
    },
  }
};

const instance = await WebAssembly.instantiate(wasmBytes, imports);
```

---

## Sandboxing in the TUI renderer

In the TUI renderer, plugins run inside Wasmtime. Wasmtime's WASM sandbox
prevents any syscall or memory access outside the module boundary. Host
functions are Rust closures registered with the Wasmtime linker:

```rust
linker.func_wrap_async("unode", "catalog_get_work", {
    let guard = guard.clone();
    let catalog = catalog.clone();
    move |mut caller: Caller<'_, PluginState>, id_ptr: i32, id_len: i32| {
        let guard = guard.clone();
        let catalog = catalog.clone();
        Box::new(async move {
            // Permission check before any work
            guard.assert("catalog.read")?;

            let id = read_string(&mut caller, id_ptr, id_len)?;
            let work = catalog.get_work(&id).await?;
            let json = serde_json::to_string(&work)?;
            write_string(&mut caller, &json)
        })
    }
})?;
```

---

## Plugin-to-plugin communication

Plugins do not communicate directly. All inter-plugin communication goes
through the host event bus:

```
Plugin A (WASM)
  ctx.events.emit("screen.refresh", { pathname: "/browse" })
    → host function: events_emit(...)
      → host event bus broadcasts to all subscribed plugins

Plugin B (WASM)
  ctx.events.on("screen.refresh", handler)
    → host function: events_on(...)
      → host registers a callback that calls plugin_dispatch() on Plugin B
```

The host controls which plugins can subscribe to which event types based on
their `events.read` and `events.write` permissions.
