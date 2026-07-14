# unode Platform Environments

This document formalizes how unode is implemented across three target environments: Web (browser), Tauri (desktop), and TUI (terminal). It focuses on sandboxing strategy and the internal API changes each environment requires.

The unode core — AST, DSL, PluginContext interface, ActionRegistry contracts, permission types — does not change across environments. What changes is how each environment *instantiates* and *enforces* those contracts.

---

## Conceptual model

Every environment implements the same protocol between host and plugin:

```
Plugin (isolated)          Host (trusted)
─────────────────          ──────────────
load(ctx)      ←── ctx ───  PluginContextAdapter
     │                            │
     │ returns data               │ enforces permissions
     ↓                            │ resolves API calls
render(data)                      │
     │                            │
     └──── CanonicalScreen ──────→ Renderer
                                  │
                                  ↓
                             Output (DOM / terminal cells)
```

The isolation boundary between "Plugin" and "Host" is implemented differently per environment. The `PluginContextAdapter` is the environment-specific class that bridges the gap.

---

## Environment 1 — Web (browser only, no Tauri)

### Threat model

In a pure browser context, the main risks are:

- Plugin reading host application state it should not see
- Plugin making unauthorized network requests
- Plugin injecting content into the host DOM

The browser already prevents filesystem access, native process execution, and cross-origin network requests (via CORS). The sandbox requirement is therefore lighter than in native environments.

### Sandboxing strategy: iframe with restricted origin

Each plugin runs inside a sandboxed iframe served from a distinct origin or blob URL. The iframe has no access to `window` of the host application.

```
Host application (https://app.mugenx.com)
└── iframe (blob: or https://plugins.mugenx.com/catalog)
      sandbox="allow-scripts"
      csp="default-src 'none'; script-src 'self' 'unsafe-inline'"
```

The `sandbox="allow-scripts"` attribute without `allow-same-origin` means the iframe cannot read or write cookies, localStorage, or indexedDB of the host origin. It cannot call `parent.document` or access the host DOM.

Communication happens exclusively via `postMessage`. The host validates the origin of every incoming message.

### Internal API changes

#### PluginContextAdapter (web iframe)

The adapter runs inside the iframe alongside the plugin code. It implements the `PluginContext` interface by translating every call into a `postMessage` RPC to the host.

```typescript
// Runs inside the plugin iframe
class WebIframePluginContextAdapter implements PluginContext<unknown> {
  private readonly pending = new Map<string, (v: unknown) => void>();

  constructor(private readonly pluginId: string) {
    window.addEventListener("message", (event) => {
      if (event.origin !== TRUSTED_HOST_ORIGIN) return;
      const msg = event.data as HostToPluginMessage;
      if (msg.type === "rpc-response") {
        this.pending.get(msg.id)?.(msg.result);
        this.pending.delete(msg.id);
      }
    });
  }

  private rpc<T>(method: string, args: unknown[]): Promise<T> {
    const id = crypto.randomUUID();
    return new Promise((resolve) => {
      this.pending.set(id, resolve as (v: unknown) => void);
      parent.postMessage(
        { type: "rpc", id, pluginId: this.pluginId, method, args },
        TRUSTED_HOST_ORIGIN
      );
    });
  }

  // ctx.api is a recursive Proxy that converts property access into RPC calls
  readonly api = buildRpcProxy((method, args) => this.rpc(method, args));

  readonly http: UNodeHttpApi = {
    fetch: (url, init) => this.rpc("unode.http.fetch", [url, init]),
    getJson: (url, headers) => this.rpc("unode.http.getJson", [url, headers]),
    postJson: (url, body, headers) => this.rpc("unode.http.postJson", [url, body, headers]),
  };

  readonly storage: UNodeStorageApi = {
    get: (scope, key) => this.rpc("unode.storage.get", [scope, key]),
    set: (scope, key, value) => this.rpc("unode.storage.set", [scope, key, value]),
    delete: (scope, key) => this.rpc("unode.storage.delete", [scope, key]),
    keys: (scope) => this.rpc("unode.storage.keys", [scope]),
  };

  // state, navigate, dispatch follow the same RPC pattern
}
```

#### PluginHost (web, runs in the host app)

The host side manages one iframe per plugin and handles incoming RPC messages:

