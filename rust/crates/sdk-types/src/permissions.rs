use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Permission result types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "behavior")]
pub enum PermissionResult {
    #[serde(rename = "allow")]
    Allow(PermissionResultAllow),
    #[serde(rename = "deny")]
    Deny(PermissionResultDeny),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PermissionResultAllow {
    #[serde(default)]
    pub updated_input: Option<Value>,
    #[serde(default)]
    pub updated_permissions: Option<PermissionUpdate>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PermissionResultDeny {
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub interrupt: Option<bool>,
}

// ---------------------------------------------------------------------------
// Permission update
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PermissionUpdate {
    #[serde(rename = "type", default)]
    pub update_type: Option<String>,
    #[serde(default)]
    pub rules: Option<Vec<PermissionRuleValue>>,
    #[serde(default)]
    pub behavior: Option<String>,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub directories: Option<Vec<String>>,
    #[serde(default)]
    pub destination: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PermissionRuleValue {
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub rule_content: Option<String>,
}

// ---------------------------------------------------------------------------
// Tool permission context
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ToolPermissionContext {
    #[serde(default)]
    pub signal: Option<String>,
    #[serde(default)]
    pub suggestions: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_result_allow_roundtrip() {
        let result = PermissionResult::Allow(PermissionResultAllow {
            updated_input: None,
            updated_permissions: None,
        });
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains(r#""behavior":"allow""#));
        let back: PermissionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, back);
    }

    #[test]
    fn permission_result_deny_roundtrip() {
        let result = PermissionResult::Deny(PermissionResultDeny {
            message: Some("not allowed".into()),
            interrupt: Some(true),
        });
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains(r#""behavior":"deny""#));
        let back: PermissionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, back);
    }
}
