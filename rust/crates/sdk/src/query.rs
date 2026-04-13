//! One-shot query function.

use std::pin::Pin;

use futures::Stream;
use sdk_types::wire::SdkOutputEnvelope;

use crate::error::{ClaudeSDKError, CLIJSONDecodeError};
use crate::transport::SubprocessTransport;
use crate::Transport;

/// Input to a query: either a plain text string or a stream of JSON values.
pub enum PromptInput {
    /// A simple text prompt.
    Text(String),
    /// A streaming prompt (stream of JSON values).
    Stream(Pin<Box<dyn Stream<Item = serde_json::Value> + Send>>),
}

impl From<String> for PromptInput {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for PromptInput {
    fn from(s: &str) -> Self {
        Self::Text(s.to_owned())
    }
}

/// Send a one-shot query to Claude Code. Creates a new session each time.
///
/// Returns a stream of output envelopes from the CLI.
pub async fn query(
    prompt: impl Into<PromptInput>,
    cli_path: Option<String>,
) -> Result<Pin<Box<dyn Stream<Item = Result<SdkOutputEnvelope, ClaudeSDKError>> + Send>>, ClaudeSDKError>
{
    let mut transport = SubprocessTransport::new(cli_path);
    transport.connect().await?;

    match prompt.into() {
        PromptInput::Text(text) => {
            let input = serde_json::json!({
                "type": "user",
                "content": text,
            });
            let json_str = serde_json::to_string(&input).map_err(|e| {
                ClaudeSDKError::CLIJSONDecode(CLIJSONDecodeError {
                    line: input.to_string(),
                    original_error: e.to_string(),
                })
            })?;
            transport.write(&json_str).await?;
        }
        PromptInput::Stream(_) => {
            return Err(ClaudeSDKError::General(
                "Streaming prompts not yet supported in query()".into(),
            ));
        }
    }

    transport.end_input().await?;

    Ok(transport.read_messages())
}
