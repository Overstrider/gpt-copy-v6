# TASK-009: Implement frontend API and stream state

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Repo: .
Kind: dev
Wave: 7
Parallel Group: frontend-data
Owner Role: frontend
Depends On: TASK-006, TASK-008

## Objective
Add frontend API schemas, zod validation, local backend client functions, and native streaming fetch state management.

## Context
- Frontend must validate API responses with zod.
- Streaming fetch should consume backend event frames.
- Frontend must never read or expose OPENROUTER_API_KEY.

## Write Scope
- frontend/src/lib/schemas.ts
- frontend/src/lib/api.ts
- frontend/src/lib/env.ts
- frontend/src/hooks/useChatStream.ts
- frontend/src/lib/api.test.ts
- frontend/src/hooks/useChatStream.test.ts

## Acceptance Criteria
- Conversation list, conversation create, message load, send, and stream calls target a configurable backend base URL.
- Every non-streaming response and structured error shape consumed by the frontend is parsed by zod.
- Streaming state handles optimistic user messages, assistant deltas, completion, cancellation, and error display.
- No frontend file references OPENROUTER_API_KEY.

## Verification Commands
- npm --prefix frontend run test -- api
- npm --prefix frontend run test -- useChatStream

## Risk Notes
- Stream parser must tolerate frame boundaries split across chunks.
- Optimistic state must reconcile with persisted server message IDs without duplicating bubbles.
