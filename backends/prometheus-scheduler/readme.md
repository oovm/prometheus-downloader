# 🗓️ prometheus-scheduler

**Task scheduling / state machine** crate for Prometheus: queueing downloads, persisting task state, pause / resume coordination with transfer backends.

Not published to crates.io. Currently a deliberate empty shell so the workspace layout matches the intended engine split.

## Intended responsibilities

| Concern | Notes |
|---------|--------|
| Task ids & states | queued / running / paused / finished / failed |
| Persistence | Pluggable; **no** SQLite hard dependency in the skeleton |
| Coordination | Works with in-process `TransferBackend` progress events |
| Multi-URL queues | Concurrent tasks ≠ single-file multi-connection Range |

Product MCP tools such as `pause_task` / `resume_task` / `get_progress` will sit on top of this crate once it is real.

## Constraints

- Keep the napi surface thin: scheduler logic in Rust, CLI/MCP as adapters
- Do not sneak a second process model in as “the” scheduler implementation

## License

CC0-1.0 — see the repository `License.md`.
