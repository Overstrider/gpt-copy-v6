# TASK-004: Implement conversation endpoints

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Repo: .
Kind: dev
Wave: 4
Parallel Group: backend-api
Owner Role: backend
Depends On: TASK-003

## Objective
Expose backend endpoints for listing conversations, creating conversations, and loading messages with validation and structured errors.

## Context
- Frontend sidebar and transcript require stable conversation APIs.
- Routes must validate IDs and request bodies.
- Response shapes should be stable for zod schemas.

## Write Scope
- backend/src/routes/conversations.rs
- backend/src/routes/mod.rs
- backend/tests/conversations.rs

## Acceptance Criteria
- List conversations endpoint returns persisted conversations in deterministic order.
- Create conversation endpoint accepts validated input and returns the new conversation JSON.
- Load messages endpoint validates conversation IDs and returns ordered messages.
- Invalid IDs and bad payloads return the shared structured error envelope.

## Verification Commands
- cargo test --manifest-path backend/Cargo.toml conversations
- cargo test --manifest-path backend/Cargo.toml validation

## Risk Notes
- Route naming must remain consistent with frontend API client paths.
- Pagination can be omitted unless needed, but response ordering must be deterministic.
