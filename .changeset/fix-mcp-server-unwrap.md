---
"@anthropic/gws": patch
---

Replace unwrap() calls with proper error handling in MCP server

- Replace `.unwrap()` on `get_one("services")` with `.unwrap_or("")`
- Replace `req.get("id").unwrap()` with `if let Some(id)` pattern
- Replace `tools_cache.as_ref().unwrap()` with match expression
- Replace `parts.last().unwrap()` with `.ok_or_else()` returning a proper validation error
- Handle broken stdout pipe by breaking the server loop instead of silently continuing
- Add unit tests for `build_mcp_cli`, `walk_resources`, and `handle_request`
