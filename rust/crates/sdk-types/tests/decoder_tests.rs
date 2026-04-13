//! Input decoder tests: verify that JSON-lines input from the Python SDK
//! client deserializes correctly into SdkInput types, and that those
//! types serialize back to the same wire format.

use sdk_types::content::{ContentBlock, TextBlock};
use sdk_types::messages::UserContent;
use sdk_types::wire::{SdkInput, WireDecoder};

// ---------------------------------------------------------------------------
// User input
// ---------------------------------------------------------------------------

#[test]
fn decode_user_input_with_string_content() {
    let line = r#"{"type":"user","content":"Hello Claude"}"#;
    let mut dec = WireDecoder::new(line.as_bytes());
    let input = dec.read_next().unwrap().unwrap();
    match &input {
        SdkInput::User { content } => {
            assert_eq!(content, &UserContent::Text("Hello Claude".into()));
        }
        _ => panic!("Expected User variant, got {:?}", input),
    }
}

#[test]
fn decode_user_input_with_block_content() {
    let line = r#"{"type":"user","content":[{"type":"text","text":"Hello"}]}"#;
    let mut dec = WireDecoder::new(line.as_bytes());
    let input = dec.read_next().unwrap().unwrap();
    match &input {
        SdkInput::User { content } => {
            assert_eq!(
                content,
                &UserContent::Blocks(vec![ContentBlock::Text(TextBlock {
                    text: "Hello".into()
                })])
            );
        }
        _ => panic!("Expected User variant"),
    }
}

