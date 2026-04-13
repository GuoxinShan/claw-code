//! Transport abstraction for communicating with the CLI subprocess.

use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use sdk_types::wire::SdkOutputEnvelope;

use crate::error::{ClaudeSDKError, CLIConnectionError, CLINotFoundError, CLIJSONDecodeError};

/// Transport trait for abstracting CLI communication.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Connect to the CLI (spawn the subprocess).
    async fn connect(&mut self) -> Result<(), ClaudeSDKError>;

    /// Write a JSON-line to the CLI's stdin.
    async fn write(&mut self, data: &str) -> Result<(), ClaudeSDKError>;

    /// Read messages from the CLI's stdout as a stream.
    fn read_messages(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<SdkOutputEnvelope, ClaudeSDKError>> + Send>>;

    /// Close the transport (kill the subprocess).
    async fn close(&mut self) -> Result<(), ClaudeSDKError>;

    /// Check if the transport is ready for I/O.
    fn is_ready(&self) -> bool;

    /// Signal end of input (close stdin).
    async fn end_input(&mut self) -> Result<(), ClaudeSDKError>;
}

/// Subprocess-based transport that spawns the claw-code binary with `--sdk`.
pub struct SubprocessTransport {
    child: Option<tokio::process::Child>,
    stdin: Option<tokio::process::ChildStdin>,
    stdout: Option<tokio::process::ChildStdout>,
    cli_path: Option<String>,
    ready: bool,
}

impl SubprocessTransport {
    /// Create a new transport. If `cli_path` is None, attempts to find `claw-code` on PATH.
    pub fn new(cli_path: Option<String>) -> Self {
        Self {
            child: None,
            stdin: None,
            stdout: None,
            cli_path,
            ready: false,
        }
    }

    /// Create a transport pointed at a specific binary.
    pub fn with_binary(cli_path: &str) -> Self {
        Self::new(Some(cli_path.to_owned()))
    }

    /// Resolve the CLI binary path, searching PATH if needed.
    fn resolve_binary(&self) -> Result<String, ClaudeSDKError> {
        if let Some(ref path) = self.cli_path {
            return Ok(path.clone());
        }
        which::which("claw-code")
            .or_else(|_| which::which("claude"))
            .map(|p| p.to_string_lossy().into_owned())
            .map_err(|_| {
                ClaudeSDKError::CLINotFound(CLINotFoundError { cli_path: None })
            })
    }
}

#[async_trait]
impl Transport for SubprocessTransport {
    async fn connect(&mut self) -> Result<(), ClaudeSDKError> {
        let binary = self.resolve_binary()?;

        let mut child = tokio::process::Command::new(&binary)
            .arg("--sdk")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| {
                ClaudeSDKError::CLIConnection(CLIConnectionError {
                    message: format!("Failed to spawn {binary}: {e}"),
                })
            })?;

        let stdin = child.stdin.take().ok_or_else(|| {
            ClaudeSDKError::CLIConnection(CLIConnectionError {
                message: "Failed to acquire stdin handle".into(),
            })
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            ClaudeSDKError::CLIConnection(CLIConnectionError {
                message: "Failed to acquire stdout handle".into(),
            })
        })?;

        self.child = Some(child);
        self.stdin = Some(stdin);
        self.stdout = Some(stdout);
        self.ready = true;

        Ok(())
    }

    async fn write(&mut self, data: &str) -> Result<(), ClaudeSDKError> {
        use tokio::io::AsyncWriteExt;

        let stdin = self.stdin.as_mut().ok_or_else(|| {
            ClaudeSDKError::CLIConnection(CLIConnectionError {
                message: "Transport not connected (no stdin)".into(),
            })
        })?;

        stdin
            .write_all(data.as_bytes())
            .await
            .map_err(|e| {
                ClaudeSDKError::CLIConnection(CLIConnectionError {
                    message: format!("Write error: {e}"),
                })
            })?;
        stdin
            .write_all(b"\n")
            .await
            .map_err(|e| {
                ClaudeSDKError::CLIConnection(CLIConnectionError {
                    message: format!("Write newline error: {e}"),
                })
            })?;

        Ok(())
    }

    fn read_messages(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<SdkOutputEnvelope, ClaudeSDKError>> + Send>> {
        use tokio::io::AsyncBufReadExt;

        let stdout = self.stdout.take();
        let (tx, rx) = tokio::sync::mpsc::channel(64);

        // Spawn a task that reads lines from stdout and sends them through the channel.
        tokio::spawn(async move {
            let Some(stdout) = stdout else {
                let _ = tx
                    .send(Err(ClaudeSDKError::CLIConnection(CLIConnectionError {
                        message: "Transport not connected (no stdout)".into(),
                    })))
                    .await;
                return;
            };
            let reader = tokio::io::BufReader::new(stdout);
            let mut lines = reader.lines();
            loop {
                match lines.next_line().await {
                    Ok(Some(line)) => {
                        if line.is_empty() {
                            continue;
                        }
                        let result = serde_json::from_str::<SdkOutputEnvelope>(&line).map_err(
                            |e| {
                                ClaudeSDKError::CLIJSONDecode(CLIJSONDecodeError {
                                    line: line.clone(),
                                    original_error: e.to_string(),
                                })
                            },
                        );
                        if tx.send(result).await.is_err() {
                            // Receiver dropped, stop reading.
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let _ = tx
                            .send(Err(ClaudeSDKError::CLIConnection(CLIConnectionError {
                                message: format!("Read error: {e}"),
                            })))
                            .await;
                        break;
                    }
                }
            }
        });

        Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx))
    }

    async fn close(&mut self) -> Result<(), ClaudeSDKError> {
        if let Some(stdin) = self.stdin.take() {
            drop(stdin);
        }
        if let Some(mut child) = self.child.take() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        self.stdout = None;
        self.ready = false;
        Ok(())
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    async fn end_input(&mut self) -> Result<(), ClaudeSDKError> {
        // Drop stdin to signal EOF
        self.stdin = None;
        Ok(())
    }
}
