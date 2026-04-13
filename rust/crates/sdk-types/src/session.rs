use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Summary information about a session.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKSessionInfo {
    pub session_id: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub last_modified: Option<String>,
    #[serde(default)]
    pub file_size: Option<u64>,
    #[serde(default)]
    pub custom_title: Option<String>,
    #[serde(default)]
    pub first_prompt: Option<String>,
    #[serde(default)]
    pub git_branch: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// A single message within a session transcript.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SessionMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    pub message: Value,
    #[serde(default)]
    pub parent_tool_use_id: Option<String>,
}

/// An MCP tool registered via the SDK.
///
/// The `handler` field is not serializable; it is represented here for type
/// completeness but actual dispatch uses closure state managed by the SDK crate.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SdkMcpTool {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub input_schema: Option<Value>,
    #[serde(default)]
    pub annotations: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_info_roundtrip() {
        let info = SDKSessionInfo {
            session_id: "abc-123".into(),
            summary: Some("test session".into()),
            last_modified: None,
            file_size: None,
            custom_title: None,
            first_prompt: Some("hello".into()),
            git_branch: None,
            cwd: Some("/tmp".into()),
            tag: None,
            created_at: None,
        };
        let json = serde_json::to_string(&info).unwrap();
        let back: SDKSessionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info, back);
    }
}
