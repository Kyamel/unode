use std::cell::Cell;

use serde::de::DeserializeOwned;
use serde_json::{json, Value as JsonValue};
use unode_sdk::export_allocators;
use unode_sdk::prelude::{
    self as ui, create_route_tabs_meta, decode_json_bytes, encode_json_bytes,
    ActionIntent, ActionRef, ActionType, CoreActionType, IntoNode, PluginDispatchOutcome,
    PluginDispatchRequest, PluginDispatchResponse, PluginLoadRequest, PluginManifestEnvelope,
    PluginRenderRequest, ScreenNode, ScreenRouteTab, TextRole, Tone, UNODE_PLUGIN_ABI_VERSION,
    with_route_tabs,
};

#[cfg(test)]
use unode_sdk::prelude::{StringOrExpr, UiNode};

const PLUGIN_ID: &str = "dev.mugens.sanity-check";
const PLUGIN_NAME: &str = "Sanity Check";
const ROUTE_PATH: &str = "/plugins/sanity-check";

static ABI_VERSION_BYTES: &[u8] = b"0.1.0\0";

thread_local! {
    static MANIFEST_BUFFER_LEN: Cell<u32> = const { Cell::new(0) };
    static LOAD_BUFFER_LEN: Cell<u32> = const { Cell::new(0) };
    static RENDER_BUFFER_LEN: Cell<u32> = const { Cell::new(0) };
    static DISPATCH_BUFFER_LEN: Cell<u32> = const { Cell::new(0) };
}

export_allocators!();

fn with_output_buffer<F>(len_cell: &'static std::thread::LocalKey<Cell<u32>>, build: F) -> u32
where
    F: FnOnce() -> Vec<u8>,
{
    len_cell.with(|slot| {
        let bytes = build();
        let len = bytes.len() as u32;
        let ptr = unode_alloc(bytes.len()) as u32;
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
        }
        slot.set(len);
        ptr
    })
}

fn output_len(len_cell: &'static std::thread::LocalKey<Cell<u32>>) -> u32 {
    len_cell.with(Cell::get)
}

fn decode_guest_json<T: DeserializeOwned>(ptr: u32, len: u32) -> T {
    let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
    decode_json_bytes(bytes).expect("guest request must be valid ABI JSON")
}

fn manifest_envelope() -> PluginManifestEnvelope {
    PluginManifestEnvelope {
        abi_version: UNODE_PLUGIN_ABI_VERSION.to_string(),
        manifest: unode_sdk::plugin_manifest(PLUGIN_ID, PLUGIN_NAME)
            .version("0.1.0")
            .description("Runtime-loaded WASM sanity-check plugin for the MGN TUI shell.")
            .author("Codex")
            .build(),
    }
}

fn route_tabs() -> Vec<ScreenRouteTab> {
    vec![
        ScreenRouteTab {
            id: "overview".to_string(),
            label: "Overview".to_string(),
            to: ROUTE_PATH.to_string(),
            badge: Some("wasm".to_string()),
        },
        ScreenRouteTab {
            id: "inspect".to_string(),
            label: "Inspect".to_string(),
            to: format!("{ROUTE_PATH}?view=inspect"),
            badge: None,
        },
    ]
}

