---
"@googleworkspace/cli": minor
---

Add auto-expand for MCP list endpoints: when `expand: true` is passed to a list tool call, the server fans out parallel get calls for each returned item ID and returns full details in a single response. Configurable via `expand_limit` (default 10) and `expand_format` (metadata or full).
