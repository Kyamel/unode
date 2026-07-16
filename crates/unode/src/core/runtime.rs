use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::ast::{BoolOrExpr, UNODE_AST_VERSION};
use crate::core::permissions::PermissionRequest;

pub const UNODE_CORE_API_VERSION: &str = UNODE_AST_VERSION;
pub type PluginId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedRoute {
    pub pattern: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub query: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub api_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<PermissionRequest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slot_contributions: Vec<SlotContributionDecl>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlotContributionDecl {
    pub id: String,
    pub target: String,
    #[serde(default)]
    pub priority: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<BoolOrExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ManifestValidationError {
    #[error("slot contribution id must not be empty")]
    EmptySlotContributionId,
    #[error("duplicate slot contribution id `{0}`")]
    DuplicateSlotContributionId(String),
    #[error("slot contribution `{id}` has an empty target")]
    EmptySlotContributionTarget { id: String },
}

impl PluginManifest {
    pub fn validate_slot_contributions(&self) -> Result<(), ManifestValidationError> {
        let mut ids = std::collections::BTreeSet::new();

        for contribution in &self.slot_contributions {
            if contribution.id.trim().is_empty() {
                return Err(ManifestValidationError::EmptySlotContributionId);
            }
            if !ids.insert(contribution.id.as_str()) {
                return Err(ManifestValidationError::DuplicateSlotContributionId(
                    contribution.id.clone(),
                ));
            }
            if contribution.target.trim().is_empty() {
                return Err(ManifestValidationError::EmptySlotContributionTarget {
                    id: contribution.id.clone(),
                });
            }
        }

        Ok(())
    }
}

impl Default for PluginManifest {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            version: "0.1.0".to_string(),
            api_version: UNODE_CORE_API_VERSION.to_string(),
            description: None,
            author: None,
            permissions: vec![],
            requires: vec![],
            host_id: None,
            slot_contributions: vec![],
        }
    }
}
