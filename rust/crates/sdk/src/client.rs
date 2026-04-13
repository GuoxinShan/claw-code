//! ClaudeSDKClient - the main client for interacting with Claude Code.

use std::pin::Pin;

use futures::{Stream, StreamExt};
use sdk_types::options::ClaudeAgentOptions;
use sdk_types::wire::SdkOutputEnvelope;

use crate::error::ClaudeSDKError;
use crate::query::PromptInput;
use crate::transport::SubprocessTransport;
use crate::Transport;

/// The main client for interacting with Claude Code via the SDK wire protocol.
pub struct ClaudeSDKClient {
    transport: Box<dyn Transport>,
    options: ClaudeAgentOptions,
    session_id: Option<String>,
}

impl ClaudeSDKClient {
    /// Create a new client with default SubprocessTransport.
    pub fn new(options: Option<ClaudeAgentOptions>) -> Self {
        let options = options.unwrap_or_else(|| ClaudeAgentOptions {
            model: None,
            system_prompt: None,
            tools: None,
            allowed_tools: None,
            disallowed_tools: None,
            permission_mode: None,
            max_turns: None,
            max_budget_usd: None,
            mcp_servers: None,
            cwd: None,
            hooks: None,
            can_use_tool: None,
            continue_conversation: None,
            resume: None,
            output_format: None,
            thinking: None,
            effort: None,
            agents: None,
            plugins: None,
            sandbox: None,
            setting_sources: None,
            enable_file_checkpointing: None,
            env: None,
            add_dirs: None,
            fork_session: None,
            include_partial_messages: None,
            max_buffer_size: None,
            cli_path: None,
            settings: None,
            extra_args: None,
            betas: None,
            fallback_model: None,
            permission_prompt_tool_name: None,
            user: None,
            stderr: None,
            max_thinking_tokens: None,
        });
        let cli_path = options.cli_path.clone();
        let transport = SubprocessTransport::new(cli_path);
        Self::with_transport(Box::new(transport), options)
    }

    /// Create a client with a custom transport.
    pub fn with_transport(
        transport: Box<dyn Transport>,
        options: ClaudeAgentOptions,
    ) -> Self {
        Self {
            transport,
            options,
            session_id: None,
        }
    }

    /// Connect to the CLI, optionally sending an initial prompt.
    pub async fn connect(&mut self, prompt: Option<PromptInput>) -> Result<(), ClaudeSDKError> {
        self.transport.connect().await?;

        if let Some(p) = prompt {
            self.query(p).await?;
        }

        Ok(())
    }

    /// Send a query (user message) to the CLI.
    pub async fn query(&mut self, prompt: impl Into<PromptInput>) -> Result<(), ClaudeSDKError> {
        let input = match prompt.into() {
            PromptInput::Text(text) => serde_json::json!({
                "type": "user",
                "content": text,
            }),
            PromptInput::Stream(_) => {
                return Err(ClaudeSDKError::General(
                    "Streaming prompts not yet supported".into(),
                ));
            }
        };
        let json_str = serde_json::to_string(&input).map_err(|e| {
            ClaudeSDKError::CLIJSONDecode(crate::error::CLIJSONDecodeError {
                line: input.to_string(),
                original_error: e.to_string(),
            })
        })?;
        self.transport.write(&json_str).await
    }

    /// Receive all messages as a stream.
    pub fn receive_messages(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<SdkOutputEnvelope, ClaudeSDKError>> + Send>> {
        self.transport.read_messages()
    }

    /// Receive messages, filtering to only assistant and result messages.
    pub fn receive_response(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<SdkOutputEnvelope, ClaudeSDKError>> + Send>> {
        let stream = self.transport.read_messages();
        Box::pin(stream.filter(|msg| {
            let keep = match msg {
                Ok(SdkOutputEnvelope::Assistant { .. }) => true,
                Ok(SdkOutputEnvelope::Result { .. }) => true,
                _ => false,
            };
            futures::future::ready(keep)
        }))
    }

    /// Send an interrupt command.
    pub async fn interrupt(&mut self) -> Result<(), ClaudeSDKError> {
        let cmd = serde_json::json!({
            "type": "command",
            "command": "interrupt"
        });
        let json_str = serde_json::to_string(&cmd).map_err(|e| {
            ClaudeSDKError::CLIJSONDecode(crate::error::CLIJSONDecodeError {
                line: cmd.to_string(),
                original_error: e.to_string(),
            })
        })?;
        self.transport.write(&json_str).await
    }

    /// Set the permission mode.
    pub async fn set_permission_mode(&mut self, mode: &str) -> Result<(), ClaudeSDKError> {
        let cmd = serde_json::json!({
            "type": "command",
            "command": "set_permission_mode",
            "mode": mode
        });
        let json_str = serde_json::to_string(&cmd).map_err(|e| {
            ClaudeSDKError::CLIJSONDecode(crate::error::CLIJSONDecodeError {
                line: cmd.to_string(),
                original_error: e.to_string(),
            })
        })?;
        self.transport.write(&json_str).await
    }

    /// Set the model.
    pub async fn set_model(&mut self, model: Option<&str>) -> Result<(), ClaudeSDKError> {
        let mut cmd = serde_json::json!({
            "type": "command",
            "command": "set_model"
        });
        if let Some(m) = model {
            cmd["model"] = serde_json::Value::String(m.to_owned());
        }
        let json_str = serde_json::to_string(&cmd).map_err(|e| {
            ClaudeSDKError::CLIJSONDecode(crate::error::CLIJSONDecodeError {
                line: cmd.to_string(),
                original_error: e.to_string(),
            })
        })?;
        self.transport.write(&json_str).await
    }

    /// Disconnect from the CLI.
    pub async fn disconnect(&mut self) -> Result<(), ClaudeSDKError> {
        self.transport.close().await
    }

    /// Get MCP server status.
    /// TODO: implement via CLI command once wire protocol supports it.
    pub async fn get_mcp_status(
        &mut self,
    ) -> Result<sdk_types::mcp::McpStatusResponse, ClaudeSDKError> {
        Err(ClaudeSDKError::General(
            "get_mcp_status not yet implemented".into(),
        ))
    }

    /// Reconnect a specific MCP server.
    /// TODO: implement via CLI command once wire protocol supports it.
    pub async fn reconnect_mcp_server(&mut self, _server_name: &str) -> Result<(), ClaudeSDKError> {
        Err(ClaudeSDKError::General(
            "reconnect_mcp_server not yet implemented".into(),
        ))
    }

    /// Toggle an MCP server on or off.
    /// TODO: implement via CLI command once wire protocol supports it.
    pub async fn toggle_mcp_server(
        &mut self,
        _server_name: &str,
        _enabled: bool,
    ) -> Result<(), ClaudeSDKError> {
        Err(ClaudeSDKError::General(
            "toggle_mcp_server not yet implemented".into(),
        ))
    }

    /// Get the current session ID, if known.
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }
}

impl Drop for ClaudeSDKClient {
    fn drop(&mut self) {
        // Best-effort cleanup. We can't do async in drop, so we just
        // drop the transport which closes handles.
        // For proper cleanup, call disconnect() before dropping.
    }
}
