//! Session management functions (synchronous file I/O).

use std::fs;
use std::path::{Path, PathBuf};

use sdk_types::session::{SDKSessionInfo, SessionMessage};

use crate::error::ClaudeSDKError;

/// Get the base sessions directory for a given project directory.
fn sessions_dir(directory: Option<&str>) -> Result<PathBuf, ClaudeSDKError> {
    let home = dirs::home_dir().ok_or_else(|| {
        ClaudeSDKError::General("Cannot determine home directory".into())
    })?;
    let base = home.join(".claude").join("projects");

    let dir = match directory {
        Some(d) => {
            let canonical = fs::canonicalize(d)
                .map_err(|e| ClaudeSDKError::General(format!("Cannot canonicalize {d}: {e}")))?;
            base.join(hash_directory(&canonical))
        }
        None => base,
    };

    Ok(dir)
}

/// Simple directory hash matching Claude Code's convention.
fn hash_directory(path: &Path) -> String {
    use std::fmt::Write;
    let lossy = path.to_string_lossy();
    let bytes = lossy.as_bytes();
    let mut hash = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        write!(hash, "{b:02x}").unwrap();
    }
    if hash.len() > 64 {
        let mut acc: u64 = 0;
        for chunk in hash.as_bytes().chunks(8) {
            let mut arr = [0u8; 8];
            arr[..chunk.len()].copy_from_slice(chunk);
            acc = acc.wrapping_add(u64::from_le_bytes(arr));
        }
        format!("{acc:016x}")
    } else {
        hash
    }
}

/// List past sessions. Synchronous.
pub fn list_sessions(
    directory: Option<&str>,
    limit: Option<usize>,
) -> Result<Vec<SDKSessionInfo>, ClaudeSDKError> {
    let dir = sessions_dir(directory)?;
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();

    let entries = fs::read_dir(&dir)
        .map_err(|e| ClaudeSDKError::General(format!("Cannot read sessions dir: {e}")))?;

    for entry in entries {
        let entry = entry.map_err(|e| ClaudeSDKError::General(format!("Dir entry error: {e}")))?;
        let path = entry.path();
        if !path.extension().is_some_and(|ext| ext == "jsonl") {
            continue;
        }

        let metadata = fs::metadata(&path)
            .map_err(|e| ClaudeSDKError::General(format!("Cannot read file metadata: {e}")))?;

        let session_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_owned();

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs().to_string());

        sessions.push(sdk_types::session::SDKSessionInfo {
            session_id,
            summary: None,
            last_modified: modified,
            file_size: Some(metadata.len()),
            custom_title: None,
            first_prompt: None,
            git_branch: None,
            cwd: None,
            tag: None,
            created_at: None,
        });
    }

    // Sort by last_modified descending (most recent first)
    sessions.sort_by(|a, b| {
        let a_time = a.last_modified.as_deref().unwrap_or("0");
        let b_time = b.last_modified.as_deref().unwrap_or("0");
        b_time.cmp(a_time)
    });

    if let Some(limit) = limit {
        sessions.truncate(limit);
    }

    Ok(sessions)
}

/// Get messages from a past session. Synchronous.
pub fn get_session_messages(
    session_id: &str,
    directory: Option<&str>,
    limit: Option<usize>,
    offset: usize,
) -> Result<Vec<SessionMessage>, ClaudeSDKError> {
    let dir = sessions_dir(directory)?;
    let file_path = dir.join(format!("{session_id}.jsonl"));

    if !file_path.exists() {
        return Err(ClaudeSDKError::General(format!(
            "Session {session_id} not found"
        )));
    }

    let contents = fs::read_to_string(&file_path)
        .map_err(|e| ClaudeSDKError::General(format!("Cannot read session file: {e}")))?;

    let mut messages: Vec<SessionMessage> = contents
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    if offset > 0 {
        messages = messages.into_iter().skip(offset).collect();
    }

    if let Some(limit) = limit {
        messages.truncate(limit);
    }

    Ok(messages)
}

/// Get info for a single session. Synchronous.
pub fn get_session_info(
    session_id: &str,
    directory: Option<&str>,
) -> Result<Option<SDKSessionInfo>, ClaudeSDKError> {
    let sessions = list_sessions(directory, None)?;
    Ok(sessions.into_iter().find(|s| s.session_id == session_id))
}

/// Rename a session. Synchronous.
/// TODO: implement title persistence when session metadata format is finalized.
pub fn rename_session(
    session_id: &str,
    _title: &str,
    directory: Option<&str>,
) -> Result<(), ClaudeSDKError> {
    let dir = sessions_dir(directory)?;
    let file_path = dir.join(format!("{session_id}.jsonl"));
    if !file_path.exists() {
        return Err(ClaudeSDKError::General(format!(
            "Session {session_id} not found"
        )));
    }
    Ok(())
}

/// Tag a session. Synchronous.
/// TODO: implement tag persistence when session metadata format is finalized.
pub fn tag_session(
    session_id: &str,
    _tag: Option<&str>,
    directory: Option<&str>,
) -> Result<(), ClaudeSDKError> {
    let dir = sessions_dir(directory)?;
    let file_path = dir.join(format!("{session_id}.jsonl"));
    if !file_path.exists() {
        return Err(ClaudeSDKError::General(format!(
            "Session {session_id} not found"
        )));
    }
    Ok(())
}
