# TASK-011: Add frontend component and Playwright tests

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Repo: .
Kind: test
Wave: 9
Parallel Group: frontend-qa
Owner Role: qa
Depends On: TASK-009, TASK-010

## Objective
Add focused frontend tests and one Playwright smoke test for sending a message without a real OpenRouter call.

## Context
- Project rules require focused component tests and one Playwright smoke test.
- Smoke coverage must prove the send-message workflow.
- Tests should use mocked or deterministic backend behavior.

## Write Scope
- frontend/vitest.config.ts
- frontend/vitest.setup.ts
- frontend/src/**/*.test.ts
- frontend/src/**/*.test.tsx
- frontend/playwright.config.ts
- frontend/e2e/chat-smoke.spec.ts

## Acceptance Criteria
- Component tests cover loading state, conversation display, sending a message, assistant response rendering, structured error display, and practical mobile sidebar behavior.
- API/schema tests cover zod validation success and failure paths.
- Playwright smoke sends a message and observes user plus assistant transcript updates with no real provider key.
- Tests do not require OPENROUTER_API_KEY.

## Verification Commands
- npm --prefix frontend run test
- npm --prefix frontend run test:e2e

## Risk Notes
- Playwright browsers may need installation before smoke tests can run.
- E2E should avoid depending on external network or live OpenRouter quota.
