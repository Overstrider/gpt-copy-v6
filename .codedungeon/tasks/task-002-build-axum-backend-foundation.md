# TASK-002: Build Axum backend foundation

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Repo: .
Kind: dev
Wave: 2
Parallel Group: backend-foundation
Owner Role: backend
Depends On: TASK-001

## Objective
Create the Rust 2024 Axum service foundation with configuration, shared state, tracing, CORS, structured errors, and GET /health.

## Context
- Backend must use Rust 2024 and Axum.
- CORS must allow the frontend dev server.
- Request and server errors must return structured JSON.

## Write Scope
- backend/Cargo.toml
- backend/src/main.rs
- backend/src/lib.rs
- backend/src/config.rs
- backend/src/state.rs
- backend/src/error.rs
- backend/src/routes/mod.rs
- backend/src/routes/health.rs

## Acceptance Criteria
- Cargo.toml uses edition 2024 and includes Axum, Tokio, tracing, tower-http, serde, sqlx, reqwest, and focused test dependencies.
- GET /health returns a stable JSON success response.
- Router installs tracing and local-development CORS layers.
- Malformed JSON and validation failures use the shared structured error envelope.

## Verification Commands
- cargo fmt --manifest-path backend/Cargo.toml --all -- --check
- cargo test --manifest-path backend/Cargo.toml health

## Risk Notes
- Middleware ordering must preserve CORS preflight behavior.
- Configuration defaults should not require real provider secrets for health checks or tests.
