---
title: "Quickstart: a plugin"
description: Build the reactive web-counter plugin from scratch and understand the sandbox boundary.
---

This walkthrough builds `web-counter`, the reactive plugin behind the web
vertical slice. It renders one line bound to host state plus three buttons. The
key idea: **state never lives in the plugin's memory** — the host owns it and
hands it back on every dispatch. The plugin only declares intent and requests
writes.

## 1. Create the crate

A plugin is a `cdylib` that depends on `unode-sdk`:

```toml title="Cargo.toml"
[package]
name = "web-counter-plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
unode-sdk = { path = "../../crates/unode-sdk" }
```

## 2. Declare the manifest

The manifest identifies the plugin and its ABI version. `plugin_manifest` is a
builder from the SDK:

```rust title="src/lib.rs"
use unode_sdk::prelude::{
    self as ui, expr, ActionIntent, ActionRef, ActionType, IntoNode,
    PluginDispatchOutcome, PluginDispatchRequest, PluginDispatchResponse,
    PluginLoadRequest, PluginManifestEnvelope, PluginRenderRequest, ScreenNode,
    TextRole, Tone, UNODE_PLUGIN_ABI_VERSION,
};

const PLUGIN_ID: &str = "dev.unode.web-counter";
const PLUGIN_NAME: &str = "Web Counter";
const COUNT_PATH: &str = "ui.count";
const LABEL_PATH: &str = "ui.countLabel";

fn manifest_envelope() -> PluginManifestEnvelope {
    PluginManifestEnvelope {
        abi_version: UNODE_PLUGIN_ABI_VERSION.to_string(),
        manifest: unode_sdk::plugin_manifest(PLUGIN_ID, PLUGIN_NAME)
            .version("0.1.0")
            .description("Reactive counter proving the unode web runtime slice.")
            .author("unode")
            .build(),
    }
}
```

## 3. Handle `load`

`load` runs on navigation and returns data the host merges into the StateStore.
For the counter it just acknowledges the route:

```rust
use serde_json::{json, Value as JsonValue};

fn load_response(request: &PluginLoadRequest) -> JsonValue {
    json!({
        "loaded": true,
        "pluginId": PLUGIN_ID,
        "route": request.route.pattern,
    })
}
```

## 4. Render intent

`render` returns a `ScreenNode` built with the `ui::` DSL. Note the single
reactive node: its content is a **binding** to `ui.countLabel`, so the host
tracks it and patches only that line when the value changes.

```rust
fn label_for(count: i64) -> String {
    format!("Count: {count}")
}

/// A plain custom action with no params.
fn custom(action: &str) -> ActionRef {
    ActionRef {
        r#type: ActionType::Custom(action.to_string()),
        params: None,
        confirm: None,
    }
}

fn render_screen(_request: &PluginRenderRequest) -> ScreenNode {
    ui::screen()
        .id("web-counter.screen")
        .title(PLUGIN_NAME)
        .subtitle("Rendered from a Rust plugin compiled to WebAssembly.")
        .initial_state(std::collections::BTreeMap::from([
            (COUNT_PATH.to_string(), json!(0)),
            (LABEL_PATH.to_string(), json!(label_for(0))),
        ]))
        .children(ui::nodes![
            // The one reactive node. Its content is a binding, so the host
            // patches only this line when `ui.countLabel` changes.
            ui::text(expr::binding::<String>(LABEL_PATH))
                .id("web-counter.value")
                .role(TextRole::Title)
                .tone(Tone::Info),
            ui::text("The number above is host state; the buttons dispatch intents.")
                .id("web-counter.hint")
                .role(TextRole::Caption)
                .tone(Tone::Muted),
            ui::actions()
                .id("web-counter.actions")
                .children([
                    ui::action("Increment", custom("counter.inc"))
                        .id("web-counter.inc")
                        .intent(ActionIntent::Primary),
                    ui::action("Decrement", custom("counter.dec"))
                        .id("web-counter.dec")
                        .intent(ActionIntent::Secondary),
                    ui::action("Reset", custom("counter.reset"))
                        .id("web-counter.reset")
                        .intent(ActionIntent::Ghost),
                ])
                .into_node(),
        ])
        .initial_focus("web-counter.inc")
        .build()
}
```

## 5. Handle `dispatch`

When the user clicks a button, the host calls `dispatch` with the current state
snapshot. The plugin computes the next value and **requests state writes through
host calls** — it never returns UI state in its response. The host applies the
writes, which produces a single patch op re-rendering only the bound line.

```rust
fn current_count(request: &PluginDispatchRequest) -> i64 {
    request
        .state_snapshot
        .get(COUNT_PATH)
        .and_then(JsonValue::as_i64)
        .unwrap_or(0)
}

fn next_count(request: &PluginDispatchRequest) -> Option<i64> {
    match &request.action.r#type {
        ActionType::Custom(a) if a == "counter.inc" => Some(current_count(request) + 1),
        ActionType::Custom(a) if a == "counter.dec" => Some(current_count(request) - 1),
        ActionType::Custom(a) if a == "counter.reset" => Some(0),
        _ => None,
    }
}

fn dispatch_response(request: &PluginDispatchRequest) -> PluginDispatchResponse {
    match next_count(request) {
        Some(count) => {
            // Writes cross the sandbox boundary as capability calls.
            ui::host::state_set(COUNT_PATH, json!(count));
            ui::host::state_set(LABEL_PATH, json!(label_for(count)));
            PluginDispatchResponse {
                handled: true,
                outcome: PluginDispatchOutcome::None,
                message: Some(format!("count -> {count}")),
                data: None,
            }
        }
        None => PluginDispatchResponse {
            handled: false,
            outcome: PluginDispatchOutcome::None,
            message: Some("web-counter ignored action".to_string()),
            data: None,
        },
    }
}
```

## 6. Export the plugin

`export_plugin!` generates the raw C ABI exports (`plugin_manifest`,
`plugin_load`, `plugin_render`, `plugin_dispatch`) plus the allocator. You never
write those by hand.

```rust
unode_sdk::export_plugin! {
    manifest: manifest_envelope,
    load: load_response,
    render: render_screen,
    dispatch: dispatch_response,
}
```

## 7. Build it

```sh
cargo build --manifest-path plugins/web-counter/Cargo.toml \
  --target wasm32-unknown-unknown --release
```

The resulting `.wasm` is the same artifact both the web and TUI runtimes
consume.

## What just happened

- The plugin described a screen **once**. Its structure is fixed for the load
  cycle.
- One node is bound to `ui.countLabel`. The host tracked that dependency during
  normalization.
- A click dispatched an *intent*, not a state mutation. The plugin asked the
  host to write, and the host woke only the subscribers of the changed path.

That path — intent in, targeted patch out — is the whole model. Read
[Reactivity](/concepts/reactivity/) to see how the host tracks and patches, and
[Runtime & Lifecycle](/concepts/runtime/) for the full route cycle.
