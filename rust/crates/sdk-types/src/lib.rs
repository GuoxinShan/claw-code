//! SDK type definitions matching the Claude Code Agent SDK Python types.
//!
//! This crate provides the foundational types for the Agent SDK wire protocol.
//! All types support serde serialization/deserialization for JSON-lines communication.

pub mod content;
pub mod errors;
pub mod hooks;
pub mod mcp;
pub mod messages;
pub mod options;
pub mod permissions;
pub mod session;
pub mod wire;

// Re-export the most commonly used types at the crate root.
pub use content::{ContentBlock, TextBlock, ThinkingBlock, ToolResultBlock, ToolResultContent, ToolUseBlock};
pub use errors::{
    ClaudeSDKError, CLIConnectionError, CLIJSONDecodeError, CLINotFoundError, ProcessError,
};
pub use hooks::{
    BaseHookInput, HookEntry, HookEvent, HookInput, HookJSONOutput, HookMatcher,
    PermissionRequestHookInput, PreToolUseHookInput, PostToolUseHookInput,
    PostToolUseFailureHookInput, PreCompactHookInput, StopHookInput,
    SubagentStartHookInput, SubagentStopHookInput, UserPromptSubmitHookInput,
    NotificationHookInput,
};
pub use mcp::{
    McpHttpServerConfig, McpSdkServerConfig, McpServerConfig, McpServerStatus,
    McpSSEServerConfig, McpStatusResponse, McpStdioServerConfig, McpToolInfo,
};
pub use messages::{
    AssistantMessage, AssistantMessageError, Message, RateLimitEvent, RateLimitInfo,
    ResultMessage, StreamEvent, SystemMessage, TaskNotificationMessage, TaskProgressMessage,
    TaskStartedMessage, TaskUsage, UserContent, UserMessage,
};
pub use options::{
    AgentDefinition, ClaudeAgentOptions, PermissionMode, SettingSource, SystemPromptPreset,
    ThinkingConfig, ToolsPreset,
};
pub use permissions::{
    PermissionResult, PermissionResultAllow, PermissionResultDeny, PermissionRuleValue,
    PermissionUpdate, ToolPermissionContext,
};
pub use session::{SDKSessionInfo, SdkMcpTool, SessionMessage};
pub use wire::{
    AssistantEnvelope, SdkInput, SdkOutputEnvelope, WireDecoder, WireDecodeError, WireEncoder,
};
