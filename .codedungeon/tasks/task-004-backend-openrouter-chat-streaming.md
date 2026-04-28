# task-004: Backend OpenRouter Chat And Streaming

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Agent: `cd_dev_worker`
Role: backend-specialist
Owns: `backend/src/provider.rs`, `backend/src/routes/chat.rs`, `backend/tests/provider.rs`, `backend/tests/streaming.rs`
Depends: task 003

Acceptance:

- OpenRouter is behind a server-side provider trait.
- `POST /conversations/:id/messages` sends and persists user/assistant messages.
- `POST /conversations/:id/messages/stream` emits SSE and persists final assistant text.
- Provider 429, timeout, invalid response, and interrupted stream paths are tested with fakes/mocks only.
- Streaming does not hold a SQLite transaction across provider reads.

Verify:

```powershell
Push-Location backend
cargo test provider streaming -- --nocapture
cargo clippy --all-targets -- -D warnings
Pop-Location
```