```typescript
class WebPluginHost {
  private readonly guard: PermissionGuard;
  private readonly iframeOrigin: string;

  handleMessage(event: MessageEvent): void {
    // Reject messages from unknown origins
    if (event.origin !== this.iframeOrigin) return;

    const msg = event.data as PluginToHostMessage;
    if (msg.type !== "rpc") return;

    this.dispatch(msg).then((result) => {
      event.source?.postMessage(
        { type: "rpc-response", id: msg.id, result },
        { targetOrigin: this.iframeOrigin }
      );
    });
  }

  private async dispatch(msg: RpcMessage): Promise<unknown> {
    // Built-in unode capabilities
    if (msg.method.startsWith("unode.http.")) {
      this.guard.assert("http.fetch");
      this.guard.assertOrigin(msg.args[0] as string);
      return this.executeHttp(msg.method, msg.args);
    }

    if (msg.method.startsWith("unode.storage.")) {
      const op = msg.method.includes("get") ? "storage.session.read" : "storage.session.write";
      this.guard.assert(op as UNodePermission);
      return this.executeStorage(msg.method, msg.args);
    }

    // Host domain API — checked against HostApiMeta
    const methodMeta = this.apiMeta[msg.method];
    if (!methodMeta) throw new PermissionDeniedError(this.pluginId, msg.method);
    this.guard.assert(methodMeta.permission);
    return this.executeApiMethod(msg.method, msg.args);
  }
}
```

#### Screen delivery

After `render()` runs inside the iframe, the plugin sends the `CanonicalScreen` to the host via postMessage:

```typescript
// Inside iframe, after render() is called
const screen = plugin.routes[0].render(data, ctx);
parent.postMessage(
  { type: "screen", pluginId, screen: screen }, // screen is frozen JSON-safe
  TRUSTED_HOST_ORIGIN
);
```

The host receives it and passes it to the Svelte renderer directly. No deserialization needed — `postMessage` in the same browser process does structured clone automatically.

### What does NOT change in the Web environment

- The plugin TypeScript source code
- The unode DSL (`ui.*` builders)
- The `PluginDefinition` interface
- The `PermissionProfile` and `PermissionGuard` types
- The `CanonicalScreen` AST shape

---

## Environment 2 — Tauri (desktop webview)

### Threat model

Tauri changes the threat model significantly. The webview has access to a `window.__TAURI__.invoke()` bridge that can call Rust commands with filesystem, network, and OS access. A plugin running in the same webview as the host application can call this bridge directly, bypassing all JavaScript-level permission checks.

The additional risks compared to pure web:

- Plugin calling `invoke("read_file", { path: "/etc/passwd" })`
- Plugin calling `invoke("shell_execute", { cmd: "..." })`
- Plugin accessing native resources the host intended to keep private

### Sandboxing strategy: iframe + Tauri Capabilities per webview

Tauri 2 introduces a Capabilities system that gates `invoke()` access per window label. The architecture exploits this:

1. Each plugin runs in a dedicated Tauri WebviewWindow (not just an HTML iframe, but a separate OS-level webview)
2. Each plugin webview is assigned a Capability profile that maps to its `PermissionProfile`
3. The plugin webview has `__TAURI__` access only to the specific invoke commands its Capability allows
4. The main app webview retains full Tauri access

```
Main WebviewWindow (label: "main")
  Capabilities: all Tauri commands
  Runs: host Svelte app + Renderer

Plugin WebviewWindow (label: "plugin-com.mugenx.catalog")
  Capabilities: only "catalog:read", "library:read", "library:write"
  Runs: plugin code + WebviewPluginContextAdapter
  No access to: filesystem commands, shell commands, other Tauri APIs
```

```json
// tauri-capabilities/plugin-catalog.json
{
  "identifier": "plugin-com.mugenx.catalog",
  "description": "Capability profile for catalog plugin",
  "windows": ["plugin-com.mugenx.catalog"],
  "permissions": [
    "unode-bridge:catalog-read",
    "unode-bridge:library-read",
    "unode-bridge:library-write"
  ]
}
```

The Tauri commands exposed to plugin webviews are thin wrappers that:
1. Identify which plugin is calling (from the window label)
2. Look up the plugin's PermissionProfile
3. Assert the required permission
4. Execute and return the result

```rust
// src-tauri/src/plugin_bridge.rs
#[tauri::command]
async fn catalog_get_work(
  window: tauri::Window,
  work_id: String,
  state: tauri::State<'_, AppState>,
) -> Result<Work, String> {
  let plugin_id = plugin_id_from_window_label(window.label());
  let profile = state.permission_profiles.get(&plugin_id)?;
  let guard = DefaultPermissionGuard::new(profile);
  guard.assert("catalog.read")?; // app domain permission check

  state.catalog.get_work(&work_id).await.map_err(|e| e.to_string())
}
```

