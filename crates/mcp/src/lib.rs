//! PixelCraft's MCP server: every engine command is reachable by agents.
//!
//! Status: the tool list (one `execute_command` tool plus the command catalog). The stdio
//! transport and the bridge to a running window follow (`docs/mcp.md`).
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

use pixelcraft_engine::{Session, command_specs};
use serde_json::{Value as Json, json};

/// The MCP `tools/list` answer.
pub fn tools() -> Json {
    json!([{
        "name": "execute_command",
        "description": "Run any PixelCraft command by id (see list_commands) with JSON params.",
        "inputSchema": { "type": "object", "properties": { "command": { "type": "string" }, "params": { "type": "object" } }, "required": ["command"] }
    }, {
        "name": "list_commands",
        "description": "Every PixelCraft command: id, label, menu path, shortcut and params.",
        "inputSchema": { "type": "object", "properties": {} }
    }])
}

/// The MCP `tools/call` answer for one tool.
pub fn call(session: &mut Session, tool: &str, args: &Json) -> Result<Json, String> {
    match tool {
        "list_commands" => serde_json::to_value(command_specs().iter().map(|c| c.info(session)).collect::<Vec<_>>()).map_err(|e| e.to_string()),
        "execute_command" => {
            let id = args.get("command").and_then(Json::as_str).ok_or("`command` is required")?;
            let params = args.get("params").cloned().unwrap_or_else(|| json!({}));
            session.execute(id, &params).map_err(|e| e.to_string())
        }
        other => Err(format!("unknown tool `{other}`")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_and_list() {
        let mut s = Session::new();
        assert!(call(&mut s, "execute_command", &json!({ "command": "file.new" })).is_ok());
        assert!(call(&mut s, "list_commands", &json!({})).unwrap().as_array().is_some_and(|a| !a.is_empty()));
        assert!(call(&mut s, "execute_command", &json!({})).is_err());
        assert!(call(&mut s, "nope", &json!({})).is_err());
    }
}