fn render_screen(request: &PluginRenderRequest) -> ScreenNode {
    let locale = request.locale.as_deref().unwrap_or("en");
    let active_tab = request
        .route
        .query
        .get("view")
        .map(String::as_str)
        .unwrap_or("overview");
    let title = request
        .data
        .get("title")
        .and_then(JsonValue::as_str)
        .unwrap_or("Runtime-loaded plugin");
    let host_message = request
        .data
        .get("hostMessage")
        .and_then(JsonValue::as_str)
        .unwrap_or("Host data not provided.");

    let params_summary = if request.route.params.is_empty() {
        "none".to_string()
    } else {
        request
            .route
            .params
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let screen = ui::screen()
        .id("sanity-check.screen")
        .title(format!("{PLUGIN_NAME} - {title}"))
        .subtitle(format!("route={} locale={locale}", request.route.pattern))
        .children([
            ui::text("This screen is rendered from a Rust plugin compiled to WebAssembly.".to_string())
                .role(TextRole::Body)
                .tone(Tone::Info)
                .into_node(),
            ui::text(format!("Host message: {host_message}"))
                .role(TextRole::Body)
                .into_node(),
            ui::text(format!("Route params: {params_summary}"))
                .role(TextRole::Caption)
                .tone(Tone::Muted)
                .into_node(),
            ui::stack()
                .id("sanity-check.details")
                .children([
                    ui::text("The plugin is isolated behind the unode WASM ABI.".to_string())
                        .role(TextRole::Body)
                        .into_node(),
                    ui::text("The TUI runtime is responsible for sandboxing and host calls.".to_string())
                        .role(TextRole::Body)
                        .into_node(),
                ])
                .into_node(),
            ui::actions()
                .id("sanity-check.actions")
                .children([
                    ui::action(
                        "Refresh screen".to_string(),
                        ActionRef {
                            r#type: ActionType::Custom("sanity.refresh".to_string()),
                            params: None,
                            confirm: None,
                        },
                    )
                    .id("sanity-check.refresh")
                    .intent(ActionIntent::Primary),
                    ui::action(
                        "Open inspect tab".to_string(),
                        ActionRef {
                            r#type: ActionType::Core(CoreActionType::Navigate),
                            params: Some(std::collections::BTreeMap::from([(
                                "to".to_string(),
                                json!(format!("{ROUTE_PATH}?view=inspect")),
                            )])),
                            confirm: None,
                        },
                    )
                    .id("sanity-check.inspect")
                    .intent(ActionIntent::Secondary),
                    ui::action(
                        "Go home via plugin dispatch".to_string(),
                        ActionRef {
                            r#type: ActionType::Custom("sanity.go-home".to_string()),
                            params: None,
                            confirm: None,
                        },
                    )
                    .id("sanity-check.go-home")
                    .intent(ActionIntent::Ghost),
                ])
                .into_node(),
        ])
        .initial_focus("sanity-check.refresh")
        .build();

    with_route_tabs(
        screen,
        create_route_tabs_meta(active_tab, route_tabs())
            .swipe_enabled(true)
            .swipe_threshold(48.0),
    )
}

fn json_or_error<T: serde::Serialize>(value: &T) -> Vec<u8> {
    encode_json_bytes(value)
        .unwrap_or_else(|err| serde_json::to_vec(&json!({ "error": err.to_string() })).expect("fallback json"))
}

fn encode_action_response(request: &PluginDispatchRequest) -> PluginDispatchResponse {
    match &request.action.r#type {
        ActionType::Custom(action) if action == "sanity.refresh" => PluginDispatchResponse {
            handled: true,
            outcome: PluginDispatchOutcome::RefreshCurrentScreen,
            message: Some(format!("Plugin requested refresh for {}", request.route.pattern)),
            data: None,
        },
        ActionType::Custom(action) if action == "sanity.go-home" => PluginDispatchResponse {
            handled: true,
            outcome: PluginDispatchOutcome::Navigate {
                to: "/home".to_string(),
            },
            message: Some("Plugin requested navigation to /home".to_string()),
            data: None,
        },
        _ => PluginDispatchResponse {
            handled: false,
            outcome: PluginDispatchOutcome::None,
            message: Some(format!(
                "Plugin ignored action on {}",
                request.route.pattern
            )),
            data: Some(json!({
                "pluginId": PLUGIN_ID,
                "actionType": request.action.r#type,
            })),
        },
    }
}

#[cfg(test)]
fn flatten_lines(screen: &ScreenNode) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(title) = &screen.title {
        lines.push(format!("title: {}", render_text_value(title)));
    }
    if let Some(subtitle) = &screen.subtitle {
        lines.push(format!("subtitle: {}", render_text_value(subtitle)));
    }
    for child in &screen.children {
        collect_node_lines(child, 0, &mut lines);
    }
    lines
}