### Communication between plugin webview and main webview

Plugin webviews cannot directly postMessage to the main webview in Tauri. Communication goes through the Rust layer:

```
Plugin WebviewWindow
  → invoke("unode_plugin_emit", { event: "screen", payload: canonicalScreen })
  → Rust receives, identifies source plugin
  → Rust emits Tauri event to main window
Main WebviewWindow
  → tauri::listen("unode:plugin:screen", handler)
  → Renderer mounts the received CanonicalScreen
```

#### PluginContextAdapter (Tauri webview)

Inside the plugin webview, the adapter uses `invoke()` instead of `postMessage`:

```typescript
// Runs inside the plugin's dedicated WebviewWindow
import { invoke } from "@tauri-apps/api/core";

class TauriWebviewPluginContextAdapter implements PluginContext<unknown> {
  readonly api = buildRpcProxy((method, args) =>
    invoke("unode_api_call", { method, args })
    // Tauri capability check happens on the Rust side
    // The plugin webview can only invoke commands in its Capability profile
  );

  readonly http: UNodeHttpApi = {
    fetch: (url, init) => invoke("unode_http_fetch", { url, init }),
    // ...
  };

  // state and navigate emit events back to the main window via Rust
  readonly state: StateStore = buildRemoteStateStore((op) =>
    invoke("unode_state_op", { op })
  );
}
```

### Key difference from pure web

In pure web, the iframe sandbox is enforced by the browser's HTML sandbox attribute. In Tauri, the sandbox is enforced by the Capabilities system at the OS webview level. The plugin cannot call any Rust command not listed in its Capability profile, regardless of what JavaScript it runs.

This means the Tauri environment provides a stronger sandbox than pure web iframe for native resource access, while maintaining the same unode interface for plugin authors.

### What does NOT change in the Tauri environment

Everything in the plugin source code remains identical to the web environment. The `PluginContextAdapter` implementation changes (uses `invoke` instead of `postMessage`), but its interface — and therefore the plugin's experience — is the same.

---

## Environment 3 — TUI (Rust native, no webview)

### Threat model

The TUI app is a native Rust binary. There is no browser sandbox, no webview, no DOM. Plugin isolation must be implemented entirely in userspace. The risks are:

- Plugin code accessing the filesystem directly (if running in the same process)
- Plugin consuming unbounded memory or CPU
- Plugin calling Rust FFI directly

### Sandboxing strategy: WASM via Wasmtime + Extism

Each plugin is distributed as a `.wasm` module. The module contains the plugin's TypeScript compiled to JavaScript, bundled together with a minimal QuickJS runtime compiled to WASM. This is the Extism model.

The plugin code itself is unmodified TypeScript. The build toolchain for the plugin package produces two artifacts:

```
plugin-catalog/
  dist/
    web/
      index.js        ← ES module, for web and Tauri environments
    native/
      plugin.wasm     ← JS bundle + QuickJS runtime compiled to WASM
```

The WASM module cannot access the host filesystem, cannot make syscalls, and cannot allocate beyond the memory limits set by the Wasmtime host. This is enforced by the WASM sandbox itself, not by unode.

### Internal API changes

#### Host functions (Rust side)

The Rust TUI renderer registers host functions into the Wasmtime instance. These are the only entry points the plugin has into the host:

```rust
// src/plugin_runtime/host_functions.rs

fn register_host_functions(
  linker: &mut Linker<PluginState>,
  guard: Arc<PermissionGuard>,
) -> Result<()> {

  // unode.api.catalog.getWork
  linker.func_wrap_async("unode", "api_call", {
    let guard = guard.clone();
    move |mut caller: Caller<'_, PluginState>, method_ptr: i32, args_ptr: i32| {
      let guard = guard.clone();
      Box::new(async move {
        let method = read_string(&mut caller, method_ptr)?;
        let args_json = read_string(&mut caller, args_ptr)?;

        // Permission check before execution
        let required = API_METHOD_PERMISSIONS.get(method.as_str())
          .ok_or_else(|| anyhow!("Unknown method: {}", method))?;
        guard.assert(required)?;

        let result = caller.data().app_state.execute_api(&method, &args_json).await?;
        write_string(&mut caller, &result)
      })
    }
  })?;

  // unode.http.fetch — gated by "http.fetch" permission
  linker.func_wrap_async("unode", "http_fetch", {
    let guard = guard.clone();
    move |mut caller: Caller<'_, PluginState>, url_ptr: i32, opts_ptr: i32| {
      let guard = guard.clone();
      Box::new(async move {
        let url = read_string(&mut caller, url_ptr)?;
        guard.assert_origin(&url)?; // checks approved origins
        // execute fetch via reqwest
        let result = caller.data().http_client.get(&url).send().await?;
        let body = result.text().await?;
        write_string(&mut caller, &body)
      })
    }
  })?;

  Ok(())
}
```

