//! Host-side plugin state: a tiny store fed by `state.set` host calls, plus
//! resolution of `binding` expressions in a rendered screen against the
//! current snapshot. Shared by TUI hosts (the playground shell, examples).

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde_json::Value as JsonValue;
use unode::core::ast::{
    ActionNode, BoolOrExpr, CollectionContinuation, OneOrExpr, PrimitiveOrExpr, ScreenNode,
    StringOrExpr, UiExpr, UiNode,
};

#[derive(Debug, Clone, Default)]
pub struct PluginState {
    values: Arc<Mutex<BTreeMap<String, JsonValue>>>,
}

impl PluginState {
    pub fn snapshot(&self) -> BTreeMap<String, JsonValue> {
        self.values.lock().expect("plugin state lock").clone()
    }

    /// Writes one state path (hosts call this from their `state.set` handler).
    pub fn set(&self, path: String, value: JsonValue) {
        self.values
            .lock()
            .expect("plugin state lock")
            .insert(path, value);
    }

    pub fn seed_missing(&self, initial_state: Option<&BTreeMap<String, JsonValue>>) {
        let Some(initial_state) = initial_state else {
            return;
        };
        let mut values = self.values.lock().expect("plugin state lock");
        for (path, value) in initial_state {
            values.entry(path.clone()).or_insert_with(|| value.clone());
        }
    }
}

pub fn resolve_screen_state(screen: &mut ScreenNode, state: &PluginState) {
    state.seed_missing(screen.initial_state.as_ref());
    let snapshot = state.snapshot();
    resolve_optional_string(&mut screen.title, &snapshot);
    resolve_optional_string(&mut screen.subtitle, &snapshot);
    for child in &mut screen.children {
        resolve_node(child, &snapshot);
    }
}

fn resolve_node(node: &mut UiNode, state: &BTreeMap<String, JsonValue>) {
    match node {
        UiNode::Section(node) => {
            resolve_optional_string(&mut node.title, state);
            resolve_optional_string(&mut node.description, state);
            resolve_nodes(&mut node.children, state);
        }
        UiNode::Stack(node) => resolve_nodes(&mut node.children, state),
        UiNode::Inline(node) => resolve_nodes(&mut node.children, state),
        UiNode::Grid(node) => {
            resolve_continuation(&mut node.continuation, state);
            resolve_nodes(&mut node.children, state);
        }
        UiNode::Scroll(node) => resolve_nodes(&mut node.children, state),
        UiNode::Text(node) => resolve_string(&mut node.content, state),
        UiNode::Value(node) => resolve_primitive(&mut node.value, state),
        UiNode::Badge(node) => resolve_string(&mut node.label, state),
        UiNode::Divider(node) => resolve_optional_string(&mut node.label, state),
        UiNode::Pressable(node) => {
            resolve_optional_string(&mut node.label, state);
            resolve_node(&mut node.child, state);
        }
        UiNode::Item(node) => {
            resolve_nodes(&mut node.leading, state);
            resolve_nodes(&mut node.primary, state);
            resolve_nodes(&mut node.secondary, state);
            resolve_nodes(&mut node.trailing, state);
        }
        UiNode::List(node) => {
            resolve_continuation(&mut node.continuation, state);
            for item in &mut node.items {
                resolve_nodes(&mut item.leading, state);
                resolve_nodes(&mut item.primary, state);
                resolve_nodes(&mut item.secondary, state);
                resolve_nodes(&mut item.trailing, state);
            }
        }
        UiNode::Action(node) => resolve_action(node, state),
        UiNode::Actions(node) => {
            for action in &mut node.children {
                resolve_action(action, state);
            }
        }
        UiNode::Disclosure(node) => {
            resolve_string(&mut node.label, state);
            resolve_optional_string(&mut node.label_expanded, state);
            resolve_nodes(&mut node.children, state);
        }
        UiNode::Menu(node) => {
            resolve_string(&mut node.label, state);
            for item in &mut node.items {
                resolve_string(&mut item.label, state);
                resolve_optional_bool(&mut item.disabled, state);
            }
        }
        UiNode::Input(node) => {
            resolve_string(&mut node.label, state);
            resolve_optional_primitive(&mut node.value, state);
            resolve_optional_string(&mut node.placeholder, state);
            resolve_optional_string(&mut node.help_text, state);
            resolve_optional_bool(&mut node.disabled, state);
        }
        UiNode::Form(node) => resolve_nodes(&mut node.children, state),
        UiNode::Status(node) => {
            resolve_optional_string(&mut node.title, state);
            resolve_string(&mut node.message, state);
            for action in &mut node.actions {
                resolve_action(action, state);
            }
        }
        UiNode::Empty(node) => {
            resolve_string(&mut node.title, state);
            resolve_optional_string(&mut node.message, state);
            for action in &mut node.actions {
                resolve_action(action, state);
            }
        }
        UiNode::Loading(node) => resolve_optional_string(&mut node.label, state),
        UiNode::Conditional(node) => {
            resolve_bool(&mut node.condition, state);
            resolve_node(&mut node.r#then, state);
            if let Some(otherwise) = &mut node.r#else {
                resolve_node(otherwise, state);
            }
        }
        UiNode::Slot(node) => {
            if let Some(fallback) = &mut node.fallback {
                resolve_node(fallback, state);
            }
        }
        UiNode::Icon(_) | UiNode::Media(_) => {}
    }
}

