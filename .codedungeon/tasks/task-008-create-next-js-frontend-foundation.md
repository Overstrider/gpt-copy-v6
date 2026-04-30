# TASK-008: Create Next.js frontend foundation

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Repo: .
Kind: dev
Wave: 2
Parallel Group: frontend-foundation
Owner Role: frontend
Depends On: TASK-001

## Objective
Create the Next.js App Router TypeScript and Tailwind application foundation with required dependencies and standard scripts.

## Context
- Frontend must use Next.js App Router, TypeScript, and Tailwind.
- lucide-react, zod, react-markdown, and remark-gfm are required.
- The first screen should be the usable chat application shell.

## Write Scope
- frontend/package.json
- frontend/package-lock.json
- frontend/next.config.ts
- frontend/tsconfig.json
- frontend/eslint.config.mjs
- frontend/tailwind.config.ts
- frontend/postcss.config.mjs
- frontend/app/layout.tsx
- frontend/app/page.tsx
- frontend/app/globals.css

## Acceptance Criteria
- Next.js App Router renders from frontend/app/page.tsx.
- Tailwind is configured and imported through the app global stylesheet.
- package scripts exist for dev, build, lint, unit tests, and Playwright smoke tests.
- Required frontend dependencies are installed and recorded in the lockfile.

## Verification Commands
- npm --prefix frontend run lint
- npm --prefix frontend run build

## Risk Notes
- Client-only chat code must not execute in a server component.
- Next, React, Tailwind, Vitest, and Playwright versions should be compatible.
