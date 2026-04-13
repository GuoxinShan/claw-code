use std::io::{BufRead, Write};

use serde::{Deserialize, Serialize};

use crate::content::ContentBlock;
use crate::messages::{RateLimitInfo, UserContent};

// ---------------------------------------------------------------------------
// SDK input messages (client -> CLI via stdin)
// ---------------------------------------------------------------------------

/// A message sent from the SDK client to the CLI over stdin.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SdkInput {
    #[serde(rename = "user")]
    User { content: UserContent },

    #[serde(rename = "user_message")]
    UserMessage {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
    },

    #[serde(rename = "command")]
    Command {
        command: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mode: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        user_message_id: Option<String>,
    },

    #[serde(rename = "resume")]
    Resume { session_id: String },
}

// ---------------------------------------------------------------------------
// SDK output envelope (CLI -> client via stdout)
// ---------------------------------------------------------------------------

/// Wrapper for messages written to stdout, adding a type discriminator.
///
/// All field names use snake_case to match the Python SDK wire format exactly.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SdkOutputEnvelope {
    #[serde(rename = "system")]
    System {
        subtype: String,
        session_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tools: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cwd: Option<String>,
    },

    #[serde(rename = "assistant")]
    Assistant {
        message: AssistantEnvelope,
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uuid: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        message_id: Option<String>,
    },

    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(default)]
        is_error: bool,
    },

    #[serde(rename = "result")]
    Result {
        subtype: String,
        duration_ms: u64,
        duration_api_ms: u64,
        is_error: bool,
        num_turns: u32,
        session_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        total_cost_usd: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        usage: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        stop_reason: Option<String>,
    },

    #[serde(rename = "stream_event")]
    StreamEvent {
        uuid: String,
        session_id: String,
        event: serde_json::Value,
    },

    #[serde(rename = "rate_limit")]
    RateLimit {
        rate_limit_info: RateLimitInfo,
        uuid: String,
        session_id: String,
    },

    #[serde(rename = "task_started")]
    TaskStarted {
        task_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        uuid: String,
        session_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_use_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        task_type: Option<String>,
    },

    #[serde(rename = "task_progress")]
    TaskProgress {
        task_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        usage: Option<serde_json::Value>,
        uuid: String,
        session_id: String,
    },

    #[serde(rename = "task_notification")]
    TaskNotification {
        task_id: String,
        status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        output_file: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        summary: Option<String>,
        uuid: String,
        session_id: String,
    },
}

/// The `message` field inside an `assistant` envelope.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AssistantEnvelope {
    pub role: String,
    pub content: Vec<ContentBlock>,
}

impl AssistantEnvelope {
    pub fn new(content: Vec<ContentBlock>) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
        }
    }
}

// ---------------------------------------------------------------------------
// WireEncoder -- writes output messages as JSON-lines to a Write sink
// ---------------------------------------------------------------------------

/// Encodes output messages as JSON-lines to a writer (typically stdout).
///
/// Each call to a `write_*` method produces exactly one line of compact JSON
/// followed by a newline (`\n`), matching the Python SDK wire format.
pub struct WireEncoder<W: Write> {
    writer: W,
}

