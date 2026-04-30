# TASK-010: Build ChatGPT-style frontend UI

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Repo: .
Kind: dev
Wave: 8
Parallel Group: frontend-ui
Owner Role: frontend
Depends On: TASK-008, TASK-009

## Objective
Implement the responsive chat application interface with conversations sidebar, transcript, composer, markdown assistant bubbles, loading state, error state, and mobile behavior.

## Context
- Frontend must be the actual chat experience, not a landing page.
- Use lucide-react icons for common actions.
- Assistant markdown must render safely with react-markdown and remark-gfm.

## Write Scope
- frontend/app/page.tsx
- frontend/app/globals.css
- frontend/src/components/ChatApp.tsx
- frontend/src/components/ChatApp.test.tsx

## Acceptance Criteria
- Desktop layout shows a usable conversation sidebar and main transcript.
- Mobile layout supports sidebar open/close without overlapping transcript or composer content.
- User and assistant bubbles are visually distinct and assistant markdown supports GFM.
- Composer exposes disabled/loading behavior during streaming and visible errors after failed requests.

## Verification Commands
- npm --prefix frontend run lint
- npm --prefix frontend run test -- ChatApp

## Risk Notes
- Text must not overflow buttons, bubbles, or mobile sidebars.
- react-markdown should not enable raw HTML unless an explicit sanitizer is added.
