use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::{Value as JsonValue, json};
use unode::core::ir::lower_screen;
use unode::core::normalize::normalize_screen;
use unode_sdk::prelude::{ResolvedRoute, ScreenNode};
use unode_sdk::{PluginLoadRequest, PluginRenderRequest};
use unode_tui_runtime::{TuiHostCallDispatcher, WasmtimeGuest};

struct PluginSpec {
    slug: &'static str,
    dir: &'static str,
    wasm_file: &'static str,
}

const PLUGINS: &[PluginSpec] = &[
    PluginSpec {
        slug: "sanity-check",
        dir: "plugins/sanity-check",
        wasm_file: "sanity_check_plugin.wasm",
    },
    PluginSpec {
        slug: "web-counter",
        dir: "plugins/web-counter",
        wasm_file: "web_counter_plugin.wasm",
    },
];

#[derive(Debug)]
struct Args {
    plugin: String,
    only: Option<String>,
    route: Option<String>,
    state: BTreeMap<String, JsonValue>,
}

fn main() -> Result<()> {
    let args = parse_args()?;
    let root = workspace_root();
    let spec = find_plugin(&args.plugin)?;
    let plugin_root = root.join(spec.dir);
    let wasm_path = find_plugin_wasm(&plugin_root, spec.wasm_file)
        .with_context(|| format!("{} has no built wasm artifact; run ./build.sh", spec.slug))?;

    let mut bridge = WasmtimeGuest::from_wasm_file(&wasm_path, dump_dispatcher())
        .with_context(|| format!("failed to instantiate {}", wasm_path.display()))?;
    let manifest = bridge.call_plugin_manifest().context("plugin_manifest")?;
    let route = args
        .route
        .clone()
        .unwrap_or_else(|| format!("/plugins/{}", spec.slug));
    let resolved_route = ResolvedRoute {
        pattern: route,
        params: BTreeMap::new(),
        query: BTreeMap::new(),
    };

    let load_request = PluginLoadRequest {
        route: resolved_route.clone(),
        state_snapshot: args.state.clone(),
        locale: Some("en".to_string()),
    };
    let load_response = bridge
        .call_plugin_load::<_, JsonValue>(&load_request)
        .context("plugin_load")?;

    let render_request = PluginRenderRequest {
        route: resolved_route,
        data: json!({
            "title": "Dump",
            "hostMessage": format!("Loaded from {}", wasm_path.display()),
        }),
        state_snapshot: args.state.clone(),
        locale: Some("en".to_string()),
    };
    let raw_ast = bridge
        .call_plugin_render::<_, ScreenNode>(&render_request)
        .context("plugin_render")?;
    let canonical_ast = normalize_screen(raw_ast.clone())
        .map_err(|err| anyhow::anyhow!("normalize_screen: {err}"))?;
    let ir = lower_screen(&canonical_ast);

    print_section(&args, "manifest", &manifest)?;
    print_section(&args, "load-request", &load_request)?;
    print_section(&args, "load-response", &load_response)?;
    print_section(&args, "render-request", &render_request)?;
    print_section(&args, "raw-ast", &raw_ast)?;
    print_section(&args, "canonical-ast", &canonical_ast)?;
    print_section(&args, "ir", &ir)?;

    Ok(())
}

fn parse_args() -> Result<Args> {
    let mut positional = Vec::new();
    let mut only = None;
    let mut route = None;
    let mut state = BTreeMap::new();
    let mut iter = std::env::args().skip(1);

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--only" => {
                only = Some(iter.next().context("--only requires a section name")?);
            }
            "--route" => {
                route = Some(iter.next().context("--route requires a path")?);
            }
            "--state" => {
                let pair = iter.next().context("--state requires key=json")?;
                let (key, value) = pair
                    .split_once('=')
                    .with_context(|| format!("state entry must be key=json, got `{pair}`"))?;
                let value = serde_json::from_str(value)
                    .unwrap_or_else(|_| JsonValue::String(value.to_string()));
                state.insert(key.to_string(), value);
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other if other.starts_with('-') => bail!("unknown option `{other}`"),
            other => positional.push(other.to_string()),
        }
    }

    let plugin = positional
        .into_iter()
        .next()
        .unwrap_or_else(|| "web-counter".to_string());

    Ok(Args {
        plugin,
        only,
        route,
        state,
    })
}

fn print_help() {
    println!(
        "Usage: cargo run -p unode-devtools --bin dump-plugin -- [plugin] [--only section] [--route path] [--state key=json]\n\
\n\
Plugins: web-counter, sanity-check\n\
Sections: manifest, load-request, load-response, render-request, raw-ast, canonical-ast, ir\n\
\n\
Examples:\n\
  cargo run -p unode-devtools --bin dump-plugin -- web-counter\n\
  cargo run -p unode-devtools --bin dump-plugin -- web-counter --only ir\n\
  cargo run -p unode-devtools --bin dump-plugin -- web-counter --state ui.count=3 --state ui.countLabel='\"Count: 3\"'"
    );
}

fn find_plugin(name: &str) -> Result<&'static PluginSpec> {
    let normalized = name
        .strip_prefix("plugins/")
        .unwrap_or(name)
        .trim_matches('/');

    PLUGINS
        .iter()
        .find(|plugin| plugin.slug == normalized || plugin.dir == normalized)
        .with_context(|| format!("unknown plugin `{name}`"))
}

fn find_plugin_wasm(plugin_root: &Path, wasm_file: &str) -> Option<PathBuf> {
    [
        plugin_root
            .join("target/wasm32-unknown-unknown/debug")
            .join(wasm_file),
        plugin_root
            .join("target/wasm32-unknown-unknown/release")
            .join(wasm_file),
    ]
    .into_iter()
    .filter_map(|path| {
        let modified = fs::metadata(&path).ok()?.modified().ok()?;
        Some((modified, path))
    })
    .max_by_key(|(modified, _)| *modified)
    .map(|(_, path)| path)
}

fn dump_dispatcher() -> TuiHostCallDispatcher {
    let mut dispatcher = TuiHostCallDispatcher::new();
    dispatcher.register("system.ping", |_| Ok(json!({ "pong": true })));
    dispatcher.register("navigation.navigate", |params| {
        Ok(json!({
            "ok": true,
            "to": params.get("to").cloned().unwrap_or(JsonValue::Null)
        }))
    });
    dispatcher.register("state.set", |params| {
        Ok(json!({
            "ok": true,
            "recorded": {
                "path": params.get("path").cloned().unwrap_or(JsonValue::Null),
                "value": params.get("value").cloned().unwrap_or(JsonValue::Null)
            }
        }))
    });
    dispatcher
}

fn print_section<T: Serialize>(args: &Args, name: &str, value: &T) -> Result<()> {
    if args.only.as_deref().is_some_and(|only| only != name) {
        return Ok(());
    }

    println!("--- {name} ---");
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
