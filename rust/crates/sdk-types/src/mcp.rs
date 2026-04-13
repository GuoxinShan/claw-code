use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::collections::HashMap;

/// Configuration for an MCP server connection.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpServerConfig {
    Stdio(McpStdioServerConfig),
    Sse(McpSSEServerConfig),
    Http(McpHttpServerConfig),
    #[serde(rename = "sdk")]
    Sdk(McpSdkServerConfig),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct McpStdioServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct McpSSEServerConfig {
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct McpHttpServerConfig {
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

/// An SDK-registered MCP server (not a subprocess — the handler lives in-process).
///
/// The `instance` field cannot be serialized across process boundaries; it is
/// represented as a unit placeholder in the type system so the struct compiles
/// and participates in the enum, while actual dispatch is handled at runtime.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct McpSdkServerConfig {
    pub name: String,
}

/// Status response from an MCP server.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct McpStatusResponse {
    pub servers: Vec<McpServerStatus>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct McpServerStatus {
    pub name: String,
    pub status: String,
    pub tools: Vec<McpToolInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct McpToolInfo {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub input_schema: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdio_config_roundtrip() {
        let config = McpServerConfig::Stdio(McpStdioServerConfig {
            command: "npx".into(),
            args: vec!["-y".into(), "@modelcontextprotocol/server".into()],
            env: HashMap::new(),
        });
        let json = serde_json::to_string(&config).unwrap();
        let back: McpServerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, back);
    }

    #[test]
    fn sse_config_roundtrip() {
        let config = McpServerConfig::Sse(McpSSEServerConfig {
            url: "http://localhost:3000/sse".into(),
            headers: HashMap::new(),
        });
        let json = serde_json::to_string(&config).unwrap();
        let back: McpServerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, back);
    }
}
