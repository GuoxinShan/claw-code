//! Custom tool helper for registering SDK-provided tools.

use std::future::Future;
use std::pin::Pin;

use serde_json::Value;

/// A tool registered by the SDK user, with a handler callback.
pub struct SdkTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub handler: Box<dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> + Send + Sync>,
}

/// Builder for constructing an SdkTool.
pub struct SdkToolBuilder {
    name: String,
    description: String,
    input_schema: Value,
}

impl SdkToolBuilder {
    /// Set the handler function for this tool.
    pub fn handler<F, Fut>(self, f: F) -> SdkTool
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Value, String>> + Send + 'static,
    {
        SdkTool {
            name: self.name,
            description: self.description,
            input_schema: self.input_schema,
            handler: Box::new(move |v| {
                let fut = f(v);
                Box::pin(fut)
            }),
        }
    }
}

/// Create a tool builder for registering a custom tool.
///
/// # Example
/// ```ignore
/// use claw_agent_sdk::tools::tool;
///
/// let my_tool = tool("my_tool", "Does something", serde_json::json!({"type": "object"}))
///     .handler(|input| async move { Ok(input) });
/// ```
pub fn tool(name: &str, description: &str, schema: Value) -> SdkToolBuilder {
    SdkToolBuilder {
        name: name.to_owned(),
        description: description.to_owned(),
        input_schema: schema,
    }
}