fn resolve_nodes(nodes: &mut [UiNode], state: &BTreeMap<String, JsonValue>) {
    for node in nodes {
        resolve_node(node, state);
    }
}

fn resolve_action(node: &mut ActionNode, state: &BTreeMap<String, JsonValue>) {
    resolve_string(&mut node.label, state);
    resolve_optional_bool(&mut node.disabled, state);
}

fn resolve_continuation(
    continuation: &mut Option<CollectionContinuation>,
    state: &BTreeMap<String, JsonValue>,
) {
    match continuation {
        Some(CollectionContinuation::Incremental(continuation)) => {
            resolve_optional_string(&mut continuation.label, state);
        }
        Some(CollectionContinuation::Remote(continuation)) => {
            resolve_optional_string(&mut continuation.label, state);
            resolve_optional_string(&mut continuation.loading_label, state);
        }
        None => {}
    }
}

fn resolve_optional_string(value: &mut Option<StringOrExpr>, state: &BTreeMap<String, JsonValue>) {
    if let Some(value) = value {
        resolve_string(value, state);
    }
}

fn resolve_string(value: &mut StringOrExpr, state: &BTreeMap<String, JsonValue>) {
    let OneOrExpr::Expr(UiExpr::Binding { path }) = value else {
        return;
    };
    if let Some(next) = state.get(path).map(json_to_string) {
        *value = OneOrExpr::Value(next);
    }
}

fn resolve_optional_bool(value: &mut Option<BoolOrExpr>, state: &BTreeMap<String, JsonValue>) {
    let Some(value) = value else {
        return;
    };
    resolve_bool(value, state);
}

fn resolve_bool(value: &mut BoolOrExpr, state: &BTreeMap<String, JsonValue>) {
    let OneOrExpr::Expr(UiExpr::Binding { path }) = value else {
        return;
    };
    if let Some(next) = state.get(path).and_then(JsonValue::as_bool) {
        *value = OneOrExpr::Value(next);
    }
}

fn resolve_optional_primitive(
    value: &mut Option<PrimitiveOrExpr>,
    state: &BTreeMap<String, JsonValue>,
) {
    if let Some(value) = value {
        resolve_primitive(value, state);
    }
}

fn resolve_primitive(value: &mut PrimitiveOrExpr, state: &BTreeMap<String, JsonValue>) {
    let OneOrExpr::Expr(UiExpr::Binding { path }) = value else {
        return;
    };
    if let Some(next) = state.get(path) {
        *value = OneOrExpr::Value(Some(next.clone()));
    }
}

fn json_to_string(value: &JsonValue) -> String {
    value
        .as_str()
        .map(ToString::to_string)
        .unwrap_or_else(|| value.to_string())
}
