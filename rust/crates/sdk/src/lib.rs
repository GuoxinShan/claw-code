//! Claw Agent SDK - Rust client library for Claude Code.
//!
//! Mirrors the Python `claude-agent-sdk` API for programmatic access to Claude Code.

// Re-export all types from sdk-types at the crate root.
pub use sdk_types::*;

pub mod client;
pub mod error;
pub mod query;
pub mod session;
pub mod tools;
pub mod transport;

pub use client::ClaudeSDKClient;
pub use error::ClaudeSDKError;
pub use query::{query, PromptInput};
pub use session::{
    get_session_info, get_session_messages, list_sessions, rename_session, tag_session,
};
pub use tools::{tool, SdkTool, SdkToolBuilder};
pub use transport::{SubprocessTransport, Transport};
