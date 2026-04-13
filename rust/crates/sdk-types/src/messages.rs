use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::content::ContentBlock;

// ---------------------------------------------------------------------------
// User message
// ---------------------------------------------------------------------------

/// Content of a user message — either a plain string or a list of content blocks.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum UserContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserMessage {
    pub content: UserContent,
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default)]
    pub parent_tool_use_id: Option<String>,
    #[serde(default)]
    pub tool_use_result: Option<Value>,
}

// ---------------------------------------------------------------------------
// Assistant message
// ---------------------------------------------------------------------------

/// Error variants that can appear on an assistant message.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssistantMessageError {
    AuthenticationFailed,
    BillingError,
    RateLimit,
    InvalidRequest,
    ServerError,
    MaxOutputTokens,
    Unknown(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssistantMessage {
    pub content: Vec<ContentBlock>,
    pub model: String,
    #[serde(default)]
    pub parent_tool_use_id: Option<String>,
    #[serde(default)]
    pub error: Option<AssistantMessageError>,
    #[serde(default)]
    pub usage: Option<Value>,
    #[serde(default)]
    pub message_id: Option<String>,
}

// ---------------------------------------------------------------------------
// System message
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SystemMessage {
    pub subtype: String,
    pub data: Value,
}

// ---------------------------------------------------------------------------
// Result message
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ResultMessage {
    pub subtype: String,
    pub duration_ms: u64,
    pub duration_api_ms: u64,
    pub is_error: bool,
    pub num_turns: u32,
    pub session_id: String,
    #[serde(default)]
    pub total_cost_usd: Option<f64>,
    #[serde(default)]
    pub usage: Option<Value>,
    #[serde(default)]
    pub result: Option<String>,
    #[serde(default)]
    pub stop_reason: Option<String>,
    #[serde(default)]
    pub structured_output: Option<Value>,
    #[serde(default)]
    pub model_usage: Option<Value>,
}

// ---------------------------------------------------------------------------
// Stream event
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StreamEvent {
    pub uuid: String,
    pub session_id: String,
    pub event: Value,
    #[serde(default)]
    pub parent_tool_use_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Rate limit event
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RateLimitInfo {
    pub status: String,
    #[serde(default)]
    pub resets_at: Option<String>,
    #[serde(default)]
    pub rate_limit_type: Option<String>,
    #[serde(default)]
    pub utilization: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitEvent {
    pub rate_limit_info: RateLimitInfo,
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Task messages
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskStartedMessage {
    pub task_id: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub tool_use_id: Option<String>,
    #[serde(default)]
    pub task_type: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TaskUsage {
    #[serde(default)]
    pub total_tokens: Option<u64>,
    #[serde(default)]
    pub tool_uses: Option<u64>,
    #[serde(default)]
    pub duration_ms: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskProgressMessage {
    pub task_id: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub usage: Option<TaskUsage>,
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskNotificationMessage {
    pub task_id: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub output_file: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Top-level message union
// ---------------------------------------------------------------------------

/// All message types that can appear on the SDK wire.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    User(UserMessage),
    Assistant(AssistantMessage),
    System(SystemMessage),
    Result(ResultMessage),
    StreamEvent(StreamEvent),
    RateLimit(RateLimitEvent),
    TaskStarted(TaskStartedMessage),
    TaskProgress(TaskProgressMessage),
    TaskNotification(TaskNotificationMessage),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::TextBlock;

    #[test]
    fn user_message_text_roundtrip() {
        let msg = Message::User(UserMessage {
            content: UserContent::Text("hello".into()),
            uuid: Some("uuid-1".into()),
            parent_tool_use_id: None,
            tool_use_result: None,
        });
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"user""#));
        assert!(json.contains(r#""content":"hello""#));
        let back: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn assistant_message_roundtrip() {
        let msg = Message::Assistant(AssistantMessage {
            content: vec![ContentBlock::Text(TextBlock {
                text: "response".into(),
            })],
            model: "claude-sonnet-4-6".into(),
            parent_tool_use_id: None,
            error: None,
            usage: Some(serde_json::json!({"input_tokens": 100, "output_tokens": 50})),
            message_id: Some("msg-1".into()),
        });
        let json = serde_json::to_string(&msg).unwrap();
        let back: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn result_message_roundtrip() {
        let msg = Message::Result(ResultMessage {
            subtype: "success".into(),
            duration_ms: 1234,
            duration_api_ms: 1000,
            is_error: false,
            num_turns: 1,
            session_id: "sess-1".into(),
            total_cost_usd: Some(0.01),
            usage: Some(serde_json::json!({"input_tokens": 100, "output_tokens": 50})),
            result: Some("done".into()),
            stop_reason: Some("end_turn".into()),
            structured_output: None,
            model_usage: None,
        });
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"result""#));
        let back: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, back);
    }
}
