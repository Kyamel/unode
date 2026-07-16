//! Typed state keys: `useState`-style ergonomics on top of the serializable
//! path-binding protocol. A [`StateKey`] names a state path once and carries
//! the value type, so reads, writes, and bindings all agree:
//!
//! ```ignore
//! const SHIP_COUNT: StateKey<u32> = StateKey::new("routeTabs.shipCount");
//!
//! // manifest / screen bindings (host resolves against the state store):
//! route("/ship").badge_bind(SHIP_COUNT.path());
//! ui::text(SHIP_COUNT.bind_text());
//!
//! // dispatch handler:
//! let next = SHIP_COUNT.get(&request.state_snapshot).unwrap_or(0) + 1;
//! SHIP_COUNT.set(next);
//! ```
//!
//! The wire format stays plain paths + JSON — `StateKey` is authoring sugar,
//! not protocol.

use std::collections::BTreeMap;
use std::marker::PhantomData;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;
use unode::core::ast::UiExpr;

use crate::host;

/// A typed handle to one state path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateKey<T> {
    path: &'static str,
    // `fn() -> T` keeps the key `Send + Sync + Copy` regardless of `T`.
    _marker: PhantomData<fn() -> T>,
}

impl<T> StateKey<T> {
    pub const fn new(path: &'static str) -> Self {
        Self {
            path,
            _marker: PhantomData,
        }
    }

    pub const fn path(&self) -> &'static str {
        self.path
    }

    /// Binding expression typed as the key's value — for fields of the same
    /// type (e.g. `StateKey<bool>` into a `BoolOrExpr`).
    pub fn bind(&self) -> UiExpr<T> {
        UiExpr::Binding {
            path: self.path.to_string(),
        }
    }

    /// Binding expression for display fields (`StringOrExpr` labels, badges,
    /// text). Hosts stringify non-string values when resolving.
    pub fn bind_text(&self) -> UiExpr<String> {
        UiExpr::Binding {
            path: self.path.to_string(),
        }
    }
}

impl<T: Serialize> StateKey<T> {
    /// Writes the value through the `state.set` host call.
    pub fn set(&self, value: T) {
        host::state_set(
            self.path,
            serde_json::to_value(value).expect("serialize state value"),
        );
    }
}

impl<T: DeserializeOwned> StateKey<T> {
    /// Reads the value from a render/dispatch state snapshot.
    pub fn get(&self, snapshot: &BTreeMap<String, JsonValue>) -> Option<T> {
        snapshot
            .get(self.path)
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok())
    }

    /// Reads the value, falling back when absent or of the wrong shape.
    pub fn get_or(&self, snapshot: &BTreeMap<String, JsonValue>, fallback: T) -> T {
        self.get(snapshot).unwrap_or(fallback)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;
    use unode::core::ast::{OneOrExpr, StringOrExpr, UiExpr};

    use super::StateKey;

    const COUNT: StateKey<u32> = StateKey::new("demo.count");
    const LABEL: StateKey<String> = StateKey::new("demo.label");

    #[test]
    fn reads_typed_values_from_snapshots() {
        let snapshot = BTreeMap::from([
            ("demo.count".to_string(), json!(7)),
            ("demo.label".to_string(), json!("seven")),
        ]);

        assert_eq!(COUNT.get(&snapshot), Some(7));
        assert_eq!(LABEL.get(&snapshot), Some("seven".to_string()));
        assert_eq!(COUNT.get_or(&BTreeMap::new(), 3), 3);
        // Wrong shape falls back instead of panicking.
        let bad = BTreeMap::from([("demo.count".to_string(), json!("not a number"))]);
        assert_eq!(COUNT.get_or(&bad, 3), 3);
    }

    #[test]
    fn set_records_a_state_write() {
        crate::host::clear_recorded_host_calls();
        COUNT.set(8);
        let calls = crate::host::recorded_host_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].operation, "state.set");
        assert_eq!(calls[0].params["path"], json!("demo.count"));
        assert_eq!(calls[0].params["value"], json!(8));
    }

    #[test]
    fn bind_text_fits_string_or_expr_fields() {
        let bound: StringOrExpr = COUNT.bind_text().into();
        assert_eq!(
            bound,
            OneOrExpr::Expr(UiExpr::Binding {
                path: "demo.count".to_string()
            })
        );
    }
}
