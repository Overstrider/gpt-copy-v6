# TASK-005: Implement OpenRouter provider and send endpoint

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Repo: .
Kind: dev
Wave: 5
Parallel Group: backend-api
Owner Role: backend
Depends On: TASK-003

## Objective
Add a mockable OpenRouter provider abstraction, default model handling, server-side provider proxying, and a non-streaming send-message endpoint.

## Context
- OpenRouter calls must be server-side only.
- OPENROUTER_MODEL defaults to nvidia/nemotron-3-super-120b-a12b:free when unset.
- Provider failures must be mapped without requiring live network calls in tests.

## Write Scope
- backend/src/provider.rs
- backend/src/routes/chat.rs
- backend/src/routes/mod.rs
- backend/tests/provider.rs
- backend/tests/chat_send.rs

## Acceptance Criteria
- Provider abstraction can be replaced by mocks in tests.
- OpenRouter API key is read only from backend configuration and never exposed in responses.
- Send-message endpoint persists the user message, invokes the provider, persists the assistant message, and returns structured JSON.
- 429, timeout, and invalid provider response cases return structured application errors.

## Verification Commands
- cargo test --manifest-path backend/Cargo.toml provider
- cargo test --manifest-path backend/Cargo.toml chat_send

## Risk Notes
- Provider logging must not print authorization headers or API keys.
- Mock provider behavior should cover both success and failure paths.
