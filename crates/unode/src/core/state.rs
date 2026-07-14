use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use serde_json::{Map as JsonMap, Value as JsonValue};

use crate::core::ast::Primitive;

pub type SubscriptionId = u64;
pub type StateListener = Arc<dyn Fn(Primitive, &str) + Send + Sync + 'static>;

pub trait StateStore {
    fn get(&self, path: &str) -> Option<&JsonValue>;
    fn get_primitive(&self, path: &str, fallback: Primitive) -> Primitive;
    fn set(&mut self, path: &str, value: JsonValue);
    fn merge_data(&mut self, data: BTreeMap<String, JsonValue>);
    fn subscribe(&mut self, path: &str, listener: StateListener) -> SubscriptionId;
    fn subscribe_prefix(&mut self, prefix: &str, listener: StateListener) -> SubscriptionId;
    fn unsubscribe(&mut self, id: SubscriptionId) -> bool;
    fn snapshot(&self) -> BTreeMap<String, JsonValue>;
    fn reset(&mut self);
}

#[derive(Default)]
pub struct MemoryStateStore {
    initial_seed: BTreeMap<String, JsonValue>,
    data: JsonValue,
    exact_listeners: BTreeMap<String, BTreeMap<SubscriptionId, StateListener>>,
    prefix_listeners: BTreeMap<String, BTreeMap<SubscriptionId, StateListener>>,
    batch_depth: usize,
    pending_paths: BTreeSet<String>,
    next_subscription_id: SubscriptionId,
}

impl std::fmt::Debug for MemoryStateStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryStateStore")
            .field("initial_seed", &self.initial_seed)
            .field("data", &self.data)
            .field("exact_listener_paths", &self.exact_listeners.keys().collect::<Vec<_>>())
            .field("prefix_listener_paths", &self.prefix_listeners.keys().collect::<Vec<_>>())
            .field("batch_depth", &self.batch_depth)
            .field("pending_paths", &self.pending_paths)
            .finish()
    }
}

impl MemoryStateStore {
    /// Creates a host-owned screen state store.
    ///
    /// The optional seed is a flat map of dot-separated paths. For example,
    /// `{ "ui.count": 1 }` is expanded internally so `get("ui.count")` works.
    /// The seed is retained as the reset baseline for the current screen.
    pub fn new(seed: Option<BTreeMap<String, JsonValue>>) -> Self {
        let initial_seed = expand_flat_object(seed.unwrap_or_default());
        let data = JsonValue::Object(to_json_map(&initial_seed));

        Self {
            initial_seed,
            data,
            exact_listeners: BTreeMap::new(),
            prefix_listeners: BTreeMap::new(),
            batch_depth: 0,
            pending_paths: BTreeSet::new(),
            next_subscription_id: 1,
        }
    }

    /// Runs several writes as one notification cycle.
    ///
    /// Use this on the host side after draining multiple plugin host calls. The
    /// store queues changed paths while the closure runs, then flushes listeners
    /// once at the end.
    pub fn batch(&mut self, f: impl FnOnce(&mut Self)) {
        self.batch_depth += 1;
        f(self);
        self.batch_depth -= 1;

        if self.batch_depth == 0 {
            self.flush();
        }
    }

    fn queue_notify(&mut self, path: &str) {
        self.pending_paths.insert(path.to_string());
        if self.batch_depth == 0 {
            self.flush();
        }
    }

    fn flush(&mut self) {
        if self.batch_depth > 0 || self.pending_paths.is_empty() {
            return;
        }

        let pending: Vec<String> = self.pending_paths.iter().cloned().collect();
        self.pending_paths.clear();

        for path in pending {
            let value = json_to_primitive(self.get(&path).cloned());

            if let Some(listeners) = self.exact_listeners.get(&path) {
                for listener in listeners.values() {
                    listener(value.clone(), &path);
                }
            }

            for (prefix, listeners) in &self.prefix_listeners {
                if prefix.is_empty() || path == *prefix || path.starts_with(&format!("{prefix}.")) {
                    for listener in listeners.values() {
                        listener(value.clone(), &path);
                    }
                }
            }
        }
    }

    fn next_subscription_id(&mut self) -> SubscriptionId {
        let id = self.next_subscription_id;
        self.next_subscription_id += 1;
        id
    }
}

impl StateStore for MemoryStateStore {
    /// Reads a JSON value by dot-separated path.
    ///
    /// Array indices are supported as numeric path segments, for example
    /// `items.0.title`.
    fn get(&self, path: &str) -> Option<&JsonValue> {
        get_by_path(&self.data, path)
    }

