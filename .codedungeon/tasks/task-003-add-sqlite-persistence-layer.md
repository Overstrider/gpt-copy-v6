# TASK-003: Add SQLite persistence layer

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Repo: .
Kind: dev
Wave: 3
Parallel Group: backend-data
Owner Role: backend
Depends On: TASK-002

## Objective
Add sqlx SQLite migrations, database initialization, data models, and repository helpers for conversations, messages, and chat request status.

## Context
- Conversation and message persistence is required.
- SQLite access must go through sqlx.
- Streaming and failure handling need explicit persisted request status.

## Write Scope
- backend/migrations/**
- backend/src/db.rs
- backend/src/models.rs

## Acceptance Criteria
- Migrations create conversations, messages, and chat request/status storage with timestamps.
- Database startup runs migrations before serving requests.
- Repository helpers support listing conversations, creating conversations, inserting messages, loading messages, and updating request status.
- Test fixtures can create isolated temporary SQLite databases.

## Verification Commands
- cargo test --manifest-path backend/Cargo.toml db
- cargo test --manifest-path backend/Cargo.toml persistence

## Risk Notes
- SQLite test databases must not leak state across tests.
- Schema choices should avoid requiring sqlx offline metadata unless the project explicitly adds it.
