use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::core::ast::UNODE_AST_VERSION;
use crate::core::canonical::CanonicalScreen;
use crate::core::ir::{lower_screen, IrScreen};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenEnvelope {
    pub r#type: String,
    pub v: String,
    pub ts: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screen_kind: Option<String>,
    pub screen: IrScreen,
}

#[derive(Debug, Clone, Default)]
pub struct SerializeOptions {
    pub screen_kind: Option<String>,
    pub pretty: bool,
}

pub fn screen_to_json(screen: &IrScreen, options: &SerializeOptions) -> Result<String, serde_json::Error> {
    let envelope = ScreenEnvelope {
        r#type: "unode-screen".into(),
        v: UNODE_AST_VERSION.into(),
        ts: current_timestamp(),
        screen_kind: options.screen_kind.clone(),
        screen: screen.clone(),
    };

    if options.pretty {
        serde_json::to_string_pretty(&envelope)
    } else {
        serde_json::to_string(&envelope)
    }
}

pub fn canonical_screen_to_json(
    screen: &CanonicalScreen,
    options: &SerializeOptions,
) -> Result<String, serde_json::Error> {
    let ir = lower_screen(screen);
    screen_to_json(&ir, options)
}

pub fn screen_from_json(json: &str) -> Result<ScreenEnvelope, String> {
    let parsed: ScreenEnvelope =
        serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {e}"))?;

    if parsed.r#type != "unode-screen" {
        return Err(format!("Unknown envelope type: {}", parsed.r#type));
    }

    if !is_version_compatible(&parsed.v, UNODE_AST_VERSION) {
        return Err(format!(
            "AST version mismatch: received {:?}, expected compatible with {:?}",
            parsed.v, UNODE_AST_VERSION
        ));
    }

    Ok(parsed)
}

fn is_version_compatible(received: &str, expected: &str) -> bool {
    let received_major = received.split('.').next().unwrap_or_default();
    let expected_major = expected.split('.').next().unwrap_or_default();
    received_major == expected_major
}

fn current_timestamp() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => format!("unix:{}", duration.as_secs()),
        Err(_) => "unix:0".to_string(),
    }
}
