# TASK-001: Set monorepo guardrails

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Repo: .
Kind: dev
Wave: 1
Parallel Group: root-setup
Owner Role: docs
Depends On: -

## Objective
Establish root documentation, placeholder environment example, and ignore rules for local secrets and generated artifacts.

## Context
- Project rules require backend/ and frontend/ monorepo layout.
- Tracked files must never contain real OpenRouter secrets.
- Generated databases, build outputs, node_modules, coverage, and test reports must stay untracked.

## Write Scope
- README.md
- .env.example
- .gitignore

## Acceptance Criteria
- README has sections for setup, environment, backend run, frontend run, tests, and troubleshooting.
- .env.example contains placeholders only and documents OPENROUTER_API_KEY plus OPENROUTER_MODEL default guidance.
- .gitignore excludes .env, SQLite database files, Rust target output, node_modules, Next build output, Playwright reports, and coverage output.

## Verification Commands
- git status --short
- powershell -NoProfile -Command "$prefix = 'OPENROUTER' + '_API_KEY=.*'; $pattern = $prefix + 'sk-|' + $prefix + 'or-'; git grep -I -n $pattern -- .; if ($LASTEXITCODE -eq 1) { exit 0 } elseif ($LASTEXITCODE -eq 0) { exit 1 } else { exit $LASTEXITCODE }"

## Risk Notes
- README command examples may need a final refresh after scripts and ports are finalized.
- Secret scanning should check tracked files, not ignored local .env.
