//! Wire format tests: verify that every ContentBlock, Message, and envelope
//! type serializes to the exact JSON the Python SDK expects, and round-trips
//! correctly.

use sdk_types::content::{ContentBlock, TextBlock, ThinkingBlock, ToolResultBlock, ToolResultContent, ToolUseBlock};
use sdk_types::messages::{
    AssistantMessage, AssistantMessageError, Message, RateLimitEvent, RateLimitInfo, ResultMessage,
    StreamEvent, SystemMessage, TaskNotificationMessage, TaskProgressMessage, TaskStartedMessage,
    TaskUsage, UserContent, UserMessage,
};

// ---------------------------------------------------------------------------
// ContentBlock variants — exact format matching Python SDK
// ---------------------------------------------------------------------------

#[test]
fn text_block_serializes_to_python_sdk_format() {
    let block = ContentBlock::Text(TextBlock {
        text: "hello".into(),
    });
    let json = serde_json::to_string(&block).unwrap();
    assert_eq!(json, r#"{"type":"text","text":"hello"}"#);
}

#[test]
fn text_block_roundtrip() {
    let block = ContentBlock::Text(TextBlock {
        text: "multi\nline\ncontent".into(),
    });
    let json = serde_json::to_string(&block).unwrap();
    let back: ContentBlock = serde_json::from_str(&json).unwrap();
    assert_eq!(block, back);
}

#[test]
fn thinking_block_serializes_to_python_sdk_format() {
    let block = ContentBlock::Thinking(ThinkingBlock {
        thinking: "pondering".into(),
        signature: "sig_abc".into(),
    });
    let json = serde_json::to_string(&block).unwrap();
    assert_eq!(
        json,
        r#"{"type":"thinking","thinking":"pondering","signature":"sig_abc"}"#
    );
}

#[test]
fn thinking_block_roundtrip() {
    let block = ContentBlock::Thinking(ThinkingBlock {
        thinking: "deep thought".into(),
        signature: "sig_xyz".into(),
    });
    let json = serde_json::to_string(&block).unwrap();
    let back: ContentBlock = serde_json::from_str(&json).unwrap();
    assert_eq!(block, back);
}

#[test]
fn tool_use_block_serializes_to_python_sdk_format() {
    let block = ContentBlock::ToolUse(ToolUseBlock {
        id: "tu_123".into(),
        name: "Read".into(),
        input: serde_json::json!({"file_path": "/test.rs"}),
    });
    let json = serde_json::to_string(&block).unwrap();
    assert_eq!(
        json,
        r#"{"type":"tool_use","id":"tu_123","name":"Read","input":{"file_path":"/test.rs"}}"#
    );
}

#[test]
fn tool_use_block_with_complex_input_roundtrip() {
    let block = ContentBlock::ToolUse(ToolUseBlock {
        id: "tu_456".into(),
        name: "Write".into(),
        input: serde_json::json!({
            "file_path": "/tmp/out.txt",
            "content": "hello world"
        }),
    });
    let json = serde_json::to_string(&block).unwrap();
    assert!(json.contains(r#""type":"tool_use""#));
    assert!(json.contains(r#""id":"tu_456""#));
    assert!(json.contains(r#""name":"Write""#));

    let back: ContentBlock = serde_json::from_str(&json).unwrap();
    assert_eq!(block, back);
}

#[test]
fn tool_result_block_with_text_content() {
    let block = ContentBlock::ToolResult(ToolResultBlock {
        tool_use_id: "tu_123".into(),
        content: ToolResultContent::Text("result text".into()),
        is_error: false,
    });
    let json = serde_json::to_string(&block).unwrap();
    assert_eq!(
        json,
        r#"{"type":"tool_result","tool_use_id":"tu_123","content":"result text","is_error":false}"#
    );
}

#[test]
fn tool_result_block_with_error() {
    let block = ContentBlock::ToolResult(ToolResultBlock {
        tool_use_id: "tu_789".into(),
        content: ToolResultContent::Text("command failed".into()),
        is_error: true,
    });
    let json = serde_json::to_string(&block).unwrap();
    assert!(json.contains(r#""is_error":true"#));

    let back: ContentBlock = serde_json::from_str(&json).unwrap();
    assert_eq!(block, back);
}

#[test]
fn tool_result_block_with_nested_blocks() {
    let block = ContentBlock::ToolResult(ToolResultBlock {
        tool_use_id: "tu_999".into(),
        content: ToolResultContent::Blocks(vec![ContentBlock::Text(TextBlock {
            text: "structured result".into(),
        })]),
        is_error: false,
    });
    let json = serde_json::to_string(&block).unwrap();
    assert!(json.contains(r#""type":"tool_result""#));
    assert!(json.contains(r#"structured result"#));

    let back: ContentBlock = serde_json::from_str(&json).unwrap();
    assert_eq!(block, back);
}

#[test]
fn tool_result_block_default_is_error_is_false() {
    let json = r#"{"type":"tool_result","tool_use_id":"tu_100","content":"ok"}"#;
    let block: ContentBlock = serde_json::from_str(json).unwrap();
    match block {
        ContentBlock::ToolResult(tr) => assert!(!tr.is_error),
        _ => panic!("Expected ToolResult variant"),
    }
}

// ---------------------------------------------------------------------------
// UserContent (from messages module)
// ---------------------------------------------------------------------------

#[test]
fn user_content_text_serializes_as_string() {
    let content = UserContent::Text("Hello Claude".into());
    let json = serde_json::to_string(&content).unwrap();
    assert_eq!(json, r#""Hello Claude""#);
}

#[test]
fn user_content_blocks_serializes_as_array() {
    let content = UserContent::Blocks(vec![ContentBlock::Text(TextBlock {
        text: "hello".into(),
    })]);
    let json = serde_json::to_string(&content).unwrap();
    assert!(json.starts_with('['));
    assert!(json.contains(r#""type":"text""#));
}

// ---------------------------------------------------------------------------
// UserMessage
// ---------------------------------------------------------------------------

#[test]
fn user_message_roundtrip() {
    let msg = UserMessage {
        content: UserContent::Text("Hello".into()),
        uuid: Some("uuid-1".into()),
        parent_tool_use_id: None,
        tool_use_result: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: UserMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, back);
}

// ---------------------------------------------------------------------------
// AssistantMessage
// ---------------------------------------------------------------------------

#[test]
fn assistant_message_with_text_only() {
    let msg = AssistantMessage {
        content: vec![ContentBlock::Text(TextBlock {
            text: "Hello!".into(),
        })],
        model: "claude-sonnet-4-6".into(),
        parent_tool_use_id: None,
        error: None,
        usage: None,
        message_id: Some("msg_001".into()),
    };
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains(r#""text":"Hello!""#));
    assert!(json.contains(r#""model":"claude-sonnet-4-6""#));

    let back: AssistantMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, back);
}

#[test]
fn assistant_message_with_tool_use() {
    let msg = AssistantMessage {
        content: vec![
            ContentBlock::Text(TextBlock {
                text: "Let me read that file.".into(),
            }),
            ContentBlock::ToolUse(ToolUseBlock {
                id: "tu_001".into(),
                name: "Read".into(),
                input: serde_json::json!({"file_path": "/src/main.rs"}),
            }),
        ],
        model: "claude-sonnet-4-6".into(),
        parent_tool_use_id: None,
        error: None,
        usage: Some(serde_json::json!({"input_tokens": 100, "output_tokens": 50})),
        message_id: Some("msg_002".into()),
    };
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains(r#""type":"tool_use""#));
    assert!(json.contains(r#""name":"Read""#));

    let back: AssistantMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, back);
}

#[test]
fn assistant_message_error_variant() {
    let msg = AssistantMessage {
        content: vec![],
        model: "claude-sonnet-4-6".into(),
        parent_tool_use_id: None,
        error: Some(AssistantMessageError::RateLimit),
        usage: None,
        message_id: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains(r#""error":"rate_limit""#));

    let back: AssistantMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, back);
}

// ---------------------------------------------------------------------------
// ResultMessage
// ---------------------------------------------------------------------------

#[test]
fn result_message_minimal_fields() {
    let msg = ResultMessage {
        subtype: "success".into(),
        duration_ms: 1500,
        duration_api_ms: 1200,
        is_error: false,
        num_turns: 1,
        session_id: "sess_abc".into(),
        total_cost_usd: None,
        usage: None,
        result: None,
        stop_reason: None,
        structured_output: None,
        model_usage: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ResultMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, back);
}

#[test]
fn result_message_all_fields() {
    let msg = ResultMessage {
        subtype: "success".into(),
        duration_ms: 1500,
        duration_api_ms: 1200,
        is_error: false,
        num_turns: 1,
        session_id: "sess_abc".into(),
        total_cost_usd: Some(0.01),
        usage: Some(serde_json::json!({
            "input_tokens": 100,
            "output_tokens": 50,
            "cache_creation_input_tokens": 0,
            "cache_read_input_tokens": 0
        })),
        result: Some("Hello!".into()),
        stop_reason: Some("end_turn".into()),
        structured_output: None,
        model_usage: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains(r#""subtype":"success""#));
    assert!(json.contains(r#""input_tokens":100"#));
    assert!(json.contains(r#""result":"Hello!""#));
    assert!(json.contains(r#""stopReason":"end_turn""#));
}

#[test]
fn result_message_error() {
    let msg = ResultMessage {
        subtype: "error".into(),
        duration_ms: 500,
        duration_api_ms: 300,
        is_error: true,
        num_turns: 0,
        session_id: "sess_err".into(),
        total_cost_usd: None,
        usage: None,
        result: Some("Something went wrong".into()),
        stop_reason: None,
        structured_output: None,
        model_usage: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains(r#""isError":true"#));
    assert!(json.contains(r#""result":"Something went wrong""#));
}

// ---------------------------------------------------------------------------
// RateLimitInfo / RateLimitEvent
// ---------------------------------------------------------------------------

#[test]
fn rate_limit_info_minimal() {
    let info = RateLimitInfo {
        status: "allowed".into(),
        resets_at: None,
        rate_limit_type: None,
        utilization: None,
    };
    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains(r#""status":"allowed""#));

    let back: RateLimitInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(info, back);
}

#[test]
fn rate_limit_event_roundtrip() {
    let event = RateLimitEvent {
        rate_limit_info: RateLimitInfo {
            status: "rate_limited".into(),
            resets_at: Some("2026-04-13T12:00:00Z".into()),
            rate_limit_type: Some("tokens".into()),
            utilization: Some(0.8),
        },
        uuid: Some("uuid_rl".into()),
        session_id: Some("sess_rl".into()),
    };
    let json = serde_json::to_string(&event).unwrap();
    let back: RateLimitEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event, back);
}

// ---------------------------------------------------------------------------
// TaskUsage
// ---------------------------------------------------------------------------

#[test]
fn task_usage_roundtrip() {
    let usage = TaskUsage {
        total_tokens: Some(100),
        tool_uses: Some(1),
        duration_ms: Some(500),
    };
    let json = serde_json::to_string(&usage).unwrap();
    let back: TaskUsage = serde_json::from_str(&json).unwrap();
    assert_eq!(usage, back);
}

// ---------------------------------------------------------------------------
// Task messages
// ---------------------------------------------------------------------------

#[test]
fn task_started_message_roundtrip() {
    let msg = TaskStartedMessage {
        task_id: "task_001".into(),
        description: Some("Running bash command".into()),
        uuid: Some("uuid_ts".into()),
        session_id: Some("sess_ts".into()),
        tool_use_id: Some("tu_300".into()),
        task_type: Some("local_bash".into()),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: TaskStartedMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, back);
}

#[test]
fn task_progress_with_typed_usage() {
    let msg = TaskProgressMessage {
        task_id: "task_001".into(),
        description: Some("Processing...".into()),
        usage: Some(TaskUsage {
            total_tokens: Some(100),
            tool_uses: Some(1),
            duration_ms: Some(500),
        }),
        uuid: Some("uuid_tp".into()),
        session_id: Some("sess_tp".into()),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: TaskProgressMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, back);
}

#[test]
fn task_notification_message_roundtrip() {
    let msg = TaskNotificationMessage {
        task_id: "task_001".into(),
        status: Some("completed".into()),
        output_file: Some("/tmp/output.txt".into()),
        summary: Some("Done successfully".into()),
        uuid: Some("uuid_tn".into()),
        session_id: Some("sess_tn".into()),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: TaskNotificationMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, back);
}

// ---------------------------------------------------------------------------
// StreamEvent
// ---------------------------------------------------------------------------

#[test]
fn stream_event_roundtrip() {
    let event = StreamEvent {
        uuid: "uuid_se".into(),
        session_id: "sess_se".into(),
        event: serde_json::json!({"type": "content_block_delta", "index": 0}),
        parent_tool_use_id: None,
    };
    let json = serde_json::to_string(&event).unwrap();
    let back: StreamEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event, back);
}

// ---------------------------------------------------------------------------
// SystemMessage
// ---------------------------------------------------------------------------

#[test]
fn system_message_roundtrip() {
    let msg = SystemMessage {
        subtype: "init".into(),
        data: serde_json::json!({"session_id": "sess_001"}),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: SystemMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, back);
}
