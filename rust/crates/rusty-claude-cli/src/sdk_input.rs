use std::io::{BufRead, Lines};

use serde::Deserialize;
use serde_json::Value;

/// Input messages received from the SDK client over stdin JSON-lines.
#[derive(Debug, Clone, PartialEq)]
pub enum SdkInput {
    UserMessage { content: String },
    UserMessageBlocks { content: Vec<Value> },
    Command { command: String, params: Value },
    Resume { session_id: String },
    EndOfInput,
}

#[derive(Deserialize)]
struct RawSdkInput {
    #[serde(rename = "type")]
    input_type: String,
    #[serde(default)]
    content: Option<Value>,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    session_id: Option<String>,
}

/// Reads SDK wire-protocol JSON-lines from a buffered reader (typically stdin).
pub struct SdkInputReader<R: BufRead> {
    lines: Lines<R>,
}

impl<R: BufRead> SdkInputReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            lines: reader.lines(),
        }
    }

    /// Read the next input message from stdin.
    ///
    /// Returns `Ok(None)` when stdin is closed (EOF).
    /// Returns `Ok(Some(SdkInput::EndOfInput))` for blank lines, allowing
    /// the caller to decide whether to skip or treat as end-of-turn.
    pub fn read_next(&mut self) -> std::io::Result<Option<SdkInput>> {
        let line = match self.lines.next() {
            Some(Ok(line)) => line,
            Some(Err(e)) => return Err(e),
            None => return Ok(None),
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(Some(SdkInput::EndOfInput));
        }

        let raw: RawSdkInput = serde_json::from_str(trimmed)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        match raw.input_type.as_str() {
            "user" | "user_message" => {
                let content = raw.content.unwrap_or(Value::Null);
                match content {
                    Value::String(s) => Ok(Some(SdkInput::UserMessage { content: s })),
                    Value::Array(blocks) => Ok(Some(SdkInput::UserMessageBlocks {
                        content: blocks,
                    })),
                    other => Ok(Some(SdkInput::UserMessage {
                        content: other.to_string(),
                    })),
                }
            }
            "command" => {
                let command = raw.command.unwrap_or_default();
                let params = raw.content.unwrap_or(Value::Object(serde_json::Map::new()));
                Ok(Some(SdkInput::Command { command, params }))
            }
            "resume" => {
                let session_id = raw.session_id.unwrap_or_default();
                Ok(Some(SdkInput::Resume { session_id }))
            }
            _ => Ok(Some(SdkInput::EndOfInput)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SdkInput;
    use super::SdkInputReader;
    use std::io::BufReader;

    fn read_all(input: &str) -> Vec<SdkInput> {
        let reader = BufReader::new(input.as_bytes());
        let mut reader = SdkInputReader::new(reader);
        let mut results = Vec::new();
        while let Ok(Some(item)) = reader.read_next() {
            results.push(item);
        }
        results
    }

    #[test]
    fn parses_user_message_with_string_content() {
        let items = read_all(r#"{"type":"user","content":"hello"}"#);
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0],
            SdkInput::UserMessage {
                content: "hello".to_string()
            }
        );
    }

    #[test]
    fn parses_user_message_with_block_content() {
        let items =
            read_all(r#"{"type":"user","content":[{"type":"text","text":"hello"},{"type":"text","text":" world"}]}"#);
        assert_eq!(items.len(), 1);
        match &items[0] {
            SdkInput::UserMessageBlocks { content } => {
                assert_eq!(content.len(), 2);
                assert_eq!(content[0]["text"], "hello");
            }
            other => panic!("expected UserMessageBlocks, got {other:?}"),
        }
    }

    #[test]
    fn parses_user_message_legacy_type() {
        let items = read_all(r#"{"type":"user_message","content":"prompt text","session_id":"abc"}"#);
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0],
            SdkInput::UserMessage {
                content: "prompt text".to_string()
            }
        );
    }

    #[test]
    fn parses_command_message() {
        let items = read_all(r#"{"type":"command","command":"set_permission_mode","content":{"mode":"acceptEdits"}}"#);
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0],
            SdkInput::Command {
                command: "set_permission_mode".to_string(),
                params: serde_json::json!({"mode": "acceptEdits"}),
            }
        );
    }

    #[test]
    fn parses_resume_message() {
        let items = read_all(r#"{"type":"resume","session_id":"sess-123"}"#);
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0],
            SdkInput::Resume {
                session_id: "sess-123".to_string()
            }
        );
    }

    #[test]
    fn empty_lines_become_end_of_input() {
        let items = read_all("\n\n");
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0], SdkInput::EndOfInput));
    }

    #[test]
    fn eof_returns_none() {
        let reader = BufReader::new("".as_bytes());
        let mut reader = SdkInputReader::new(reader);
        assert!(reader.read_next().unwrap().is_none());
    }

    #[test]
    fn multiple_lines_in_sequence() {
        let input = concat!(
            r#"{"type":"user","content":"first"}"#, "\n",
            r#"{"type":"command","command":"interrupt"}"#, "\n",
            r#"{"type":"user","content":"second"}"#, "\n",
        );
        let items = read_all(input);
        assert_eq!(items.len(), 3);
        assert!(matches!(items[0], SdkInput::UserMessage { .. }));
        assert!(matches!(items[1], SdkInput::Command { .. }));
        assert!(matches!(items[2], SdkInput::UserMessage { .. }));
    }
}
