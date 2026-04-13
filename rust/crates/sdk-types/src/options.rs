use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::hooks::HookMatcher;
use crate::mcp::McpServerConfig;

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// PermissionMode
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PermissionMode {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "acceptEdits")]
    AcceptEdits,
    #[serde(rename = "plan")]
    Plan,
    #[serde(rename = "dontAsk")]
    DontAsk,
    #[serde(rename = "bypassPermissions")]
    BypassPermissions,
}

// ---------------------------------------------------------------------------
// ThinkingConfig
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ThinkingConfig {
    Adaptive,
    Enabled {
        #[serde(rename = "budgetTokens")]
        budget_tokens: u64,
    },
    Disabled,
}

// ---------------------------------------------------------------------------
// SettingSource
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SettingSource {
    User,
    Policy,
    Defaults,
}

// ---------------------------------------------------------------------------
// Preset types
// ---------------------------------------------------------------------------

/// A named system prompt preset.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SystemPromptPreset {
    pub name: String,
    pub prompt: String,
}

/// A named tools preset.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ToolsPreset {
    pub name: String,
    pub tools: Vec<String>,
}

// ---------------------------------------------------------------------------
// AgentDefinition
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentDefinition {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub tools: Option<Vec<String>>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub skills: Option<Vec<String>>,
    #[serde(default)]
    pub memory: Option<bool>,
    #[serde(default)]
    pub mcp_servers: Option<HashMap<String, McpServerConfig>>,
}

// ---------------------------------------------------------------------------
// ClaudeAgentOptions
// ---------------------------------------------------------------------------

/// Full set of configuration options for launching a Claude SDK session.
///
/// Field names use camelCase serialization to match the Python SDK wire format.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeAgentOptions {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub tools: Option<Vec<String>>,
    #[serde(default)]
    pub allowed_tools: Option<Vec<String>>,
    #[serde(default)]
    pub disallowed_tools: Option<Vec<String>>,
    #[serde(default)]
    pub permission_mode: Option<PermissionMode>,
    #[serde(default)]
    pub max_turns: Option<u32>,
    #[serde(default)]
    pub max_budget_usd: Option<f64>,
    #[serde(default)]
    pub mcp_servers: Option<HashMap<String, McpServerConfig>>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub hooks: Option<Vec<HookMatcher>>,
    /// Callback — not serializable. Ignored during serde.
    #[serde(default, skip)]
    pub can_use_tool: Option<()>,
    #[serde(default)]
    pub continue_conversation: Option<bool>,
    #[serde(default)]
    pub resume: Option<String>,
    #[serde(default)]
    pub output_format: Option<String>,
    #[serde(default)]
    pub thinking: Option<ThinkingConfig>,
    #[serde(default)]
    pub effort: Option<String>,
    #[serde(default)]
    pub agents: Option<Vec<AgentDefinition>>,
    #[serde(default)]
    pub plugins: Option<Vec<Value>>,
    #[serde(default)]
    pub sandbox: Option<bool>,
    #[serde(default)]
    pub setting_sources: Option<Vec<SettingSource>>,
    #[serde(default)]
    pub enable_file_checkpointing: Option<bool>,
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
    #[serde(default)]
    pub add_dirs: Option<Vec<String>>,
    #[serde(default)]
    pub fork_session: Option<String>,
    #[serde(default)]
    pub include_partial_messages: Option<bool>,
    #[serde(default)]
    pub max_buffer_size: Option<u64>,
    #[serde(default)]
    pub cli_path: Option<String>,
    #[serde(default)]
    pub settings: Option<Value>,
    #[serde(default)]
    pub extra_args: Option<Vec<String>>,
    #[serde(default)]
    pub betas: Option<Vec<String>>,
    #[serde(default)]
    pub fallback_model: Option<String>,
    #[serde(default)]
    pub permission_prompt_tool_name: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub stderr: Option<String>,
    #[serde(default)]
    pub max_thinking_tokens: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_mode_serialization() {
        let mode = PermissionMode::AcceptEdits;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, r#""acceptEdits""#);
        let back: PermissionMode = serde_json::from_str(&json).unwrap();
        assert_eq!(mode, back);
    }

    #[test]
    fn thinking_config_enabled_roundtrip() {
        let config = ThinkingConfig::Enabled { budget_tokens: 10_000 };
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, r#"{"type":"enabled","budgetTokens":10000}"#);
        let back: ThinkingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, back);
    }

    #[test]
    fn thinking_config_adaptive_roundtrip() {
        let config = ThinkingConfig::Adaptive;
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, r#"{"type":"adaptive"}"#);
        let back: ThinkingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, back);
    }

    #[test]
    fn options_roundtrip() {
        let opts = ClaudeAgentOptions {
            model: Some("claude-sonnet-4-6".into()),
            max_turns: Some(10),
            permission_mode: Some(PermissionMode::Default),
            cwd: Some("/tmp".into()),
            ..Default::default()
        };
        let json = serde_json::to_string(&opts).unwrap();
        let back: ClaudeAgentOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(opts.model, back.model);
        assert_eq!(opts.max_turns, back.max_turns);
        assert_eq!(opts.permission_mode, back.permission_mode);
        assert_eq!(opts.cwd, back.cwd);
    }
}