#### PluginContextAdapter (inside WASM, TypeScript side)

Inside the WASM module, the plugin sees a `PluginContext` implemented against the WASM host function imports:

```typescript
// Runs inside the WASM module — compiled alongside the plugin
// Uses Extism PDK to call host functions

import { hostFn, Memory } from "@extism/pdk";

// Declared host function — implemented in Rust above
declare function api_call(methodPtr: i64, argsPtr: i64): i64;
declare function http_fetch(urlPtr: i64, optsPtr: i64): i64;

class WasmPluginContextAdapter implements PluginContext<unknown> {
  readonly api = buildRpcProxy((method, args) => {
    const methodMem = Memory.fromString(method);
    const argsMem = Memory.fromString(JSON.stringify(args));
    const resultPtr = api_call(methodMem.offset, argsMem.offset);
    return JSON.parse(Memory.find(resultPtr).readString());
  });

  readonly http: UNodeHttpApi = {
    fetch: (url) => {
      const urlMem = Memory.fromString(url);
      const resultPtr = http_fetch(urlMem.offset, 0n);
      return Memory.find(resultPtr).readString();
    },
    // ...
  };
}
```

#### Screen delivery (TUI)

After `render()` runs inside the WASM module, the CanonicalScreen is serialized to JSON and returned as the WASM module's output:

```typescript
// Inside WASM module — entry point called by Rust
export function run_load_render(): void {
  const ctx = new WasmPluginContextAdapter();
  const data = plugin.routes[0].load(ctx); // async resolved by QuickJS event loop
  const screen = plugin.routes[0].render(data, ctx);
  const json = JSON.stringify(screen); // CanonicalScreen is JSON-safe by design
  Memory.outputString(json);
}
```

On the Rust side:

```rust
// Call the WASM plugin and receive the CanonicalScreen
let output = plugin_instance.call("run_load_render", route_context_json)?;
let screen: CanonicalScreen = serde_json::from_slice(&output)?;
// Pass to TUI renderer
tui_renderer.mount(screen, state_store);
```

### State management in TUI

In web and Tauri, the StateStore lives in JavaScript inside the host app. In TUI, the StateStore lives in Rust. State reads and writes from plugin action handlers cross the WASM boundary:

```
Plugin action handler (WASM/JS)
  ctx.state.set("isFavorited", true)
    → WASM host function: state_set("isFavorited", "true")
      → Rust StateStore.set("isFavorited", true)
        → Subscriber notification
          → TUI renderer patches affected terminal cells
```

This means action handlers in TUI have higher latency per state operation than in web (each call crosses the WASM boundary). For typical plugin interactions — a button press triggers one or two state mutations — this is imperceptible. For tight loops, it would be a concern, but tight loops have no place in UI action handlers.

---

## Environment 4 — TUI (Deno-based, no Rust)

### Motivation

The Rust + Wasmtime approach gives the strongest possible sandbox and the best performance ceiling, but it has significant upfront cost: a Rust codebase, WASM build toolchain for plugins, host function FFI, and a custom TUI rendering layer. For a first TUI implementation — or for a TUI that does not require third-party plugin isolation — a Deno-based host is substantially simpler while still being a correct implementation of the unode contract.

Deno's built-in permission system also maps almost directly onto unode's `UNodePermission` model, which means a lot of the sandbox infrastructure comes for free.

### Threat model

Deno runs plugins as TypeScript in the same V8 process as the host. Unlike WASM, there is no memory isolation between host and plugin. The Deno sandbox is capability-based at the OS level — it controls what system calls the entire process can make — not at the plugin-within-process level.

This means the Deno TUI environment has a different (weaker) isolation guarantee than Rust + WASM:

- All plugins and the host share the same V8 heap — a plugin can in principle reach host objects via prototype chains or shared module state if not carefully structured
- A plugin that bypasses the `PluginContextAdapter` and directly imports a Deno API (`Deno.readFile`, `Deno.connect`) is not blocked at the runtime level the way it would be in WASM

