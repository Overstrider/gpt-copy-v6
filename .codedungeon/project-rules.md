# Project Rules

Status: APPROVED
Generated: setup-example

## Sources Reviewed

- AGENTS.md
- README.md
- .env.example
- prompts/full-v6.txt
- examples/example-model.md setup contract

## Architecture And Boundaries

- The repository is a generated ChatGPT-style app named gpt-copy-v6.
- Backend source belongs in backend/ and must use Rust 2024 with Axum.
- Frontend source belongs in frontend/ and must use Next.js App Router with TypeScript.
- CodeDungeon runtime belongs in .codedungeon/ and Codex provider files belong in .codex/ and .agents/.
- OpenRouter provider calls must stay server-side; browser code must not receive provider secrets.

## Project Rules

- MUST preserve the ChatGPT-style app scope for gpt-copy-v6.
- MUST use a Rust 2024 Axum backend in backend/.
- MUST use a Next.js App Router TypeScript frontend in frontend/.
- MUST keep OpenRouter calls server-side.
- MUST configure OpenRouter with OPENROUTER_API_KEY and OPENROUTER_MODEL.
- MUST default OPENROUTER_MODEL to nvidia/nemotron-3-super-120b-a12b:free when no local override exists.
- MUST NOT commit secrets, tokens, provider keys, or machine-local private configuration.
- MUST keep .env ignored and .env.example placeholder-only.
- MUST keep Project Rules status, digest, and read flags in plans, tasks, reviews, handoffs, and final reports.

## Commands And Verification

- VERIFY backend changes with available Rust formatting, linting, build, and tests.
- VERIFY frontend changes with available linting, tests, build, and Playwright smoke coverage.
- VERIFY no tracked file contains an OpenRouter key before reporting completion.
- VERIFY final CodeDungeon reports only claim completion when Verification is PASS and review is APPROVED.

## Security And Data Rules

- MUST treat OPENROUTER_API_KEY as a local secret only.
- MUST handle OpenRouter 429, timeout, invalid response, and interrupted stream errors without crashing.
- MUST test provider failures with mocks instead of relying on real OpenRouter quota.
- MUST avoid committing generated databases, build outputs, node_modules, or test reports.

## Agent Operating Rules

- MUST read .codedungeon/project-rules.compact.md before planning, implementing, reviewing, or reporting.
- MUST keep CodeDungeon workflows PR-centered and leave final merge to the user.
- MUST not call external providers from automated tests unless explicitly required by the user.
- ASK WHEN requirements conflict with these rules or require exposing secrets.

## Open Questions

- None.