use serde_json::Value as JsonValue;

use crate::core::ast::{ActionRef, BoolOrExpr, NumberOrExpr, PrimitiveOrExpr, StringOrExpr};
use crate::core::canonical::{CanonicalUiNode, ReactiveField};

#[derive(Debug, Clone)]
pub enum PatchValue {
    String(StringOrExpr),
    Bool(BoolOrExpr),
    Number(NumberOrExpr),
    Primitive(PrimitiveOrExpr),
    Action(ActionRef),
    Json(JsonValue),
}

#[derive(Debug, Clone)]
pub enum PatchOp {
    SetProp {
        key: String,
        field: ReactiveField,
        value: PatchValue,
    },
    ReplaceNode {
        key: String,
        node: CanonicalUiNode,
    },
    ReplaceChildren {
        key: String,
        children: Vec<CanonicalUiNode>,
    },
}