impl<W: Write> WireEncoder<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Unwrap the encoder, returning the underlying writer.
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Write a raw JSON value as a single line.
    pub fn write_line(&mut self, json: &serde_json::Value) -> Result<(), std::io::Error> {
        let compact = serde_json::to_string(json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        writeln!(self.writer, "{compact}")
    }

    /// Write the `system/init` message at session start.
    pub fn write_init(
        &mut self,
        session_id: &str,
        model: &str,
        tools: &[String],
        cwd: &str,
    ) -> Result<(), std::io::Error> {
        self.write_envelope(&SdkOutputEnvelope::System {
            subtype: "init".to_owned(),
            session_id: session_id.to_owned(),
            tools: Some(tools.to_vec()),
            model: Some(model.to_owned()),
            cwd: Some(cwd.to_owned()),
        })
    }

    /// Write an assistant message with content blocks.
    pub fn write_assistant(
        &mut self,
        content: Vec<ContentBlock>,
        model: &str,
        session_id: &str,
        uuid: &str,
        message_id: &str,
    ) -> Result<(), std::io::Error> {
        self.write_envelope(&SdkOutputEnvelope::Assistant {
            message: AssistantEnvelope::new(content),
            model: Some(model.to_owned()),
            session_id: Some(session_id.to_owned()),
            uuid: Some(uuid.to_owned()),
            message_id: Some(message_id.to_owned()),
        })
    }

    /// Write a tool result message.
    pub fn write_tool_result(
        &mut self,
        tool_use_id: &str,
        content: &str,
        is_error: bool,
    ) -> Result<(), std::io::Error> {
        self.write_envelope(&SdkOutputEnvelope::ToolResult {
            tool_use_id: tool_use_id.to_owned(),
            content: content.to_owned(),
            is_error,
        })
    }

    /// Write the final result message.
    #[allow(clippy::too_many_arguments)]
    pub fn write_result(
        &mut self,
        subtype: &str,
        duration_ms: u64,
        duration_api_ms: u64,
        is_error: bool,
        num_turns: u32,
        session_id: &str,
        cost_usd: Option<f64>,
        usage: Option<serde_json::Value>,
        result: Option<String>,
        stop_reason: Option<String>,
    ) -> Result<(), std::io::Error> {
        self.write_envelope(&SdkOutputEnvelope::Result {
            subtype: subtype.to_owned(),
            duration_ms,
            duration_api_ms,
            is_error,
            num_turns,
            session_id: session_id.to_owned(),
            total_cost_usd: cost_usd,
            usage,
            result,
            stop_reason,
        })
    }

    /// Write a streaming event.
    pub fn write_stream_event(
        &mut self,
        uuid: &str,
        session_id: &str,
        event: serde_json::Value,
    ) -> Result<(), std::io::Error> {
        self.write_envelope(&SdkOutputEnvelope::StreamEvent {
            uuid: uuid.to_owned(),
            session_id: session_id.to_owned(),
            event,
        })
    }

    /// Write a rate-limit event.
    pub fn write_rate_limit(
        &mut self,
        rate_limit_info: RateLimitInfo,
        uuid: &str,
        session_id: &str,
    ) -> Result<(), std::io::Error> {
        self.write_envelope(&SdkOutputEnvelope::RateLimit {
            rate_limit_info,
            uuid: uuid.to_owned(),
            session_id: session_id.to_owned(),
        })
    }

    /// Write a task-started lifecycle message.
    pub fn write_task_started(
        &mut self,
        task_id: &str,
        description: &str,
        uuid: &str,
        session_id: &str,
        tool_use_id: Option<&str>,
        task_type: Option<&str>,
    ) -> Result<(), std::io::Error> {
        self.write_envelope(&SdkOutputEnvelope::TaskStarted {
            task_id: task_id.to_owned(),
            description: Some(description.to_owned()),
            uuid: uuid.to_owned(),
            session_id: session_id.to_owned(),
            tool_use_id: tool_use_id.map(|s| s.to_owned()),
            task_type: task_type.map(|s| s.to_owned()),
        })
    }

    /// Write a task-progress lifecycle message.
    pub fn write_task_progress(
        &mut self,
        task_id: &str,
        description: &str,
        usage: serde_json::Value,
        uuid: &str,
        session_id: &str,
    ) -> Result<(), std::io::Error> {
        self.write_envelope(&SdkOutputEnvelope::TaskProgress {
            task_id: task_id.to_owned(),
            description: Some(description.to_owned()),
            usage: Some(usage),
            uuid: uuid.to_owned(),
            session_id: session_id.to_owned(),
        })
    }

    /// Write a task-notification lifecycle message.
    pub fn write_task_notification(
        &mut self,
        task_id: &str,
        status: &str,
        output_file: &str,
        summary: &str,
        uuid: &str,
        session_id: &str,
    ) -> Result<(), std::io::Error> {
        self.write_envelope(&SdkOutputEnvelope::TaskNotification {
            task_id: task_id.to_owned(),
            status: status.to_owned(),
            output_file: Some(output_file.to_owned()),
            summary: Some(summary.to_owned()),
            uuid: uuid.to_owned(),
            session_id: session_id.to_owned(),
        })
    }

    /// Serialize an `SdkOutputEnvelope` and write it as one JSON-line.
    fn write_envelope(&mut self, envelope: &SdkOutputEnvelope) -> Result<(), std::io::Error> {
        let compact = serde_json::to_string(envelope)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        writeln!(self.writer, "{compact}")
    }
}

