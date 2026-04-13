use std::fmt;

/// Base error type for SDK operations.
#[derive(Debug)]
pub enum ClaudeSDKError {
    /// The Claude CLI binary could not be found.
    CLINotFound(CLINotFoundError),
    /// Failed to connect to the CLI subprocess.
    CLIConnection(CLIConnectionError),
    /// The CLI process exited with a non-zero code.
    Process(ProcessError),
    /// Failed to decode JSON from the CLI output.
    CLIJSONDecode(CLIJSONDecodeError),
    /// A generic SDK error with a message.
    General(String),
}

impl std::error::Error for ClaudeSDKError {}

impl fmt::Display for ClaudeSDKError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CLINotFound(e) => write!(f, "CLI not found: {e}"),
            Self::CLIConnection(e) => write!(f, "CLI connection error: {e}"),
            Self::Process(e) => write!(f, "Process error: {e}"),
            Self::CLIJSONDecode(e) => write!(f, "JSON decode error: {e}"),
            Self::General(msg) => write!(f, "{msg}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CLINotFoundError {
    pub cli_path: Option<String>,
}

impl fmt::Display for CLINotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.cli_path {
            Some(path) => write!(f, "CLI not found at path: {path}"),
            None => write!(f, "CLI not found on PATH"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CLIConnectionError {
    pub message: String,
}

impl fmt::Display for CLIConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Clone, Debug)]
pub struct ProcessError {
    pub exit_code: Option<i32>,
    pub stderr: String,
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.exit_code {
            Some(code) => write!(f, "Process exited with code {code}: {}", self.stderr),
            None => write!(f, "Process failed: {}", self.stderr),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CLIJSONDecodeError {
    pub line: String,
    pub original_error: String,
}

impl fmt::Display for CLIJSONDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Failed to decode JSON line: {} (error: {})",
            self.line, self.original_error
        )
    }
}
