# task-003: Backend SQLite REST

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Agent: `cd_dev_worker`
Role: backend-specialist
Owns: `backend/migrations/`, `backend/src/db.rs`, `backend/src/models.rs`, `backend/src/routes/conversations.rs`, `backend/tests/persistence.rs`, `backend/tests/api_validation.rs`
Depends: task 002

Acceptance:

- SQLite tables cover conversations, messages, and chat_requests.
- Endpoints exist for `GET /conversations`, `POST /conversations`, and `GET /conversations/:id/messages`.
- Validation failures return structured JSON errors.
- Tests use isolated SQLite DBs.

Verify:

```powershell
Push-Location backend
cargo fmt -- --check
cargo test persistence api_validation -- --nocapture
Pop-Location
```
