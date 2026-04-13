use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Hook events
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    UserPromptSubmit,
    Stop,
    SubagentStop,
    PreCompact,
    Notification,
    SubagentStart,
    PermissionRequest,
}

// ---------------------------------------------------------------------------
// Base hook input
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BaseHookInput {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub transcript_path: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub permission_mode: Option<String>,
}

// ---------------------------------------------------------------------------
// Specialized hook inputs
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PreToolUseHookInput {
    #[serde(flatten)]
    pub base: BaseHookInput,
    pub tool_name: String,
    pub tool_input: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PostToolUseHookInput {
    #[serde(flatten)]
    pub base: BaseHookInput,
    pub tool_name: String,
    pub tool_input: Value,
    pub tool_output: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PostToolUseFailureHookInput {
    #[serde(flatten)]
    pub base: BaseHookInput,
    pub tool_name: String,
    pub tool_input: Value,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserPromptSubmitHookInput {
    #[serde(flatten)]
    pub base: BaseHookInput,
    pub prompt: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StopHookInput {
    #[serde(flatten)]
    pub base: BaseHookInput,
    #[serde(default)]
    pub stop_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SubagentStopHookInput {
    #[serde(flatten)]
    pub base: BaseHookInput,
    #[serde(default)]
    pub result: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PreCompactHookInput {
    #[serde(flatten)]
    pub base: BaseHookInput,
    #[serde(default)]
    pub custom_instructions: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationHookInput {
    #[serde(flatten)]
    pub base: BaseHookInput,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SubagentStartHookInput {
    #[serde(flatten)]
    pub base: BaseHookInput,
    #[serde(default)]
    pub prompt: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRequestHookInput {
    #[serde(flatten)]
    pub base: BaseHookInput,
    pub tool_name: String,
    pub tool_input: Value,
}

// ---------------------------------------------------------------------------
// Hook input union
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "hook_event", rename_all = "snake_case")]
pub enum HookInput {
    PreToolUse(PreToolUseHookInput),
    PostToolUse(PostToolUseHookInput),
    PostToolUseFailure(PostToolUseFailureHookInput),
    UserPromptSubmit(UserPromptSubmitHookInput),
    Stop(StopHookInput),
    SubagentStop(SubagentStopHookInput),
    PreCompact(PreCompactHookInput),
    Notification(NotificationHookInput),
    SubagentStart(SubagentStartHookInput),
    PermissionRequest(PermissionRequestHookInput),
}

// ---------------------------------------------------------------------------
// Hook configuration
// ---------------------------------------------------------------------------

/// A single hook matcher configuration from settings.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HookMatcher {
    #[serde(default)]
    pub matcher: Option<String>,
    pub hooks: Vec<HookEntry>,
    #[serde(default)]
    pub timeout: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HookEntry {
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub script: Option<String>,
    #[serde(default)]
    pub timeout: Option<u64>,
}

// ---------------------------------------------------------------------------
// Hook output
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HookJSONOutput {
    #[serde(default)]
    pub decision: Option<String>,
    #[serde(default, rename = "systemMessage")]
    pub system_message: Option<String>,
    #[serde(default)]
    pub hook_specific_output: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_event_serialization() {
        let event = HookEvent::PreToolUse;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, r#""pre_tool_use""#);
    }

    #[test]
    fn hook_matcher_roundtrip() {
        let matcher = HookMatcher {
            matcher: Some("Read".into()),
            hooks: vec![HookEntry {
                command: Some("echo hello".into()),
                script: None,
                timeout: Some(5000),
            }],
            timeout: None,
        };
        let json = serde_json::to_string(&matcher).unwrap();
        let back: HookMatcher = serde_json::from_str(&json).unwrap();
        assert_eq!(matcher, back);
    }
}
