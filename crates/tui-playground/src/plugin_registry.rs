use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::{Value as JsonValue, json};
use unode_plugin_sdk::prelude::PluginManifest;
use unode_runtime::{
    CommandResult, DeferredText, RegisteredCommand, RegisteredNavigationItem, RegisteredRoute,
};
use unode_tui_runtime::{CachedTuiPlugin, PluginState, TuiHostCallDispatcher, TuiRuntime};

pub struct LoadedPlugin {
    pub runtime_plugin: CachedTuiPlugin,
    /// Primary (navigation) route; exercised by the integration tests.
    #[allow(dead_code)]
    pub route: String,
    pub display_source: String,
    pub source_newer_than_wasm: bool,
    pub state: PluginState,
}

struct PluginSpec {
    dir: &'static str,
    wasm_file: &'static str,
    missing_message: &'static str,
    priority: i32,
}

const PLUGIN_SPECS: &[PluginSpec] = &[
    PluginSpec {
        dir: "plugins/sanity-check",
        wasm_file: "sanity_check_plugin.wasm",
        missing_message: "Sanity plugin not built yet. Run `cargo build --manifest-path plugins/sanity-check/Cargo.toml --target wasm32-unknown-unknown`.",
        priority: 410,
    },
    PluginSpec {
        dir: "plugins/counter",
        wasm_file: "web_counter_plugin.wasm",
        missing_message: "Web Counter plugin not built yet. Run `cargo build --manifest-path plugins/counter/Cargo.toml --target wasm32-unknown-unknown`.",
        priority: 400,
    },
];

pub fn load_runtime_plugins(
    runtime: &mut TuiRuntime<()>,
) -> Result<(Vec<LoadedPlugin>, Vec<String>)> {
    let mut messages = Vec::new();
    let mut plugins = Vec::new();

    for spec in PLUGIN_SPECS {
        let plugin_root = workspace_root().join(spec.dir);
        let Some(wasm_path) = find_plugin_wasm(&plugin_root, spec.wasm_file) else {
            messages.push(spec.missing_message.to_string());
            continue;
        };

        let source_newer_than_wasm = plugin_source_is_newer_than_wasm(&plugin_root, &wasm_path);
        let state = PluginState::default();
        let dispatcher = plugin_dispatcher(state.clone());

        let runtime_plugin = CachedTuiPlugin::from_wasm_file(&wasm_path, dispatcher)
            .with_context(|| format!("failed to instantiate plugin at {}", wasm_path.display()))?;
        let manifest = runtime_plugin.manifest().clone();
        let route = primary_route(&manifest.manifest);

        if manifest.manifest.routes.is_empty() {
            // Legacy plugins without declared routes keep the host-assigned
            // `/plugins/{slug}` route.
            runtime.inner.routes.register(RegisteredRoute {
                plugin_id: manifest.manifest.id.clone(),
                pattern: route.clone(),
                screen_kind: format!("{}.screen", manifest.manifest.id),
                priority: 500,
            });
        } else {
            runtime
                .inner
                .routes
                .register_manifest_routes(&manifest.manifest, 500);
        }
        runtime.inner.navigation.register(RegisteredNavigationItem {
            id: format!("nav.{}", manifest.manifest.id),
            plugin_id: manifest.manifest.id.clone(),
            label: DeferredText::from(manifest.manifest.name.clone()),
            short_label: None,
            to: route.clone(),
            icon: None,
            section: Some("plugins".to_string()),
            priority: spec.priority,
            when: None,
        });
        runtime.inner.commands.register(RegisteredCommand {
            id: format!("open.{}", manifest.manifest.id),
            plugin_id: manifest.manifest.id.clone(),
            title: DeferredText::from(format!("Open {}", manifest.manifest.name)),
            category: Some(DeferredText::from("Plugins")),
            keywords: vec!["plugin".to_string(), plugin_slug(&manifest.manifest.id)],
            when: None,
            run: {
                let route = route.clone();
                std::sync::Arc::new(move |_| CommandResult::Navigate(route.clone()))
            },
        });

        let display_source = display_path_for_ui(&wasm_path);
        messages.push(format!(
            "Loaded {} from {}",
            manifest.manifest.name, display_source
        ));
        if source_newer_than_wasm {
            messages.push(format!(
                "Warning: {} source changed after the wasm build. Rebuild the plugin inside `nix-shell`.",
                manifest.manifest.name
            ));
        }

        plugins.push(LoadedPlugin {
            runtime_plugin,
            route,
            display_source,
            source_newer_than_wasm,
            state,
        });
    }

    Ok((plugins, messages))
}