// ---------------------------------------------------------------------------
// WireDecoder -- reads input messages from JSON-lines on a BufRead source
// ---------------------------------------------------------------------------

/// Decodes input messages from JSON-lines on a buffered reader (typically stdin).
pub struct WireDecoder<R: BufRead> {
    reader: R,
}

impl<R: BufRead> WireDecoder<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    /// Unwrap the decoder, returning the underlying reader.
    pub fn into_inner(self) -> R {
        self.reader
    }

    /// Read the next input message.
    ///
    /// Returns `Ok(None)` when the reader reaches EOF.
    /// Blank lines are skipped silently.
    pub fn read_next(&mut self) -> Result<Option<SdkInput>, WireDecodeError> {
        loop {
            let mut line = String::new();
            let bytes = self
                .reader
                .read_line(&mut line)
                .map_err(WireDecodeError::Io)?;
            if bytes == 0 {
                return Ok(None);
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let msg: SdkInput = serde_json::from_str(trimmed).map_err(|e| WireDecodeError::Parse {
                line: trimmed.to_owned(),
                source: e,
            })?;
            return Ok(Some(msg));
        }
    }
}

// ---------------------------------------------------------------------------
// Decode errors
// ---------------------------------------------------------------------------

/// Errors that can occur while decoding wire input.
#[derive(Debug)]
pub enum WireDecodeError {
    /// An I/O error reading from the underlying stream.
    Io(std::io::Error),
    /// The line could not be parsed as a valid `SdkInput`.
    Parse {
        line: String,
        source: serde_json::Error,
    },
}

impl std::fmt::Display for WireDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Parse { line, source } => {
                write!(f, "parse error on line {line:?}: {source}")
            }
        }
    }
}

