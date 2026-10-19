# MCP

> **Last updated:** 2026-10-19

`crates/mcp` exposes two tools:

- `list_commands`: every command with its id, label, menu path, shortcut and params;
- `execute_command`: `{ "command": "<id>", "params": { … } }` runs any command.

The stdio transport (`pixelcraft-cli mcp`) and the bridge to a running window are next.