The appropriate threat model for Deno TUI is therefore: **trusted or audited plugins, or plugins from a known registry**. It is not the right model for running arbitrary third-party plugins without review. For that scenario, the Rust + WASM approach is necessary.

If you want stronger isolation within Deno, Deno's `Worker` with `--permissions` provides subprocess-level isolation (see below). That is the recommended approach for untrusted plugins even in the Deno environment.

### Sandboxing strategy: Deno Worker per plugin

Deno supports spawning `Worker` threads with an isolated permission set:

```typescript
// Host creates one Worker per plugin
const worker = new Worker(
  new URL("./plugin-runner.ts", import.meta.url).href,
  {
    type: "module",
    deno: {
      permissions: {
        // Derived from the plugin's PermissionProfile
        net: ["cdn.mugenx.com"],   // "http.fetch" permission, allowedOrigins
        read: false,               // no filesystem read
        write: false,              // no filesystem write
        run: false,                // no subprocess execution
        env: false,                // no environment variable access
        ffi: false,                // no native FFI
      },
    },
  }
);
```

Each plugin Worker runs in its own V8 context with its own module graph. It cannot import modules from the host's module graph, and its `Deno.*` API access is limited to the declared permissions.

Communication between host and plugin uses the same `postMessage` / `MessageChannel` API as web Workers — structurally identical to the iframe model in the web environment. This is not coincidental: the Deno Worker model was explicitly designed to mirror the browser Worker model.

```
Deno host process
├── Host TUI app (main thread)
│     ├── TUI renderer (Ink or raw terminal)
│     ├── ActionRegistry
│     ├── StateStore
│     └── PluginHost
│           ├── Worker: plugin-com.mugenx.catalog
│           │     permissions: { net: ["cdn.mugenx.com"], read: false, ... }
│           └── Worker: plugin-com.mugenx.reader
│                 permissions: { net: false, read: false, ... }
└── (shared V8 process, isolated V8 contexts per Worker)
```

### Why this maps cleanly to unode permissions

Deno's permission flags map almost one-to-one to `UNodePermission`:

| UNodePermission | Deno Worker permission |
|---|---|
| `http.fetch` | `net: [allowedOrigins]` |
| `http.fetch.any` | `net: true` |
| `storage.persistent.read` | `read: [namespaced path]` |
| `storage.persistent.write` | `write: [namespaced path]` |
| `storage.session.*` | No Deno equivalent needed — in-memory |
| `clipboard.*` | No Deno equivalent — not a Deno API |
| `events.*` | No Deno equivalent — MessageChannel |

The host builds the Deno Worker permission object directly from the plugin's `PermissionProfile` when instantiating the Worker. This means OS-level permission enforcement (via Deno's seccomp/pledge layer) backs up the JavaScript-level `PermissionGuard` checks. Even if the JS guard is bypassed, the Worker cannot make the syscall.

### Internal API changes

#### plugin-runner.ts (runs inside the Worker)

The runner is a small bootstrap script that the host loads into every plugin Worker. It imports the plugin module, instantiates the `DenoWorkerPluginContextAdapter`, and handles `postMessage` dispatch:

```typescript
// plugin-runner.ts — runs inside the Deno Worker
import type { PluginDefinition } from "unode";
import { DenoWorkerPluginContextAdapter } from "@unode/deno-adapter";

// The plugin module path is sent as the first message from the host
self.addEventListener("message", async (event) => {
  if (event.data.type !== "init") return;

  const { pluginId, pluginPath, route, locale } = event.data;

  // Dynamic import of the plugin module
  // The Worker's module graph is isolated — it cannot reach host modules
  const { default: plugin } = await import(pluginPath) as {
    default: PluginDefinition
  };

  const ctx = new DenoWorkerPluginContextAdapter(pluginId, locale);

  // Find matching route
  const matchedRoute = plugin.routes.find(r => matchPattern(r.pattern, route.pattern));
  if (!matchedRoute) {
    self.postMessage({ type: "error", error: "No matching route" });
    return;
  }

  // Execute load/render cycle
  const data = await matchedRoute.load(ctx);
  const screen = matchedRoute.render(data, ctx);

  // CanonicalScreen is JSON-safe — postMessage structured-clones it
  self.postMessage({ type: "screen", screen });

  // Keep Worker alive for action handling
  ctx.listenForActions(plugin);
}, { once: true });
```

#### DenoWorkerPluginContextAdapter