impl std::error::Error for WireDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Parse { source, .. } => Some(source),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::TextBlock;
    use serde_json::json;

    // ---- Encoder tests ----

    #[test]
    fn encode_init() {
        let mut buf = Vec::new();
        {
            let mut enc = WireEncoder::new(&mut buf);
            enc.write_init(
                "sess1",
                "claude-sonnet-4-6",
                &["Read".into(), "Write".into()],
                "/home/user/project",
            )
            .unwrap();
        }
        let line = String::from_utf8(buf).unwrap();
        assert!(line.ends_with('\n'));
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["type"], "system");
        assert_eq!(v["subtype"], "init");
        assert_eq!(v["session_id"], "sess1");
        assert_eq!(v["model"], "claude-sonnet-4-6");
        assert_eq!(v["cwd"], "/home/user/project");
        assert_eq!(v["tools"], json!(["Read", "Write"]));
    }

    #[test]
    fn encode_assistant() {
        let mut buf = Vec::new();
        {
            let mut enc = WireEncoder::new(&mut buf);
            enc.write_assistant(
                vec![ContentBlock::Text(TextBlock {
                    text: "Hello!".into(),
                })],
                "claude-sonnet-4-6",
                "sess1",
                "uuid-123",
                "msg_456",
            )
            .unwrap();
        }
        let line = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["type"], "assistant");
        assert_eq!(v["message"]["role"], "assistant");
        assert_eq!(v["message"]["content"][0]["type"], "text");
        assert_eq!(v["message"]["content"][0]["text"], "Hello!");
        assert_eq!(v["model"], "claude-sonnet-4-6");
        assert_eq!(v["session_id"], "sess1");
        assert_eq!(v["uuid"], "uuid-123");
        assert_eq!(v["message_id"], "msg_456");
    }

    #[test]
    fn encode_tool_result() {
        let mut buf = Vec::new();
        {
            let mut enc = WireEncoder::new(&mut buf);
            enc.write_tool_result("tu_789", "file contents here", false)
                .unwrap();
        }
        let line = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["type"], "tool_result");
        assert_eq!(v["tool_use_id"], "tu_789");
        assert_eq!(v["content"], "file contents here");
        assert_eq!(v["is_error"], false);
    }

    #[test]
    fn encode_result_full() {
        let mut buf = Vec::new();
        {
            let mut enc = WireEncoder::new(&mut buf);
            enc.write_result(
                "success",
                1500,
                1200,
                false,
                1,
                "sess1",
                Some(0.01),
                Some(json!({"input_tokens": 100, "output_tokens": 50})),
                Some("Done!".into()),
                Some("end_turn".into()),
            )
            .unwrap();
        }
        let line = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["type"], "result");
        assert_eq!(v["subtype"], "success");
        assert_eq!(v["duration_ms"], 1500);
        assert_eq!(v["duration_api_ms"], 1200);
        assert_eq!(v["is_error"], false);
        assert_eq!(v["num_turns"], 1);
        assert_eq!(v["session_id"], "sess1");
        assert!(v.get("total_cost_usd").is_some());
        assert!(v.get("usage").is_some());
        assert_eq!(v["result"], "Done!");
        assert_eq!(v["stop_reason"], "end_turn");
    }

    #[test]
    fn encode_result_minimal() {
        let mut buf = Vec::new();
        {
            let mut enc = WireEncoder::new(&mut buf);
            enc.write_result("error", 100, 80, true, 2, "sess2", None, None, None, None)
                .unwrap();
        }
        let line = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["type"], "result");
        assert_eq!(v["is_error"], true);
        assert!(v.get("total_cost_usd").is_none());
        assert!(v.get("usage").is_none());
        assert!(v.get("result").is_none());
        assert!(v.get("stop_reason").is_none());
    }

    #[test]
    fn encode_stream_event() {
        let mut buf = Vec::new();
        {
            let mut enc = WireEncoder::new(&mut buf);
            enc.write_stream_event("uuid-1", "sess1", json!({"delta": "Hi"}))
                .unwrap();
        }
        let line = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["type"], "stream_event");
        assert_eq!(v["uuid"], "uuid-1");
        assert_eq!(v["session_id"], "sess1");
        assert_eq!(v["event"]["delta"], "Hi");
    }

    #[test]
    fn encode_rate_limit() {
        let mut buf = Vec::new();
        {
            let mut enc = WireEncoder::new(&mut buf);
            enc.write_rate_limit(
                RateLimitInfo {
                    status: "allowed".into(),
                    resets_at: None,
                    rate_limit_type: None,
                    utilization: None,
                },
                "uuid-2",
                "sess1",
            )
            .unwrap();
        }
        let line = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["type"], "rate_limit");
        assert_eq!(v["rate_limit_info"]["status"], "allowed");
    }

    #[test]
    fn encode_task_started() {
        let mut buf = Vec::new();
        {
            let mut enc = WireEncoder::new(&mut buf);
            enc.write_task_started(
                "task-1",
                "running bash",
                "uuid-3",
                "sess1",
                Some("tu_001"),
                Some("local_bash"),
            )
            .unwrap();
        }
        let line = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["type"], "task_started");
        assert_eq!(v["task_id"], "task-1");
        assert_eq!(v["description"], "running bash");
        assert_eq!(v["tool_use_id"], "tu_001");
        assert_eq!(v["task_type"], "local_bash");
    }

    #[test]
    fn encode_task_progress() {
        let mut buf = Vec::new();
        {
            let mut enc = WireEncoder::new(&mut buf);
            enc.write_task_progress(
                "task-1",
                "still running",
                json!({"total_tokens": 100, "tool_uses": 1, "duration_ms": 500}),
                "uuid-3",
                "sess1",
            )
            .unwrap();
        }
        let line = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["type"], "task_progress");
        assert_eq!(v["usage"]["total_tokens"], 100);
    }

    #[test]
    fn encode_task_notification() {
        let mut buf = Vec::new();
        {
            let mut enc = WireEncoder::new(&mut buf);
            enc.write_task_notification(
                "task-1",
                "completed",
                "/tmp/out.txt",
                "all done",
                "uuid-3",
                "sess1",
            )
            .unwrap();
        }
        let line = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["type"], "task_notification");
        assert_eq!(v["status"], "completed");
        assert_eq!(v["output_file"], "/tmp/out.txt");
        assert_eq!(v["summary"], "all done");
    }

    // ---- Decoder tests ----

    #[test]
    fn decode_user_text() {
        let input = r#"{"type":"user","content":"prompt text"}"#;
        let mut dec = WireDecoder::new(input.as_bytes());
        let msg = dec.read_next().unwrap().unwrap();
        match msg {
            SdkInput::User { content } => {
                assert_eq!(content, UserContent::Text("prompt text".into()));
            }
            _ => panic!("expected User variant, got {msg:?}"),
        }
    }

    #[test]
    fn decode_user_blocks() {
        let input = r#"{"type":"user","content":[{"type":"text","text":"hello"}]}"#;
        let mut dec = WireDecoder::new(input.as_bytes());
        let msg = dec.read_next().unwrap().unwrap();
        match msg {
            SdkInput::User { content } => {
                assert_eq!(
                    content,
                    UserContent::Blocks(vec![ContentBlock::Text(TextBlock {
                        text: "hello".into()
                    })])
                );
            }
            _ => panic!("expected User variant"),
        }
    }

    #[test]
    fn decode_user_message() {
        let input = r#"{"type":"user_message","content":"hi","session_id":"sess1"}"#;
        let mut dec = WireDecoder::new(input.as_bytes());
        let msg = dec.read_next().unwrap().unwrap();
        match msg {
            SdkInput::UserMessage {
                content,
                session_id,
            } => {
                assert_eq!(content, "hi");
                assert_eq!(session_id.as_deref(), Some("sess1"));
            }
            _ => panic!("expected UserMessage variant"),
        }
    }

    #[test]
    fn decode_command_set_permission_mode() {
        let input = r#"{"type":"command","command":"set_permission_mode","mode":"acceptEdits"}"#;
        let mut dec = WireDecoder::new(input.as_bytes());
        let msg = dec.read_next().unwrap().unwrap();
        match &msg {
            SdkInput::Command {
                command,
                mode,
                model,
                user_message_id,
            } => {
                assert_eq!(command, "set_permission_mode");
                assert_eq!(mode.as_deref(), Some("acceptEdits"));
                assert!(model.is_none());
                assert!(user_message_id.is_none());
            }
            _ => panic!("expected Command variant"),
        }
    }

    #[test]
    fn decode_command_interrupt() {
        let input = r#"{"type":"command","command":"interrupt"}"#;
        let mut dec = WireDecoder::new(input.as_bytes());
        let msg = dec.read_next().unwrap().unwrap();
        match &msg {
            SdkInput::Command {
                command,
                mode,
                model,
                user_message_id,
            } => {
                assert_eq!(command, "interrupt");
                assert!(mode.is_none());
                assert!(model.is_none());
                assert!(user_message_id.is_none());
            }
            _ => panic!("expected Command variant"),
        }
    }

    #[test]
    fn decode_command_rewind_files() {
        let input =
            r#"{"type":"command","command":"rewind_files","user_message_id":"msg_1"}"#;
        let mut dec = WireDecoder::new(input.as_bytes());
        let msg = dec.read_next().unwrap().unwrap();
        match &msg {
            SdkInput::Command {
                command,
                user_message_id,
                ..
            } => {
                assert_eq!(command, "rewind_files");
                assert_eq!(user_message_id.as_deref(), Some("msg_1"));
            }
            _ => panic!("expected Command variant"),
        }
    }

    #[test]
    fn decode_resume() {
        let input = r#"{"type":"resume","session_id":"old-sess"}"#;
        let mut dec = WireDecoder::new(input.as_bytes());
        let msg = dec.read_next().unwrap().unwrap();
        match msg {
            SdkInput::Resume { session_id } => {
                assert_eq!(session_id, "old-sess");
            }
            _ => panic!("expected Resume variant"),
        }
    }

    #[test]
    fn decode_eof() {
        let input = b"";
        let mut dec = WireDecoder::new(&input[..]);
        assert!(dec.read_next().unwrap().is_none());
    }

    #[test]
    fn decode_skips_blank_lines() {
        let input = "\n\n{\"type\":\"command\",\"command\":\"interrupt\"}\n\n";
        let mut dec = WireDecoder::new(input.as_bytes());
        let msg = dec.read_next().unwrap().unwrap();
        match &msg {
            SdkInput::Command { command, .. } => {
                assert_eq!(command, "interrupt");
            }
            _ => panic!("expected Command variant"),
        }
        assert!(dec.read_next().unwrap().is_none());
    }

    #[test]
    fn decode_multiple_messages() {
        let input = concat!(
            "{\"type\":\"user\",\"content\":\"hi\"}\n",
            "{\"type\":\"command\",\"command\":\"interrupt\"}\n",
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
    fn decode_invalid_json_returns_error() {
        let input = b"not json at all\n";
        let mut dec = WireDecoder::new(&input[..]);
        let result = dec.read_next();
        assert!(result.is_err());
    }

    // ---- Spec-format verification ----

    #[test]
    fn init_format_matches_spec() {
        let mut buf = Vec::new();
        {
            let mut enc = WireEncoder::new(&mut buf);
            enc.write_init(
                "abc",
                "claude-sonnet-4-6",
                &[
                    "Read".into(),
                    "Write".into(),
                    "Bash".into(),
                    "Edit".into(),
                    "Grep".into(),
                    "Glob".into(),
                ],
                "/home/user/project",
            )
            .unwrap();
        }
        let line = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["type"], "system");
        assert_eq!(v["subtype"], "init");
        assert_eq!(v["session_id"], "abc");
        assert_eq!(v["model"], "claude-sonnet-4-6");
        assert_eq!(v["cwd"], "/home/user/project");
    }

    #[test]
    fn result_format_matches_spec() {
        let mut buf = Vec::new();
        {
            let mut enc = WireEncoder::new(&mut buf);
            enc.write_result(
                "success",
                1500,
                1200,
                false,
                1,
                "abc",
                Some(0.01),
                Some(json!({"input_tokens":100,"output_tokens":50,"cache_creation_input_tokens":0,"cache_read_input_tokens":0})),
                Some("Hello!".into()),
                Some("end_turn".into()),
            )
            .unwrap();
        }
        let line = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["type"], "result");
        assert_eq!(v["subtype"], "success");
        assert_eq!(v["duration_ms"], 1500);
        assert_eq!(v["duration_api_ms"], 1200);
        assert_eq!(v["is_error"], false);
        assert_eq!(v["num_turns"], 1);
        assert_eq!(v["session_id"], "abc");
        assert_eq!(v["result"], "Hello!");
        assert_eq!(v["stop_reason"], "end_turn");
    }

    #[test]
    fn tool_result_uses_string_content() {
        // Verify tool_result serializes with string content (not array),
        // matching the Python SDK wire format.
        let mut buf = Vec::new();
        {
            let mut enc = WireEncoder::new(&mut buf);
            enc.write_tool_result("tu_001", "file contents", false).unwrap();
        }
        let line = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert!(v["content"].is_string());
        assert_eq!(v["content"], "file contents");
    }

    #[test]
    fn envelope_output_roundtrip() {
        let envelope = SdkOutputEnvelope::System {
            subtype: "init".into(),
            session_id: "sess-1".into(),
            tools: Some(vec!["Read".into(), "Write".into()]),
            model: Some("claude-sonnet-4-6".into()),
            cwd: Some("/tmp".into()),
        };
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains(r#""type":"system""#));
        assert!(json.contains(r#""session_id":"sess-1""#));
        let back: SdkOutputEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(envelope, back);
    }
}