fn plugin_dispatcher(state: PluginState) -> TuiHostCallDispatcher {
    let mut dispatcher = TuiHostCallDispatcher::new();
    dispatcher.register("system.ping", |_| Ok(json!({ "pong": true })));
    dispatcher.register("navigation.navigate", |params| {
        Ok(json!({
            "ok": true,
            "to": params.get("to").cloned().unwrap_or(JsonValue::Null)
        }))
    });
    dispatcher.register("state.set", move |params| {
        let path = params
            .get("path")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| unode_tui_runtime::TuiHostCallError::Handler {
                operation: "state.set".to_string(),
                message: "missing string `path`".to_string(),
            })?;
        let value = params.get("value").cloned().unwrap_or(JsonValue::Null);
        state.set(path.to_string(), value);
        Ok(json!({ "ok": true }))
    });
    dispatcher
}

pub fn find_plugin_wasm(plugin_root: &Path, wasm_file: &str) -> Option<PathBuf> {
    let candidates = [
        plugin_root
            .join("target/wasm32-unknown-unknown/debug")
            .join(wasm_file),
        plugin_root
            .join("target/wasm32-unknown-unknown/release")
            .join(wasm_file),
    ];

    candidates
        .into_iter()
        .filter_map(|path| {
            let modified = fs::metadata(&path).ok()?.modified().ok()?;
            Some((modified, path))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

fn plugin_source_is_newer_than_wasm(plugin_root: &Path, wasm_path: &Path) -> bool {
    let Ok(wasm_modified) = fs::metadata(wasm_path).and_then(|metadata| metadata.modified()) else {
        return false;
    };

    latest_modified_in(plugin_root.join("src"))
        .into_iter()
        .chain(
            [
                plugin_root.join("Cargo.toml"),
                plugin_root.join("Cargo.lock"),
            ]
            .into_iter()
            .filter_map(|path| fs::metadata(path).ok())
            .filter_map(|metadata| metadata.modified().ok()),
        )
        .any(|modified| modified > wasm_modified)
}

fn latest_modified_in(path: PathBuf) -> Vec<std::time::SystemTime> {
    let Ok(metadata) = fs::metadata(&path) else {
        return vec![];
    };

    if metadata.is_file() {
        return metadata.modified().ok().into_iter().collect();
    }

    let Ok(entries) = fs::read_dir(path) else {
        return vec![];
    };

    entries
        .filter_map(|entry| entry.ok())
        .flat_map(|entry| latest_modified_in(entry.path()))
        .collect()
}

/// The route used for navigation entries: the first declared static pattern,
/// falling back to the host-assigned `/plugins/{slug}` route.
fn primary_route(manifest: &PluginManifest) -> String {
    manifest
        .routes
        .iter()
        .map(|route| route.pattern.as_str())
        .find(|pattern| !pattern.contains(':'))
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("/plugins/{}", plugin_slug(&manifest.id)))
}

pub fn plugin_slug(plugin_id: &str) -> String {
    plugin_id
        .split('.')
        .next_back()
        .unwrap_or(plugin_id)
        .to_string()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn display_path_for_ui(path: &Path) -> String {
    path.strip_prefix(workspace_root())
        .unwrap_or(path)
        .display()
        .to_string()
}
