# Project Rules Compact

PROJECT_RULES_STATUS: APPROVED
PROJECT_RULES_SOURCE: .codedungeon/project-rules.md

- MUST preserve the ChatGPT-style app scope for gpt-copy-v6.
- MUST use a Rust 2024 Axum backend in backend/.
- MUST use a Next.js App Router TypeScript frontend in frontend/.
- MUST keep OpenRouter calls server-side.
- MUST configure OpenRouter with OPENROUTER_API_KEY and OPENROUTER_MODEL.
- MUST default OPENROUTER_MODEL to nvidia/nemotron-3-super-120b-a12b:free when no local override exists.
- MUST NOT commit secrets, tokens, provider keys, or machine-local private configuration.
- MUST keep .env ignored and .env.example placeholder-only.
- MUST keep Project Rules status, digest, and read flags in plans, tasks, reviews, handoffs, and final reports.
- VERIFY backend changes with available Rust formatting, linting, build, and tests.
- VERIFY frontend changes with available linting, tests, build, and Playwright smoke coverage.
- VERIFY no tracked file contains an OpenRouter key before reporting completion.
- VERIFY final CodeDungeon reports only claim completion when Verification is PASS and review is APPROVED.
- MUST treat OPENROUTER_API_KEY as a local secret only.
- MUST handle OpenRouter 429, timeout, invalid response, and interrupted stream errors without crashing.
- MUST test provider failures with mocks instead of relying on real OpenRouter quota.
- MUST avoid committing generated databases, build outputs, node_modules, or test reports.
- MUST read .codedungeon/project-rules.compact.md before planning, implementing, reviewing, or reporting.
- MUST keep CodeDungeon workflows PR-centered and leave final merge to the user.
- MUST not call external providers from automated tests unless explicitly required by the user.
