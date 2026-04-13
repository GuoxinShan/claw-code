//! Golden file tests: verify full message envelopes (SdkOutputEnvelope and
//! Message) serialize to exactly the JSON format the Python SDK expects.

use sdk_types::content::{ContentBlock, TextBlock, ToolUseBlock};
use sdk_types::messages::{
    AssistantMessage, Message, RateLimitEvent, RateLimitInfo, ResultMessage, StreamEvent,
    TaskNotificationMessage, TaskProgressMessage, TaskStartedMessage, TaskUsage, UserContent,
    UserMessage,
};
use sdk_types::wire::{
    AssistantEnvelope, SdkOutputEnvelope, WireDecoder, WireEncoder,
};

// ---------------------------------------------------------------------------
// SdkOutputEnvelope — system init
// ---------------------------------------------------------------------------

#[test]
fn system_init_envelope_format() {
    let envelope = SdkOutputEnvelope::System {
        subtype: "init".into(),
        session_id: "sess_001".into(),
        tools: Some(vec!["Read".into(), "Write".into(), "Bash".into()]),
        model: Some("claude-sonnet-4-6".into()),
        cwd: Some("/home/user/project".into()),
    };
    let json = serde_json::to_string(&envelope).unwrap();

    assert!(json.starts_with(r#"{"type":"system""#));
    assert!(json.contains(r#""subtype":"init""#));
    assert!(json.contains(r#""session_id":"sess_001""#));
    assert!(json.contains(r#""model":"claude-sonnet-4-6""#));
    assert!(json.contains(r#""Read""#));
    assert!(json.contains(r#"cwd"#));

    let back: SdkOutputEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(envelope, back);
}

#[test]
fn system_init_envelope_minimal() {
    let envelope = SdkOutputEnvelope::System {
        subtype: "init".into(),
        session_id: "sess_min".into(),
        tools: None,
        model: None,
        cwd: None,
    };
    let json = serde_json::to_string(&envelope).unwrap();

    assert!(json.starts_with(r#"{"type":"system""#));
    assert!(json.contains(r#""subtype":"init""#));
    assert!(json.contains(r#""session_id":"sess_min""#));
}

// ---------------------------------------------------------------------------
// SdkOutputEnvelope — assistant
// ---------------------------------------------------------------------------

#[test]
fn assistant_envelope_text_only() {
    let envelope = SdkOutputEnvelope::Assistant {
        message: AssistantEnvelope::new(vec![ContentBlock::Text(TextBlock {
            text: "Hello!".into(),
        })]),
        model: Some("claude-sonnet-4-6".into()),
        session_id: Some("sess_002".into()),
        uuid: Some("uuid_abc".into()),
        message_id: Some("msg_001".into()),
    };
    let json = serde_json::to_string(&envelope).unwrap();

    assert!(json.starts_with(r#"{"type":"assistant""#));
    assert!(json.contains(r#""model":"claude-sonnet-4-6""#));
    assert!(json.contains(r#""session_id":"sess_002""#));
    assert!(json.contains(r#""uuid":"uuid_abc""#));

    let back: SdkOutputEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(envelope, back);
}

#[test]
fn assistant_envelope_with_tool_use() {
    let envelope = SdkOutputEnvelope::Assistant {
        message: AssistantEnvelope::new(vec![
            ContentBlock::Text(TextBlock {
                text: "Let me read that.".into(),
            }),
            ContentBlock::ToolUse(ToolUseBlock {
                id: "tu_001".into(),
                name: "Read".into(),
                input: serde_json::json!({"file_path": "/src/main.rs"}),
            }),
        ]),
        model: Some("claude-sonnet-4-6".into()),
        session_id: Some("sess_003".into()),
        uuid: Some("uuid_def".into()),
        message_id: Some("msg_002".into()),
    };
    let json = serde_json::to_string(&envelope).unwrap();

    assert!(json.starts_with(r#"{"type":"assistant""#));
    assert!(json.contains(r#"tool_use"#));
    assert!(json.contains(r#""name":"Read""#));

    let back: SdkOutputEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(envelope, back);
}

#[test]
fn assistant_envelope_optional_fields_omitted() {
    let envelope = SdkOutputEnvelope::Assistant {
        message: AssistantEnvelope::new(vec![]),
        model: None,
        session_id: None,
        uuid: None,
        message_id: None,
    };
    let json = serde_json::to_string(&envelope).unwrap();

    assert!(json.starts_with(r#"{"type":"assistant""#));
}

// ---------------------------------------------------------------------------
// SdkOutputEnvelope — tool result
// ---------------------------------------------------------------------------

#[test]
fn tool_result_envelope_format() {
    let envelope = SdkOutputEnvelope::ToolResult {
        tool_use_id: "tu_100".into(),
        content: "file contents here".into(),
        is_error: false,
    };
    let json = serde_json::to_string(&envelope).unwrap();

    assert!(json.starts_with(r#"{"type":"tool_result""#));
    assert!(json.contains(r#""tool_use_id":"tu_100""#));
    assert!(json.contains(r#"file contents here"#));

    let back: SdkOutputEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(envelope, back);
}

#[test]
fn tool_result_envelope_error() {
    let envelope = SdkOutputEnvelope::ToolResult {
        tool_use_id: "tu_200".into(),
        content: "command failed".into(),
        is_error: true,
    };
    let json = serde_json::to_string(&envelope).unwrap();

    assert!(json.contains(r#""is_error":true"#));
}

// ---------------------------------------------------------------------------
// SdkOutputEnvelope — result
// ---------------------------------------------------------------------------

#[test]
fn result_envelope_success_format() {
    let envelope = SdkOutputEnvelope::Result {
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
    };
    let json = serde_json::to_string(&envelope).unwrap();

    assert!(json.starts_with(r#"{"type":"result""#));
    assert!(json.contains(r#""subtype":"success""#));
    assert!(json.contains(r#""duration_ms":1500"#));
    assert!(json.contains(r#""duration_api_ms":1200"#));
    assert!(json.contains(r#""is_error":false"#));
    assert!(json.contains(r#""num_turns":1"#));
    assert!(json.contains(r#""input_tokens":100"#));
    assert!(json.contains(r#""result":"Hello!""#));
    assert!(json.contains(r#""stop_reason":"end_turn""#));

    let back: SdkOutputEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(envelope, back);
}

#[test]
fn result_envelope_error_format() {
    let envelope = SdkOutputEnvelope::Result {
        subtype: "error".into(),
        duration_ms: 500,
        duration_api_ms: 300,
        is_error: true,
        num_turns: 0,
        session_id: "sess_err".into(),
        total_cost_usd: None,
        usage: None,
        result: Some("Error: file not found".into()),
        stop_reason: None,
    };
    let json = serde_json::to_string(&envelope).unwrap();

    assert!(json.starts_with(r#"{"type":"result""#));
    assert!(json.contains(r#""subtype":"error""#));
    assert!(json.contains(r#""is_error":true"#));
}

// ---------------------------------------------------------------------------
// SdkOutputEnvelope — stream event
// ---------------------------------------------------------------------------

#[test]
fn stream_event_envelope() {
    let envelope = SdkOutputEnvelope::StreamEvent {
        uuid: "uuid_stream".into(),
        session_id: "sess_str".into(),
        event: serde_json::json!({"type": "content_block_start", "index": 0}),
    };
    let json = serde_json::to_string(&envelope).unwrap();

    assert!(json.starts_with(r#"{"type":"stream_event""#));
    assert!(json.contains(r#""uuid":"uuid_stream""#));
    assert!(json.contains(r#""session_id":"sess_str""#));
    assert!(json.contains(r#""content_block_start""#));

    let back: SdkOutputEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(envelope, back);
}

// ---------------------------------------------------------------------------
// SdkOutputEnvelope — rate limit
// ---------------------------------------------------------------------------

#[test]
fn rate_limit_envelope() {
    let envelope = SdkOutputEnvelope::RateLimit {
        rate_limit_info: RateLimitInfo {
            status: "allowed".into(),
            resets_at: None,
            rate_limit_type: None,
            utilization: None,
        },
        uuid: "uuid_rl".into(),
        session_id: "sess_rl".into(),
    };
    let json = serde_json::to_string(&envelope).unwrap();

    assert!(json.starts_with(r#"{"type":"rate_limit""#));
    assert!(json.contains(r#"rate_limit_info"#));
    assert!(json.contains(r#""status":"allowed""#));
    assert!(json.contains(r#""uuid":"uuid_rl""#));

    let back: SdkOutputEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(envelope, back);
}

// ---------------------------------------------------------------------------
// SdkOutputEnvelope — task lifecycle
// ---------------------------------------------------------------------------

#[test]
fn task_started_envelope() {
    let envelope = SdkOutputEnvelope::TaskStarted {
        task_id: "task_001".into(),
        description: Some("Running bash command".into()),
        uuid: "uuid_ts".into(),
        session_id: "sess_ts".into(),
        tool_use_id: Some("tu_300".into()),
        task_type: Some("local_bash".into()),
    };
    let json = serde_json::to_string(&envelope).unwrap();

    assert!(json.starts_with(r#"{"type":"task_started""#));
    assert!(json.contains(r#""task_id":"task_001""#));
    assert!(json.contains(r#""description":"Running bash command""#));
    assert!(json.contains(r#""tool_use_id":"tu_300""#));
    assert!(json.contains(r#""task_type":"local_bash""#));

    let back: SdkOutputEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(envelope, back);
}

#[test]
fn task_progress_envelope() {
    let envelope = SdkOutputEnvelope::TaskProgress {
        task_id: "task_001".into(),
        description: Some("Still running...".into()),
        usage: Some(serde_json::json!({
            "total_tokens": 100,
            "tool_uses": 1,
            "duration_ms": 500
        })),
        uuid: "uuid_tp".into(),
        session_id: "sess_tp".into(),
    };
    let json = serde_json::to_string(&envelope).unwrap();

    assert!(json.starts_with(r#"{"type":"task_progress""#));
    assert!(json.contains(r#""task_id":"task_001""#));
    assert!(json.contains(r#"total_tokens"#));

    let back: SdkOutputEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(envelope, back);
}

#[test]
fn task_notification_envelope_completed() {
    let envelope = SdkOutputEnvelope::TaskNotification {
        task_id: "task_001".into(),
        status: "completed".into(),
        output_file: Some("/tmp/output.txt".into()),
        summary: Some("Done successfully".into()),
        uuid: "uuid_tn".into(),
        session_id: "sess_tn".into(),
    };
    let json = serde_json::to_string(&envelope).unwrap();

    assert!(json.starts_with(r#"{"type":"task_notification""#));
    assert!(json.contains(r#""task_id":"task_001""#));
    assert!(json.contains(r#""status":"completed""#));
    assert!(json.contains(r#""output_file":"/tmp/output.txt""#));
    assert!(json.contains(r#""summary":"Done successfully""#));

    let back: SdkOutputEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(envelope, back);
}

// ---------------------------------------------------------------------------
// Message union (top-level tagged enum from messages.rs)
// ---------------------------------------------------------------------------

#[test]
fn message_user_text_roundtrip() {
    let msg = Message::User(UserMessage {
        content: UserContent::Text("Hello".into()),
        uuid: Some("uuid-1".into()),
        parent_tool_use_id: None,
        tool_use_result: None,
    });
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains(r#""type":"user""#));
    assert!(json.contains(r#""content":"Hello""#));

    let back: Message = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, back);
}

#[test]
fn message_assistant_roundtrip() {
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
    assert!(json.contains(r#""type":"assistant""#));

    let back: Message = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, back);
}

#[test]
fn message_result_roundtrip() {
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

// ---------------------------------------------------------------------------
// WireEncoder streaming
// ---------------------------------------------------------------------------

#[test]
fn wire_encoder_writes_init_to_buffer() {
    let mut buf = Vec::new();
    {
        let mut enc = WireEncoder::new(&mut buf);
        enc.write_init("s1", "claude-sonnet-4-6", &["Read".into()], "/tmp")
            .unwrap();
    }
    let output = String::from_utf8(buf).unwrap();
    assert!(output.ends_with('\n'));
    let v: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(v["type"], "system");
    assert_eq!(v["subtype"], "init");
}

#[test]
fn wire_encoder_writes_result_to_buffer() {
    let mut buf = Vec::new();
    {
        let mut enc = WireEncoder::new(&mut buf);
        enc.write_result(
            "success",
            100,
            80,
            false,
            1,
            "s",
            None,
            None,
            Some("ok".into()),
            None,
        )
        .unwrap();
    }
    let output = String::from_utf8(buf).unwrap();
    assert!(output.ends_with('\n'));
    let v: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(v["type"], "result");
    assert_eq!(v["result"], "ok");
}

#[test]
fn wire_decoder_parses_user_input() {
    let input = r#"{"type":"user","content":"hello"}"#;
    let mut dec = WireDecoder::new(input.as_bytes());
    let msg = dec.read_next().unwrap().unwrap();
    assert!(matches!(msg, sdk_types::wire::SdkInput::User { .. }));
}

// ---------------------------------------------------------------------------
// Full lifecycle sequence using Message enum
// ---------------------------------------------------------------------------

#[test]
fn full_message_lifecycle_sequence() {
    let messages: Vec<Message> = vec![
        // Assistant text
        Message::Assistant(AssistantMessage {
            content: vec![ContentBlock::Text(TextBlock {
                text: "I'll help you with that.".into(),
            })],
            model: "claude-sonnet-4-6".into(),
            parent_tool_use_id: None,
            error: None,
            usage: None,
            message_id: Some("msg_1".into()),
        }),
        // Assistant tool use
        Message::Assistant(AssistantMessage {
            content: vec![ContentBlock::ToolUse(ToolUseBlock {
                id: "tu_life".into(),
                name: "Read".into(),
                input: serde_json::json!({"file_path": "/project/main.rs"}),
            })],
            model: "claude-sonnet-4-6".into(),
            parent_tool_use_id: None,
            error: None,
            usage: None,
            message_id: Some("msg_2".into()),
        }),
        // Task started
        Message::TaskStarted(TaskStartedMessage {
            task_id: "task_life".into(),
            description: Some("Reading file".into()),
            uuid: Some("uuid_ts1".into()),
            session_id: Some("sess_life".into()),
            tool_use_id: Some("tu_life".into()),
            task_type: Some("local_bash".into()),
        }),
        // Task progress
        Message::TaskProgress(TaskProgressMessage {
            task_id: "task_life".into(),
            description: Some("Processing...".into()),
            usage: Some(TaskUsage {
                total_tokens: Some(50),
                tool_uses: Some(1),
                duration_ms: Some(500),
            }),
            uuid: Some("uuid_tp1".into()),
            session_id: Some("sess_life".into()),
        }),
        // Task notification
        Message::TaskNotification(TaskNotificationMessage {
            task_id: "task_life".into(),
            status: Some("completed".into()),
            output_file: None,
            summary: Some("File read successfully".into()),
            uuid: Some("uuid_tn1".into()),
            session_id: Some("sess_life".into()),
        }),
        // Rate limit
        Message::RateLimit(RateLimitEvent {
            rate_limit_info: RateLimitInfo {
                status: "allowed".into(),
                resets_at: None,
                rate_limit_type: None,
                utilization: None,
            },
            uuid: Some("uuid_rl1".into()),
            session_id: Some("sess_life".into()),
        }),
        // Stream event
        Message::StreamEvent(StreamEvent {
            uuid: "uuid_se1".into(),
            session_id: "sess_life".into(),
            event: serde_json::json!({"type": "content_block_delta"}),
            parent_tool_use_id: None,
        }),
        // Final result
        Message::Result(ResultMessage {
            subtype: "success".into(),
            duration_ms: 3000,
            duration_api_ms: 2500,
            is_error: false,
            num_turns: 2,
            session_id: "sess_life".into(),
            total_cost_usd: Some(0.05),
            usage: Some(serde_json::json!({
                "input_tokens": 500,
                "output_tokens": 200
            })),
            result: Some("Done!".into()),
            stop_reason: Some("end_turn".into()),
            structured_output: None,
            model_usage: None,
        }),
    ];

    // Round-trip every message
    for (i, msg) in messages.iter().enumerate() {
        let json = serde_json::to_string(msg).unwrap();
        let back: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, &back, "roundtrip failed at index {i}: {json}");
    }
}
