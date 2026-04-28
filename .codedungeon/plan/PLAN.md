# CodeDungeon Plan: gpt-copy-v6

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Goal: implement `gpt-copy-v6`, a ChatGPT-style local monorepo with a Rust 2024 Axum backend, a Next.js App Router TypeScript/Tailwind frontend, SQLite persistence, server-only OpenRouter access, focused tests, adversarial review evidence, a GitHub PR, and CodeDungeon final gates.

Ordered tasks:

- [x] task-001-pr-workflow-preflight.md
- [x] task-002-backend-foundation-health.md
- [x] task-003-backend-sqlite-rest.md
- [x] task-004-backend-openrouter-chat-streaming.md
- [x] task-005-frontend-scaffold-api-boundary.md
- [x] task-006-frontend-chat-experience.md
- [x] task-007-frontend-playwright-smoke.md
- [x] task-008-docs-env-gitignore.md
- [x] task-009-integrated-verification.md
- [ ] task-010-pr-review-final-gates.md

Parallelization:

- Backend tasks 002 through 004 are sequential.
- Frontend task 005 can start after backend API shapes are defined and can overlap with backend provider work.
- Frontend tasks 006 and 007 are sequential after task 005.
- Docs/env task 008 waits for stable commands and file layout.
- Verification and PR/review/final gates run last.

PHASE_4_PLAN
