# task-009: Integrated Verification

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Agent: `cd_api_tester` and `cd_e2e_tester`
Role: integrated verifier
Owns: no source files
Depends: tasks 002 through 008

Acceptance:

- Backend format/check/clippy/tests pass.
- Frontend lint/test/build/Playwright pass.
- No automated test calls external OpenRouter.
- Tracked-file secret scan passes.

Verify:

```powershell
Push-Location backend; cargo fmt -- --check; cargo check; cargo clippy --all-targets -- -D warnings; cargo test; Pop-Location
Push-Location frontend; npm run lint; npm run test; npm run build; npx playwright test; Pop-Location
git grep -n -E "sk-or-v1-|OPENROUTER_API_KEY=." -- . ':!*.lock'
```