    fn get_primitive(&self, path: &str, fallback: Primitive) -> Primitive {
        match self.get(path) {
            Some(JsonValue::Null) => None,
            Some(JsonValue::String(s)) => Some(JsonValue::String(s.clone())),
            Some(JsonValue::Number(n)) => Some(JsonValue::Number(n.clone())),
            Some(JsonValue::Bool(v)) => Some(JsonValue::Bool(*v)),
            _ => fallback,
        }
    }

    /// Writes a JSON value by dot-separated path and notifies subscribers.
    ///
    /// Missing intermediate objects or arrays are created as needed. Empty paths
    /// are ignored.
    fn set(&mut self, path: &str, value: JsonValue) {
        set_by_path(&mut self.data, path, value);
        self.queue_notify(path);
    }

    /// Merges a flat data map into the current state.
    ///
    /// This is used for plugin `load()` data and screen `initial_state`. Keys are
    /// expanded as paths before being merged into the nested JSON object.
    fn merge_data(&mut self, data: BTreeMap<String, JsonValue>) {
        let expanded = expand_flat_object(data);
        merge_into_object(&mut self.data, JsonValue::Object(to_json_map(&expanded)));

        for key in expanded.keys() {
            self.queue_notify(key);
        }
    }

    fn subscribe(&mut self, path: &str, listener: StateListener) -> SubscriptionId {
        let id = self.next_subscription_id();
        self.exact_listeners
            .entry(path.to_string())
            .or_default()
            .insert(id, listener);
        id
    }

    fn subscribe_prefix(&mut self, prefix: &str, listener: StateListener) -> SubscriptionId {
        let id = self.next_subscription_id();
        self.prefix_listeners
            .entry(prefix.to_string())
            .or_default()
            .insert(id, listener);
        id
    }

    fn unsubscribe(&mut self, id: SubscriptionId) -> bool {
        let mut removed = false;

        self.exact_listeners.retain(|_, listeners| {
            removed |= listeners.remove(&id).is_some();
            !listeners.is_empty()
        });

        self.prefix_listeners.retain(|_, listeners| {
            removed |= listeners.remove(&id).is_some();
            !listeners.is_empty()
        });

        removed
    }

    fn snapshot(&self) -> BTreeMap<String, JsonValue> {
        match &self.data {
            JsonValue::Object(map) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            _ => BTreeMap::new(),
        }
    }

    fn reset(&mut self) {
        self.data = JsonValue::Object(to_json_map(&self.initial_seed));

        let keys: Vec<String> = self.initial_seed.keys().cloned().collect();
        for key in keys {
            self.queue_notify(&key);
        }

        self.flush();
    }
}

fn split_path(path: &str) -> Vec<&str> {
    path.split('.').filter(|segment| !segment.is_empty()).collect()
}

fn get_by_path<'a>(root: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
    let mut current = root;

    for segment in split_path(path) {
        match current {
            JsonValue::Object(map) => {
                current = map.get(segment)?;
            }
            JsonValue::Array(items) => {
                let index = segment.parse::<usize>().ok()?;
                current = items.get(index)?;
            }
            _ => return None,
        }
    }

    Some(current)
}

fn set_by_path(root: &mut JsonValue, path: &str, value: JsonValue) {
    let segments = split_path(path);
    if segments.is_empty() {
        return;
    }

    let mut current = root;

    for (index, segment) in segments.iter().enumerate() {
        let is_last = index == segments.len() - 1;

        match current {
            JsonValue::Object(map) => {
                if is_last {
                    map.insert((*segment).to_string(), value);
                    return;
                }

                let next_is_index = segments[index + 1].parse::<usize>().is_ok();
                current = map.entry((*segment).to_string()).or_insert_with(|| {
                    if next_is_index {
                        JsonValue::Array(vec![])
                    } else {
                        JsonValue::Object(JsonMap::new())
                    }
                });
            }
            JsonValue::Array(items) => {
                let current_index = match segment.parse::<usize>() {
                    Ok(value) => value,
                    Err(_) => return,
                };

                if items.len() <= current_index {
                    items.resize_with(current_index + 1, || JsonValue::Null);
                }

                if is_last {
                    items[current_index] = value;
                    return;
                }

                let next_is_index = segments[index + 1].parse::<usize>().is_ok();
                if items[current_index].is_null() {
                    items[current_index] = if next_is_index {
                        JsonValue::Array(vec![])
                    } else {
                        JsonValue::Object(JsonMap::new())
                    };
                }

                current = &mut items[current_index];
            }
            _ => {
                *current = JsonValue::Object(JsonMap::new());
                if let JsonValue::Object(map) = current {
                    let next_is_index = !is_last && segments[index + 1].parse::<usize>().is_ok();
                    current = map.entry((*segment).to_string()).or_insert_with(|| {
                        if next_is_index {
                            JsonValue::Array(vec![])
                        } else {
                            JsonValue::Object(JsonMap::new())
                        }
                    });
                }
            }
        }
    }
}

