# task-008: Docs, Env, And Ignore Rules

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Agent: `cd_dev_worker`
Role: docs/env maintainer
Owns: `README.md`, `.env.example`, `.gitignore`
Depends: tasks 002 through 007

Acceptance:

- README has exact setup, env, backend run, frontend run, test, smoke, and troubleshooting commands.
- `.env.example` is placeholder-only and keeps the required default model.
- `.gitignore` covers secrets, DBs, Rust/Next outputs, node_modules, Playwright reports, and generated coverage.

Verify:

```powershell
git diff --check
$openrouterKeyPrefix = "sk-" + "or-v1-"
git grep -n -- $openrouterKeyPrefix -- . ':!*.lock'
```
