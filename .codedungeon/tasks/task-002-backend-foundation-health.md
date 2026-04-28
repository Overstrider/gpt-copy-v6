# task-002: Backend Foundation And Health

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Agent: `cd_dev_worker`
Role: backend-specialist
Owns: `backend/Cargo.toml`, `backend/Cargo.lock`, `backend/src/main.rs`, `backend/src/lib.rs`, `backend/src/config.rs`, `backend/src/error.rs`, `backend/src/state.rs`, `backend/src/routes/health.rs`, `backend/tests/health.rs`
Depends: task 001

Acceptance:

- Rust 2024 Axum app starts.
- `GET /health` returns JSON status.
- Config reads env and defaults `OPENROUTER_MODEL`.
- Tracing and frontend-dev CORS are configured.
- Structured JSON error type exists.
- No OpenRouter call path is added yet.

Verify:

```powershell
Push-Location backend
cargo fmt -- --check
cargo check
cargo test health -- --nocapture
Pop-Location
```
