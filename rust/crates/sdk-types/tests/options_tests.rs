//! Options serialization tests: verify PermissionMode, ThinkingConfig,
//! ClaudeAgentOptions, and related types serialize/deserialize to the exact
//! JSON the Python SDK expects.

use sdk_types::options::{PermissionMode, ThinkingConfig};

// ---------------------------------------------------------------------------
// PermissionMode
// ---------------------------------------------------------------------------

#[test]
fn permission_mode_default() {
    let mode = PermissionMode::Default;
    let json = serde_json::to_string(&mode).unwrap();
    assert_eq!(json, r#""default""#);
}

#[test]
fn permission_mode_accept_edits() {
    let mode = PermissionMode::AcceptEdits;
    let json = serde_json::to_string(&mode).unwrap();
    assert_eq!(json, r#""acceptEdits""#);
}

#[test]
fn permission_mode_plan() {
    let mode = PermissionMode::Plan;
    let json = serde_json::to_string(&mode).unwrap();
    assert_eq!(json, r#""plan""#);
}

#[test]
fn permission_mode_dont_ask() {
    let mode = PermissionMode::DontAsk;
    let json = serde_json::to_string(&mode).unwrap();
    assert_eq!(json, r#""dontAsk""#);
}

#[test]
fn permission_mode_bypass_permissions() {
    let mode = PermissionMode::BypassPermissions;
    let json = serde_json::to_string(&mode).unwrap();
    assert_eq!(json, r#""bypassPermissions""#);
}

#[test]
fn permission_mode_roundtrip_all_variants() {
    let modes = [
        PermissionMode::Default,
        PermissionMode::AcceptEdits,
        PermissionMode::Plan,
        PermissionMode::DontAsk,
        PermissionMode::BypassPermissions,
    ];
    for mode in modes {
        let json = serde_json::to_string(&mode).unwrap();
        let back: PermissionMode = serde_json::from_str(&json).unwrap();
        assert_eq!(mode, back, "roundtrip failed for {json}");
    }
}

#[test]
fn permission_mode_deserialize_from_python_strings() {
    let cases = [
        (r#""default""#, PermissionMode::Default),
        (r#""acceptEdits""#, PermissionMode::AcceptEdits),
        (r#""plan""#, PermissionMode::Plan),
        (r#""dontAsk""#, PermissionMode::DontAsk),
        (r#""bypassPermissions""#, PermissionMode::BypassPermissions),
    ];
    for (json, expected) in cases {
        let parsed: PermissionMode = serde_json::from_str(json).unwrap();
        assert_eq!(parsed, expected, "failed to parse {json}");
    }
}

// ---------------------------------------------------------------------------
// ThinkingConfig — uses #[serde(tag = "type", rename_all = "camelCase")]
// so budget_tokens serializes as budgetTokens.
// ---------------------------------------------------------------------------

#[test]
fn thinking_config_adaptive() {
    let config = ThinkingConfig::Adaptive;
    let json = serde_json::to_string(&config).unwrap();
    assert_eq!(json, r#"{"type":"adaptive"}"#);
}

#[test]
fn thinking_config_enabled() {
    let config = ThinkingConfig::Enabled { budget_tokens: 20000 };
    let json = serde_json::to_string(&config).unwrap();
    assert_eq!(json, r#"{"type":"enabled","budgetTokens":20000}"#);
}

#[test]
fn thinking_config_disabled() {
    let config = ThinkingConfig::Disabled;
    let json = serde_json::to_string(&config).unwrap();
    assert_eq!(json, r#"{"type":"disabled"}"#);
}

#[test]
fn thinking_config_roundtrip_all_variants() {
    let configs = [
        ThinkingConfig::Adaptive,
        ThinkingConfig::Enabled { budget_tokens: 20000 },
        ThinkingConfig::Enabled { budget_tokens: 100000 },
        ThinkingConfig::Disabled,
    ];
    for config in &configs {
        let json = serde_json::to_string(config).unwrap();
        let back: ThinkingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, &back, "roundtrip failed for {json}");
    }
}

#[test]
fn thinking_config_deserialize_from_python_json() {
    // Python SDK sends camelCase field names
    let cases = [
        (r#"{"type":"adaptive"}"#, ThinkingConfig::Adaptive),
        (
            r#"{"type":"enabled","budgetTokens":20000}"#,
            ThinkingConfig::Enabled { budget_tokens: 20000 },
        ),
        (r#"{"type":"disabled"}"#, ThinkingConfig::Disabled),
    ];
    for (json, expected) in cases {
        let parsed: ThinkingConfig = serde_json::from_str(json).unwrap();
        assert_eq!(parsed, expected, "failed to parse {json}");
    }
}