Inside the Worker, the adapter translates `PluginContext` calls into `postMessage` to the host, identical in structure to the web iframe adapter:

```typescript
// @unode/deno-adapter — runs inside the Worker
class DenoWorkerPluginContextAdapter implements PluginContext<unknown> {
  private readonly pending = new Map<string, (v: unknown) => void>();

  constructor(
    readonly pluginId: string,
    readonly locale: string,
  ) {
    self.addEventListener("message", (event) => {
      const msg = event.data;
      if (msg.type === "rpc-response") {
        this.pending.get(msg.id)?.(msg.result);
        this.pending.delete(msg.id);
      }
      if (msg.type === "rpc-error") {
        // reject the pending promise
      }
    });
  }

  private rpc<T>(method: string, args: unknown[]): Promise<T> {
    const id = crypto.randomUUID();
    return new Promise((resolve, reject) => {
      this.pending.set(id, resolve as (v: unknown) => void);
      self.postMessage({ type: "rpc", id, method, args });
    });
  }

  // Identical proxy pattern to the web iframe adapter
  readonly api = buildRpcProxy((method, args) => this.rpc(method, args));

  readonly http: UNodeHttpApi = {
    // http.fetch is also gated by the Worker's net permission at OS level.
    // The JS-level guard on the host side is a second layer of defence.
    fetch: (url, init) => this.rpc("unode.http.fetch", [url, init]),
    getJson: (url, headers) => this.rpc("unode.http.getJson", [url, headers]),
    postJson: (url, body, headers) => this.rpc("unode.http.postJson", [url, body, headers]),
  };

  readonly storage: UNodeStorageApi = {
    get: (scope, key) => this.rpc("unode.storage.get", [scope, key]),
    set: (scope, key, value) => this.rpc("unode.storage.set", [scope, key, value]),
    delete: (scope, key) => this.rpc("unode.storage.delete", [scope, key]),
    keys: (scope) => this.rpc("unode.storage.keys", [scope]),
  };

  readonly state: StateStore = buildRemoteStateStore(
    (op) => this.rpc("unode.state", [op])
  );

  readonly navigate = (to: string, opts?: NavigateOptions) =>
    this.rpc("unode.navigate", [to, opts]);

  readonly dispatch = (action: ActionRef) =>
    this.rpc("unode.dispatch", [action]);

  // After screen is mounted, the host sends "action" messages here
  listenForActions(plugin: PluginDefinition): void {
    self.addEventListener("message", async (event) => {
      const msg = event.data;
      if (msg.type !== "action") return;

      const handler = plugin.actions?.[msg.action.type];
      if (!handler) return;

      await handler(this, msg.action.params ?? {});
    });
  }
}
```

#### PluginHost (Deno host, main thread)

The host side mirrors the web iframe host, but uses Worker messaging instead of iframe postMessage:

```typescript
// Runs on main thread in the Deno TUI host
class DenoPluginHost {
  private readonly workers = new Map<string, Worker>();

  async register(
    plugin: PluginDefinition,
    profile: PermissionProfile,
    pluginPath: string,
  ): Promise<void> {
    const guard = new DefaultPermissionGuard(profile);

    // Build Deno-level permission object from PermissionProfile
    // This enforces permissions at OS/syscall level, not just JS level
    const denoPermissions = buildDenoPermissions(profile);

    const worker = new Worker(
      new URL("./plugin-runner.ts", import.meta.url).href,
      { type: "module", deno: { permissions: denoPermissions } }
    );

    worker.addEventListener("message", (event) =>
      this.handleWorkerMessage(plugin.id, guard, event)
    );

    this.workers.set(plugin.id, worker);
  }

  private async handleWorkerMessage(
    pluginId: string,
    guard: PermissionGuard,
    event: MessageEvent,
  ): Promise<void> {
    const msg = event.data;

    if (msg.type === "screen") {
      // Deliver CanonicalScreen to the TUI renderer
      this.tuiRenderer.mount(msg.screen, this.stateStore);
      return;
    }

    if (msg.type === "rpc") {
      const result = await this.dispatch(pluginId, guard, msg);
      const worker = this.workers.get(pluginId)!;
      worker.postMessage({ type: "rpc-response", id: msg.id, result });
    }
  }

  private async dispatch(
    pluginId: string,
    guard: PermissionGuard,
    msg: RpcMessage,
  ): Promise<unknown> {
    // unode built-in: state
    if (msg.method === "unode.state") {
      return this.handleStateOp(msg.args[0] as StateOp);
    }

    // unode built-in: navigate
    if (msg.method === "unode.navigate") {
      return this.navigator.navigate(msg.args[0] as string, msg.args[1]);
    }

    // unode built-in: http
    if (msg.method.startsWith("unode.http.")) {
      guard.assert("http.fetch");
      guard.assertOrigin(msg.args[0] as string);
      // Note: the Worker's net permission also blocks this at OS level
      // if the origin was not declared — double enforcement
      return this.executeHttp(msg.method, msg.args);
    }

    // unode built-in: storage
    if (msg.method.startsWith("unode.storage.")) {
      const permission = storagePermissionFor(msg.method);
      guard.assert(permission);
      return this.executeStorage(pluginId, msg.method, msg.args);
    }

    // Host domain API
    const methodMeta = this.apiMeta[msg.method];
    if (!methodMeta) throw new PermissionDeniedError(pluginId, msg.method);
    guard.assert(methodMeta.permission);
    return this.executeApiMethod(msg.method, msg.args);
  }
}
```

