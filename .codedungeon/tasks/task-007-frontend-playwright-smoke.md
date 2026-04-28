# task-007: Frontend Playwright Smoke

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Agent: `cd_e2e_tester`
Role: frontend smoke verifier
Owns: `frontend/playwright.config.ts`, `frontend/e2e/chat-smoke.spec.ts`, `frontend/e2e/fixtures/`
Depends: task 006

Acceptance:

- Playwright proves a user can type and send a message and see an assistant response.
- Test uses mocked network or backend fake provider, never real OpenRouter quota.

Verify:

```powershell
Push-Location frontend
npx playwright test
Pop-Location
```
