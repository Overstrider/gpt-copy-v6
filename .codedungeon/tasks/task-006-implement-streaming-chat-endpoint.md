# TASK-006: Implement streaming chat endpoint

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Repo: .
Kind: dev
Wave: 6
Parallel Group: backend-streaming
Owner Role: backend
Depends On: TASK-004, TASK-005

## Objective
Expose an SSE or streaming fetch endpoint that streams assistant deltas, persists final results, and records interrupted or failed stream status.

## Context
- Streaming chat messages are required.
- Interrupted stream errors must be handled without crashing.
- Frontend will consume event frames through native streaming fetch.

## Write Scope
- backend/src/provider.rs
- backend/src/routes/chat.rs
- backend/tests/streaming.rs

## Acceptance Criteria
- Streaming endpoint emits deterministic event types for start, delta, complete, and error.
- Successful streams persist the completed assistant message and request status.
- Interrupted streams and provider stream errors persist failed or interrupted status and emit a structured error event.
- No streaming test calls real OpenRouter.

## Verification Commands
- cargo test --manifest-path backend/Cargo.toml streaming
- cargo test --manifest-path backend/Cargo.toml provider

## Risk Notes
- Chunk boundaries may split event payloads and should be covered in tests.
- Cancellation behavior depends on async drop semantics and must avoid panics.
