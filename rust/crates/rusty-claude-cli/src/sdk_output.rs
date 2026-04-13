use std::io::{BufWriter, Write};

use serde_json::Value;

/// Writes SDK wire-protocol JSON-lines to an underlying writer (typically stdout).
pub struct SdkOutputAdapter<W: Write> {
    writer: BufWriter<W>,
    session_id: String,
    model: String,
}

impl<W: Write> SdkOutputAdapter<W> {
    pub fn new(writer: W, session_id: String, model: String) -> Self {
        Self {
            writer: BufWriter::new(writer),
            session_id,
            model,
        }
    }

    /// Emit the init system message at the start of an SDK session.
    pub fn emit_init(&mut self, tools: &[String], cwd: &str) -> std::io::Result<()> {
        self.write_json_line(&serde_json::json!({
            "type": "system",
            "subtype": "init",
            "session_id": self.session_id,
            "tools": tools,
            "model": self.model,
            "cwd": cwd,
        }))
    }

    /// Emit an assistant message with the given content blocks.
    pub fn emit_assistant_message(
        &mut self,
        content: Vec<Value>,
        uuid: &str,
        message_id: &str,
    ) -> std::io::Result<()> {
        self.write_json_line(&serde_json::json!({
            "type": "assistant",
            "message": {
                "role": "assistant",
                "content": content,
            },
            "model": self.model,
            "session_id": self.session_id,
            "uuid": uuid,
            "message_id": message_id,
        }))
    }

    /// Emit a tool use block as an assistant message.
    pub fn emit_tool_use(
        &mut self,
        id: &str,
        name: &str,
        input: &Value,
        uuid: &str,
        message_id: &str,
    ) -> std::io::Result<()> {
        self.write_json_line(&serde_json::json!({
            "type": "assistant",
            "message": {
                "role": "assistant",
                "content": [{
                    "type": "tool_use",
                    "id": id,
                    "name": name,
                    "input": input,
                }],
            },
            "model": self.model,
            "session_id": self.session_id,
            "uuid": uuid,
            "message_id": message_id,
        }))
    }

    /// Emit a tool result message.
    pub fn emit_tool_result(
        &mut self,
        tool_use_id: &str,
        content: &str,
        is_error: bool,
    ) -> std::io::Result<()> {
        self.write_json_line(&serde_json::json!({
            "type": "tool_result",
            "tool_use_id": tool_use_id,
            "content": content,
            "is_error": is_error,
        }))
    }

    /// Emit the final result message at the end of an SDK turn.
    pub fn emit_result(
        &mut self,
        subtype: &str,
        duration_ms: u64,
        duration_api_ms: u64,
        is_error: bool,
        num_turns: u32,
        cost_usd: Option<f64>,
        usage: Option<Value>,
        result: Option<String>,
        stop_reason: Option<String>,
    ) -> std::io::Result<()> {
        let mut obj = serde_json::json!({
            "type": "result",
            "subtype": subtype,
            "duration_ms": duration_ms,
            "duration_api_ms": duration_api_ms,
            "is_error": is_error,
            "num_turns": num_turns,
            "session_id": self.session_id,
        });
        let map = obj.as_object_mut().expect("json object");
        if let Some(cost) = cost_usd {
            map.insert("total_cost_usd".to_string(), serde_json::json!(cost));
        }
        if let Some(usage) = usage {
            map.insert("usage".to_string(), usage);
        }
        if let Some(result) = result {
            map.insert("result".to_string(), serde_json::json!(result));
        }
        if let Some(reason) = stop_reason {
            map.insert("stop_reason".to_string(), serde_json::json!(reason));
        }
        self.write_json_line(&obj)
    }

    /// Emit a raw JSON value as one line.
    fn write_json_line(&mut self, value: &Value) -> std::io::Result<()> {
        writeln!(
            self.writer,
            "{}",
            serde_json::to_string(value)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        )
    }

    /// Flush the underlying writer.
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::SdkOutputAdapter;

    fn make_adapter() -> SdkOutputAdapter<Vec<u8>> {
        SdkOutputAdapter::new(Vec::new(), "test-session".to_string(), "test-model".to_string())
    }

    fn lines(output: &[u8]) -> Vec<String> {
        let s = String::from_utf8_lossy(output);
        s.lines().map(String::from).collect()
    }

    #[test]
    fn emit_init_wires_correct_json() {
        let mut adapter = make_adapter();
        adapter
            .emit_init(&["Bash".to_string(), "Read".to_string()], "/home/user")
            .unwrap();
        let output = adapter.writer.into_inner().unwrap();
        let ls = lines(&output);
        assert_eq!(ls.len(), 1);
        let v: serde_json::Value = serde_json::from_str(&ls[0]).unwrap();
        assert_eq!(v["type"], "system");
        assert_eq!(v["subtype"], "init");
        assert_eq!(v["session_id"], "test-session");
        assert_eq!(v["model"], "test-model");
        assert_eq!(v["tools"], serde_json::json!(["Bash", "Read"]));
    }

    #[test]
    fn emit_assistant_message_wires_content_blocks() {
        let mut adapter = make_adapter();
        adapter
            .emit_assistant_message(
                vec![serde_json::json!({"type": "text", "text": "hello"})],
                "uuid-1",
                "msg-1",
            )
            .unwrap();
        let output = adapter.writer.into_inner().unwrap();
        let v: serde_json::Value = serde_json::from_str(&lines(&output)[0]).unwrap();
        assert_eq!(v["type"], "assistant");
        assert_eq!(v["message"]["role"], "assistant");
        assert_eq!(v["message"]["content"][0]["text"], "hello");
    }

    #[test]
    fn emit_tool_result_includes_error_flag() {
        let mut adapter = make_adapter();
        adapter
            .emit_tool_result("tool-123", "failed output", true)
            .unwrap();
        let output = adapter.writer.into_inner().unwrap();
        let v: serde_json::Value = serde_json::from_str(&lines(&output)[0]).unwrap();
        assert_eq!(v["type"], "tool_result");
        assert_eq!(v["tool_use_id"], "tool-123");
        assert_eq!(v["is_error"], true);
    }

    #[test]
    fn emit_result_omits_optional_fields_when_none() {
        let mut adapter = make_adapter();
        adapter
            .emit_result("success", 100, 80, false, 1, None, None, None, None)
            .unwrap();
        let output = adapter.writer.into_inner().unwrap();
        let v: serde_json::Value = serde_json::from_str(&lines(&output)[0]).unwrap();
        assert_eq!(v["type"], "result");
        assert_eq!(v["subtype"], "success");
        assert!(v.get("total_cost_usd").is_none());
        assert!(v.get("usage").is_none());
    }

    #[test]
    fn emit_result_includes_optional_fields_when_provided() {
        let mut adapter = make_adapter();
        adapter
            .emit_result(
                "success",
                100,
                80,
                false,
                1,
                Some(0.05),
                Some(serde_json::json!({"input_tokens": 100, "output_tokens": 50})),
                Some("done".to_string()),
                Some("end_turn".to_string()),
            )
            .unwrap();
        let output = adapter.writer.into_inner().unwrap();
        let v: serde_json::Value = serde_json::from_str(&lines(&output)[0]).unwrap();
        assert_eq!(v["total_cost_usd"], 0.05);
        assert_eq!(v["usage"]["input_tokens"], 100);
        assert_eq!(v["result"], "done");
        assert_eq!(v["stop_reason"], "end_turn");
    }
}