### TUI rendering layer

The Deno TUI host needs a terminal rendering library. Two realistic options:

**Ink (React for CLIs)** — runs on Deno via compatibility shim. Rich layout engine, familiar component model. The unode renderer for this environment would be an Ink component tree built from `CanonicalScreen`:

```typescript
// DenoInkRenderer: walks CanonicalScreen and produces Ink components
function renderNode(node: CanonicalNode, ctx: ResolverContext): React.ReactNode {
  switch (node.kind) {
    case "stack":
      return <Box flexDirection="column" gap={resolveGap(node.gap)}>
        {node.children.map(c => renderNode(c, ctx))}
      </Box>;
    case "text":
      return <Text color={resolveTone(node.tone)}>
        {resolver.resolveString(node.content, ctx)}
      </Text>;
    case "action":
      return <ActionItem node={node} ctx={ctx} />;
    // ...
  }
}
```

**Raw terminal control (ansi-escapes + manual layout)** — more work but no React dependency. Better for a truly minimal TUI. This is what tools like Zellij and Helix do natively.

For a first implementation, Ink is the pragmatic choice. The renderer can be replaced with raw terminal control later without changing the unode contracts.

### State management in Deno TUI

Unlike the Rust TUI where StateStore lives in Rust, in Deno TUI the StateStore lives in the main thread (TypeScript). This is the same model as web and Tauri:

```
Plugin Worker
  ctx.state.set("isFavorited", true)
    → postMessage({ type: "rpc", method: "unode.state", args: [{ op: "set", path: "...", value: true }] })

Main thread DenoPluginHost
  → receives RPC
  → guard.assert("storage.session.write") if needed
  → stateStore.set("isFavorited", true)
    → subscriber notification
      → Ink renderer re-renders affected component
```

This is lower latency than the Rust WASM approach because there is no WASM boundary crossing — just in-process Worker message passing, which Deno handles efficiently via shared ArrayBuffer where possible.

### Hot reload

Deno TUI has the best hot reload story of all native environments. Because plugins are TypeScript modules loaded via dynamic `import()` in Workers, and Deno has `--watch` mode, you can:

