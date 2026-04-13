use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A content block within a message.
///
/// Uses `#[serde(tag = "type")]` for JSON discrimination matching the wire protocol.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text(TextBlock),
    Thinking(ThinkingBlock),
    ToolUse(ToolUseBlock),
    ToolResult(ToolResultBlock),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TextBlock {
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ThinkingBlock {
    pub thinking: String,
    pub signature: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ToolUseBlock {
    pub id: String,
    pub name: String,
    pub input: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ToolResultBlock {
    pub tool_use_id: String,
    pub content: ToolResultContent,
    #[serde(default)]
    pub is_error: bool,
}

/// Content of a tool result can be a plain string or a list of content blocks.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ToolResultContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

impl ContentBlock {
    /// Returns the text content if this is a TextBlock, otherwise None.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(block) => Some(&block.text),
            _ => None,
        }
    }

    /// Returns the tool name if this is a ToolUseBlock, otherwise None.
    pub fn as_tool_use(&self) -> Option<&str> {
        match self {
            Self::ToolUse(block) => Some(&block.name),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_block_roundtrip() {
        let block = ContentBlock::Text(TextBlock {
            text: "hello".into(),
        });
        let json = serde_json::to_string(&block).unwrap();
        assert_eq!(json, r#"{"type":"text","text":"hello"}"#);
        let back: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(block, back);
    }

    #[test]
    fn thinking_block_roundtrip() {
        let block = ContentBlock::Thinking(ThinkingBlock {
            thinking: "pondering".into(),
            signature: "sig123".into(),
        });
        let json = serde_json::to_string(&block).unwrap();
        assert_eq!(json, r#"{"type":"thinking","thinking":"pondering","signature":"sig123"}"#);
        let back: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(block, back);
    }

    #[test]
    fn tool_use_block_roundtrip() {
        let block = ContentBlock::ToolUse(ToolUseBlock {
            id: "id_1".into(),
            name: "Read".into(),
            input: serde_json::json!({"file_path": "/tmp/test.rs"}),
        });
        let json = serde_json::to_string(&block).unwrap();
        assert_eq!(
            json,
            r#"{"type":"tool_use","id":"id_1","name":"Read","input":{"file_path":"/tmp/test.rs"}}"#
        );
        let back: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(block, back);
    }

    #[test]
    fn tool_result_block_roundtrip() {
        let block = ContentBlock::ToolResult(ToolResultBlock {
            tool_use_id: "id_1".into(),
            content: ToolResultContent::Text("result text".into()),
            is_error: false,
        });
        let json = serde_json::to_string(&block).unwrap();
        assert_eq!(
            json,
            r#"{"type":"tool_result","tool_use_id":"id_1","content":"result text","is_error":false}"#
        );
        let back: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(block, back);
    }
}