#[test]
fn user_input_roundtrip_string() {
    let input = SdkInput::User {
        content: UserContent::Text("Hello Claude".into()),
    };
    let json = serde_json::to_string(&input).unwrap();
    assert_eq!(json, r#"{"type":"user","content":"Hello Claude"}"#);

    let back: SdkInput = serde_json::from_str(&json).unwrap();
    assert_eq!(input, back);
}

#[test]
fn user_input_roundtrip_blocks() {
    let input = SdkInput::User {
        content: UserContent::Blocks(vec![ContentBlock::Text(TextBlock {
            text: "Hello".into(),
        })]),
    };
    let json = serde_json::to_string(&input).unwrap();
    assert!(json.starts_with(r#"{"type":"user""#));
    assert!(json.contains(r#""type":"text""#));

    let back: SdkInput = serde_json::from_str(&json).unwrap();
    assert_eq!(input, back);
}

// ---------------------------------------------------------------------------
// UserMessage (with session_id)
// ---------------------------------------------------------------------------

#[test]
fn decode_user_message_with_session() {
    let line = r#"{"type":"user_message","content":"Continue","session_id":"sess_001"}"#;
    let mut dec = WireDecoder::new(line.as_bytes());
    let input = dec.read_next().unwrap().unwrap();
    match &input {
        SdkInput::UserMessage {
            content,
            session_id,
        } => {
            assert_eq!(content, "Continue");
            assert_eq!(session_id.as_deref(), Some("sess_001"));
        }
        _ => panic!("Expected UserMessage variant"),
    }
}

#[test]
fn user_message_with_session_roundtrip() {
    let input = SdkInput::UserMessage {
        content: "Continue".into(),
        session_id: Some("sess_001".into()),
    };
    let json = serde_json::to_string(&input).unwrap();
    assert!(json.contains(r#""type":"user_message""#));
    assert!(json.contains(r#""content":"Continue""#));
    assert!(json.contains(r#""session_id":"sess_001""#));

    let back: SdkInput = serde_json::from_str(&json).unwrap();
    assert_eq!(input, back);
}

#[test]
fn user_message_without_session_id() {
    let input = SdkInput::UserMessage {
        content: "Hello".into(),
        session_id: None,
    };
    let json = serde_json::to_string(&input).unwrap();
    assert!(!json.contains("session_id"));
}

// ---------------------------------------------------------------------------
// Command messages (flat format matching Python SDK)
// ---------------------------------------------------------------------------

#[test]
fn decode_command_interrupt() {
    let line = r#"{"type":"command","command":"interrupt"}"#;
    let mut dec = WireDecoder::new(line.as_bytes());
    let input = dec.read_next().unwrap().unwrap();
    match &input {
        SdkInput::Command { command, .. } => {
            assert_eq!(command, "interrupt");
        }
        _ => panic!("Expected Command variant"),
    }
}

#[test]
fn decode_command_set_permission_mode() {
    let line = r#"{"type":"command","command":"set_permission_mode","mode":"acceptEdits"}"#;
    let mut dec = WireDecoder::new(line.as_bytes());
    let input = dec.read_next().unwrap().unwrap();
    match &input {
        SdkInput::Command {
            command, mode, ..
        } => {
            assert_eq!(command, "set_permission_mode");
            assert_eq!(mode.as_deref(), Some("acceptEdits"));
        }
        _ => panic!("Expected Command variant"),
    }
}

#[test]
fn decode_command_set_model() {
    let line = r#"{"type":"command","command":"set_model","model":"claude-sonnet-4-6"}"#;
    let mut dec = WireDecoder::new(line.as_bytes());
    let input = dec.read_next().unwrap().unwrap();
    match &input {
        SdkInput::Command {
            command, model, ..
        } => {
            assert_eq!(command, "set_model");
            assert_eq!(model.as_deref(), Some("claude-sonnet-4-6"));
        }
        _ => panic!("Expected Command variant"),
    }
}

#[test]
fn command_roundtrip() {
    let input = SdkInput::Command {
        command: "set_permission_mode".into(),
        mode: Some("acceptEdits".into()),
        model: None,
        user_message_id: None,
    };
    let json = serde_json::to_string(&input).unwrap();
    assert!(json.contains(r#""type":"command""#));
    assert!(json.contains(r#""command":"set_permission_mode""#));
    assert!(json.contains(r#""mode":"acceptEdits""#));

    let back: SdkInput = serde_json::from_str(&json).unwrap();
    assert_eq!(input, back);
}

// ---------------------------------------------------------------------------
// Resume
// ---------------------------------------------------------------------------

#[test]
fn decode_resume() {
    let line = r#"{"type":"resume","session_id":"sess_abc"}"#;
    let mut dec = WireDecoder::new(line.as_bytes());
    let input = dec.read_next().unwrap().unwrap();
    match &input {
        SdkInput::Resume { session_id } => {
            assert_eq!(session_id, "sess_abc");
        }
        _ => panic!("Expected Resume variant"),
    }
}

#[test]
fn resume_roundtrip() {
    let input = SdkInput::Resume {
        session_id: "sess_abc".into(),
    };
    let json = serde_json::to_string(&input).unwrap();
    assert!(json.contains(r#""type":"resume""#));
    assert!(json.contains(r#""session_id":"sess_abc""#));

    let back: SdkInput = serde_json::from_str(&json).unwrap();
    assert_eq!(input, back);
}

// ---------------------------------------------------------------------------
// Unknown type should fail
// ---------------------------------------------------------------------------

#[test]
fn unknown_type_returns_error() {
    let line = r#"{"type":"unknown_variant","data":123}"#;
    let result = serde_json::from_str::<SdkInput>(line);
    assert!(result.is_err(), "Should fail to parse unknown type");
}

// ---------------------------------------------------------------------------
// Input envelope discrimination
// ---------------------------------------------------------------------------

#[test]
fn all_input_types_are_correctly_tagged() {
    let cases: Vec<(&str, &str)> = vec![
        (r#"{"type":"user","content":"hi"}"#, "user"),
        (
            r#"{"type":"user_message","content":"hi","session_id":"s1"}"#,
            "user_message",
        ),
        (
            r#"{"type":"command","command":"interrupt"}"#,
            "command",
        ),
        (r#"{"type":"resume","session_id":"s2"}"#, "resume"),
    ];

    for (line, expected_type) in cases {
        let input: SdkInput = serde_json::from_str(line).unwrap();
        let json = serde_json::to_string(&input).unwrap();
        assert!(
            json.contains(&format!(r#""type":"{expected_type}""#)),
            "Expected type tag '{}' in: {}",
            expected_type,
            json
        );
    }
}

// ---------------------------------------------------------------------------
// WireDecoder streaming
// ---------------------------------------------------------------------------

#[test]
fn decoder_reads_multiple_lines() {
    let input = concat!(
        r#"{"type":"user","content":"hi"}"#,
        "\n",
        r#"{"type":"command","command":"interrupt"}"#,
        "\n",
    );
    let mut dec = WireDecoder::new(input.as_bytes());
    let first = dec.read_next().unwrap().unwrap();
    let second = dec.read_next().unwrap().unwrap();
    let third = dec.read_next().unwrap();
    assert!(matches!(first, SdkInput::User { .. }));
    assert!(matches!(second, SdkInput::Command { .. }));
    assert!(third.is_none());
}

#[test]
fn decoder_skips_blank_lines() {
    let input = "\n\n{\"type\":\"command\",\"command\":\"interrupt\"}\n\n";
    let mut dec = WireDecoder::new(input.as_bytes());
    let msg = dec.read_next().unwrap().unwrap();
    match &msg {
        SdkInput::Command { command, .. } => {
            assert_eq!(command, "interrupt");
        }
        _ => panic!("Expected Command variant"),
    }
    assert!(dec.read_next().unwrap().is_none());
}