1. Terminate the plugin Worker
2. Re-import the updated module (Deno's module cache is busted by file change)
3. Re-instantiate the `DenoWorkerPluginContextAdapter`
4. Re-run `load()` and `render()` for the current route

This is essentially the same DX as Vite HMR, but in a terminal.

### Deno vs Rust+WASM: when to choose which

| Criterion | Deno TUI | Rust + WASM TUI |
|---|---|---|
| Implementation cost | Low — mostly TypeScript | High — Rust + WASM toolchain |
| Memory isolation between plugins | Worker V8 context (moderate) | WASM linear memory (strong) |
| Plugin can bypass JS-level guards | Yes, if it avoids the adapter | No — WASM boundary is enforced |
| OS-level syscall enforcement | Yes, via Deno Worker permissions | Yes, via WASM sandbox |
| Performance ceiling | V8 JIT (excellent) | V8 JIT inside WASM (slightly lower) |
| Async/await in plugins | Native, no special handling | QuickJS event loop inside WASM |
| Hot reload in dev | Excellent | Requires Worker teardown |
| Binary distribution | Requires Deno runtime | Self-contained binary |
| Right for | Trusted plugins, dev tooling, internal apps | Third-party plugins, production TUI app |

The Deno TUI is the right first implementation if you want to validate the architecture without a Rust codebase. The Rust + WASM TUI is the right production implementation if you need a self-contained binary or stronger plugin isolation guarantees.

Both implement the same unode contracts. Switching between them does not require changes to plugin source code.

---

## Cross-environment comparison

| Concern | Web (browser) | Tauri (desktop) | TUI Rust+WASM | TUI Deno |
|---|---|---|---|---|
| Plugin execution context | Sandboxed iframe | Dedicated WebviewWindow | WASM module (Wasmtime) | Deno Worker |
| Native resource access | Browser prevents | Tauri Capabilities | WASM sandbox | Deno Worker permissions |
| Memory isolation | Strong (iframe origin) | Strong (OS webview) | Strong (WASM linear memory) | Moderate (V8 context) |
| Communication to host | `postMessage` | `invoke()` + Tauri events | WASM host functions | `postMessage` (Worker) |
| Permission enforcement | JS handler + browser | Rust command handler | Rust host function | JS handler + Deno syscall |
| StateStore location | JS, main thread | JS, main thread | Rust | JS, main thread |
| Screen delivery | `postMessage` structured clone | Tauri event JSON | WASM output JSON | `postMessage` structured clone |
| Async in plugin | Native Promises | Native Promises | QuickJS event loop | Native Promises |
| Plugin build artifact | ES module `.js` | ES module `.js` | WASM module `.wasm` | ES module `.js` |
| Plugin source changes | None | None | None | None |
| Hot reload in dev | HMR via bundler | WebviewWindow reload | Worker teardown | Deno `--watch` |
| Binary distribution | N/A | Tauri bundler | Single Rust binary | Requires Deno runtime |

---

## What the plugin author sees

The plugin author writes TypeScript against the unode interface. The build toolchain for their plugin package handles producing the correct artifacts for each environment. The SDK package (`@unode/plugin-sdk` or equivalent) ships:

- The `ui.*` DSL builders
- The `PluginContext` TypeScript types
- The `PluginDefinition` interface
- For WASM builds: the `WasmPluginContextAdapter` compiled in

The plugin author does not import adapters directly. The adapter is injected by the runtime when the plugin is instantiated. From the plugin's perspective, `ctx.api.catalog.getWork(id)` is always just an async function call.

---

## Invariants that hold across all environments

These properties must be true regardless of which environment is running:

**1. The AST is always immutable.**
`CanonicalScreen` is frozen before leaving `render()` in all environments. The renderer never mutates AST nodes.

**2. Permission checks always happen on the host side.**
The plugin never checks its own permissions. The adapter never skips checks. The host-side handler (postMessage handler, Tauri command, WASM host function) is always the enforcement point.

**3. The `PluginContext` interface is identical.**
Plugin code that compiles and runs correctly in one environment will compile and run correctly in all environments. No environment-specific imports in plugin source.

**4. Actions are always symbolic.**
`ActionRef` contains a string type and a plain params object in all environments. No function references cross environment boundaries.

**5. State is always serializable at the boundary.**
Even where StateStore lives in JavaScript (web, Tauri), values written via `ctx.state.set()` must be primitives or arrays of primitives. Complex objects are never stored in the reactive store directly.

---

## Open questions

**Locale updates in Rust TUI.**
In web and Tauri, locale changes are pushed to the plugin iframe via postMessage. In Rust TUI, locale changes require a signal from the Rust host into the WASM module. The mechanism (a dedicated WASM host function `on_locale_change`) needs to be formalized. In Deno TUI, locale updates can follow the same postMessage pattern as web — simpler.

**Plugin-to-plugin events in Rust TUI.**
The `ctx.events` bus in web, Tauri, and Deno routes messages through the host JavaScript event system. In Rust TUI, events must be routed through Rust host functions. The semantics are the same; the routing layer is different and needs a concrete implementation contract.

**Deno TUI rendering library selection.**
Ink is the pragmatic first choice but adds a React dependency to the TUI host. The alternative is raw terminal control via ansi-escapes, which removes the dependency but requires implementing layout from scratch. This decision does not affect unode contracts — it is purely a renderer implementation choice.

**Choosing between Deno TUI and Rust TUI.**
Both are valid implementations of the same unode contracts. If the project starts with Deno TUI for speed of iteration, the migration path to Rust TUI is: replace the Worker-based `PluginHost` with a Wasmtime-based one, add a WASM build target to the plugin SDK, and rewrite the rendering layer in Rust. The plugin source code does not change.
