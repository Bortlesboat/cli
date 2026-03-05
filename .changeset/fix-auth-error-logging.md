---
"@anthropic/gws": patch
---

Log auth errors to stderr instead of silently swallowing them

Previously, when OAuth authentication failed in the main CLI flow, the error
was silently discarded and the request proceeded unauthenticated. This caused
confusing 401/403 responses from the API with no indication of the root cause.

Now prints the original auth error and a hint to stderr, making it clear why
authentication failed (expired token, missing credentials, wrong account, etc.).
