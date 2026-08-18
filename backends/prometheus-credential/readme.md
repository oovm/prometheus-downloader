# 🔐 prometheus-credential

Local **credential vault** lifecycle for Prometheus. The product goal is: cookies, tokens, and similar secrets stay on the user’s machine, under a vault file the user controls.

Not published to crates.io. Exposed to JavaScript today as `createVault(path, password)` via `prometheus-napi`.

## Current status

Skeleton API:

- Create an empty vault file at a path
- Open / round-trip enough structure for smoke tests

**Encryption is not implemented yet.** Treat on-disk vault bytes as a placeholder format that will change. Do not store production secrets expecting confidentiality from this crate alone.

## Intended direction (not all shipped)

| Concern | Direction |
|---------|-----------|
| At-rest encryption | Real crypto before calling the vault “ready” |
| Auth flows | User-owned sessions; interactive login helpers later |
| Scope | Credentials for hosts the user can access — not a credential-stuffing toolkit |

Torch (`@doki-land/prometheus-torch`) must **not** grow a second vault. Node calls into this crate through napi.

## License

CC0-1.0 — see the repository `License.md`.