fn expand_flat_object(data: BTreeMap<String, JsonValue>) -> BTreeMap<String, JsonValue> {
    let mut result = JsonValue::Object(JsonMap::new());

    for (key, value) in data {
        set_by_path(&mut result, &key, value);
    }

    match result {
        JsonValue::Object(map) => map.into_iter().collect(),
        _ => BTreeMap::new(),
    }
}

fn merge_into_object(target: &mut JsonValue, source: JsonValue) {
    match (target, source) {
        (JsonValue::Object(target_map), JsonValue::Object(source_map)) => {
            for (key, value) in source_map {
                match target_map.get_mut(&key) {
                    Some(existing) if existing.is_object() && value.is_object() => {
                        merge_into_object(existing, value);
                    }
                    _ => {
                        target_map.insert(key, value);
                    }
                }
            }
        }
        (target, source) => {
            *target = source;
        }
    }
}

fn to_json_map(data: &BTreeMap<String, JsonValue>) -> JsonMap<String, JsonValue> {
    data.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

fn json_to_primitive(value: Option<JsonValue>) -> Primitive {
    match value {
        Some(JsonValue::Null) | None => None,
        Some(other) => Some(other),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use serde_json::json;

    use super::{MemoryStateStore, StateStore};

    #[test]
    fn supports_nested_get_set_paths() {
        let mut store = MemoryStateStore::new(None);

        store.set("work.title", json!("Blue Box"));
        store.set("items.0.name", json!("foo"));

        assert_eq!(store.get("work.title"), Some(&json!("Blue Box")));
        assert_eq!(store.get("items.0.name"), Some(&json!("foo")));
    }

    #[test]
    fn merge_data_accepts_flat_keys() {
        let mut store = MemoryStateStore::new(None);

        store.merge_data(
            [
                ("work.title".to_string(), json!("Blue Box")),
                ("work.year".to_string(), json!(2021)),
            ]
            .into_iter()
            .collect(),
        );

        assert_eq!(store.get("work.title"), Some(&json!("Blue Box")));
        assert_eq!(store.get("work.year"), Some(&json!(2021)));
    }

    #[test]
    fn exact_and_prefix_subscriptions_fire() {
        let mut store = MemoryStateStore::new(None);
        let exact_hits = Arc::new(Mutex::new(Vec::new()));
        let prefix_hits = Arc::new(Mutex::new(Vec::new()));

        let exact_hits_clone = exact_hits.clone();
        store.subscribe(
            "work.title",
            Arc::new(move |value, path| {
                exact_hits_clone
                    .lock()
                    .unwrap()
                    .push((path.to_string(), value.clone()));
            }),
        );

        let prefix_hits_clone = prefix_hits.clone();
        store.subscribe_prefix(
            "work",
            Arc::new(move |value, path| {
                prefix_hits_clone
                    .lock()
                    .unwrap()
                    .push((path.to_string(), value.clone()));
            }),
        );

        store.set("work.title", json!("Blue Box"));

        assert_eq!(exact_hits.lock().unwrap().len(), 1);
        assert_eq!(prefix_hits.lock().unwrap().len(), 1);
    }

    #[test]
    fn batch_coalesces_notifications() {
        let mut store = MemoryStateStore::new(None);
        let hits = Arc::new(Mutex::new(Vec::new()));
        let hits_clone = hits.clone();

        store.subscribe_prefix(
            "",
            Arc::new(move |_, path| {
                hits_clone.lock().unwrap().push(path.to_string());
            }),
        );

        store.batch(|store| {
            store.set("a", json!(1));
            store.set("a", json!(2));
            store.set("b", json!(3));
        });

        let hits = hits.lock().unwrap().clone();
        assert_eq!(hits, vec!["a".to_string(), "b".to_string()]);
    }
}
