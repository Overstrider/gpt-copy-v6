# TASK-007: Complete backend verification coverage

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Repo: .
Kind: test
Wave: 7
Parallel Group: backend-qa
Owner Role: qa
Depends On: TASK-002, TASK-004, TASK-005, TASK-006

## Objective
Add and run focused backend tests for health, validation, persistence, mocked provider behavior, and streaming failure paths.

## Context
- Project rules require backend formatting, linting, build, and tests.
- Automated backend tests must not call external providers.
- Backend coverage should prove behavior, not just compilation.

## Write Scope
- backend/tests/**
- backend/src/**

## Acceptance Criteria
- Health tests verify GET /health success.
- Validation tests cover malformed JSON, empty content, oversized content, bad IDs, and structured errors.
- Persistence tests cover conversation creation, listing order, message loading, and failed request status.
- Provider and streaming tests cover success, 429, timeout, invalid response, and interrupted stream behavior using mocks.

## Verification Commands
- cargo fmt --manifest-path backend/Cargo.toml --all -- --check
- cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
- cargo test --manifest-path backend/Cargo.toml

## Risk Notes
- Clippy may require a rustup component that is not installed locally.
- SQLite tests should use isolated temporary database files or isolated in-memory pools.
