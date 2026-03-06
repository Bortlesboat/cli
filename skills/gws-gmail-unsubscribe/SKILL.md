---
name: gws-gmail-unsubscribe
version: 1.0.0
description: "Gmail: List and one-click unsubscribe from mailing lists (RFC 8058)."
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["gws"]
    cliHelp: "gws gmail +unsubscribe --help"
---

# gmail +unsubscribe

> **PREREQUISITE:** Read `../gws-shared/SKILL.md` for auth, global flags, and security rules. If missing, run `gws generate-skills` to create it.

List and one-click unsubscribe from mailing lists (RFC 8058)

## Usage

```bash
gws gmail +unsubscribe
```

## Flags

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| `--list` | — | — | Scan recent emails for List-Unsubscribe headers, group by sender |
| `--from` | — | — | Unsubscribe from a specific sender (matches against From header) |
| `--dry-run` | — | — | Show what would happen without executing the unsubscribe |
| `--max` | — | 100 | Maximum messages to scan (default: 100) |

## Examples

```bash
gws gmail +unsubscribe --list
gws gmail +unsubscribe --list --max 200
gws gmail +unsubscribe --from noreply@mail.example.com
gws gmail +unsubscribe --from example.com --dry-run
```

## Tips

- Uses RFC 8058 one-click unsubscribe when available (POST with
- List-Unsubscribe=One-Click body). Falls back to showing mailto
- addresses for senders without RFC 8058 support.
- Use --dry-run to preview the unsubscribe action safely.

## See Also

- [gws-shared](../gws-shared/SKILL.md) — Global flags and auth
- [gws-gmail](../gws-gmail/SKILL.md) — All send, read, and manage email commands