#[cfg(test)]
fn collect_node_lines(node: &UiNode, depth: usize, lines: &mut Vec<String>) {
    let prefix = "  ".repeat(depth);
    match node {
        UiNode::Text(text) => lines.push(format!("{prefix}- {}", render_text_value(&text.content))),
        UiNode::Stack(stack) => {
            lines.push(format!("{prefix}[stack]"));
            for child in &stack.children {
                collect_node_lines(child, depth + 1, lines);
            }
        }
        UiNode::Section(section) => {
            let title = section
                .title
                .as_ref()
                .map(render_text_value)
                .unwrap_or_else(|| "section".to_string());
            lines.push(format!("{prefix}[section] {title}"));
            for child in &section.children {
                collect_node_lines(child, depth + 1, lines);
            }
        }
        other => lines.push(format!("{prefix}[node] {}", serde_json::to_string(other).unwrap_or_default())),
    }
}

#[cfg(test)]
fn render_text_value(value: &StringOrExpr) -> String {
    match value {
        StringOrExpr::Value(text) => text.clone(),
        StringOrExpr::Expr(expr) => serde_json::to_string(expr).unwrap_or_else(|_| "<expr>".to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_abi_version() -> *const u8 {
    ABI_VERSION_BYTES.as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_manifest() -> u32 {
    with_output_buffer(&MANIFEST_BUFFER_LEN, || json_or_error(&manifest_envelope()))
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_manifest_len() -> u32 {
    output_len(&MANIFEST_BUFFER_LEN)
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_load(request_ptr: u32, request_len: u32) -> u32 {
    let request = decode_guest_json::<PluginLoadRequest>(request_ptr, request_len);
    with_output_buffer(&LOAD_BUFFER_LEN, || {
        json_or_error(&json!({
            "loaded": true,
            "pluginId": PLUGIN_ID,
            "route": request.route.pattern,
            "locale": request.locale,
        }))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_load_result_len() -> u32 {
    output_len(&LOAD_BUFFER_LEN)
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_render(request_ptr: u32, request_len: u32) -> u32 {
    let request = decode_guest_json::<PluginRenderRequest>(request_ptr, request_len);
    with_output_buffer(&RENDER_BUFFER_LEN, || json_or_error(&render_screen(&request)))
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_render_result_len() -> u32 {
    output_len(&RENDER_BUFFER_LEN)
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_dispatch(request_ptr: u32, request_len: u32) -> u32 {
    let request = decode_guest_json::<PluginDispatchRequest>(request_ptr, request_len);
    with_output_buffer(&DISPATCH_BUFFER_LEN, || json_or_error(&encode_action_response(&request)))
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_dispatch_result_len() -> u32 {
    output_len(&DISPATCH_BUFFER_LEN)
}

#[cfg(test)]
mod tests {
    use super::{flatten_lines, manifest_envelope, render_screen, ROUTE_PATH};
    use serde_json::json;
    use std::collections::BTreeMap;
    use unode_sdk::PluginRenderRequest;
    use unode_sdk::prelude::ResolvedRoute;

    #[test]
    fn manifest_has_expected_plugin_identity() {
        let manifest = manifest_envelope();
        assert_eq!(manifest.manifest.id, "dev.mugens.sanity-check");
        assert_eq!(manifest.manifest.name, "Sanity Check");
    }

    #[test]
    fn render_screen_contains_sanity_lines() {
        let screen = render_screen(&PluginRenderRequest {
            route: ResolvedRoute {
                pattern: ROUTE_PATH.to_string(),
                params: BTreeMap::new(),
                query: BTreeMap::new(),
            },
            data: json!({
                "title": "Smoke test",
                "hostMessage": "Loaded through Wasmtime"
            }),
            state_snapshot: BTreeMap::new(),
            locale: Some("pt-BR".to_string()),
        });

        let lines = flatten_lines(&screen);
        assert!(lines.iter().any(|line| line.contains("Loaded through Wasmtime")));
        assert!(lines.iter().any(|line| line.contains("Rust plugin compiled to WebAssembly")));
    }
}
