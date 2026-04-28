# task-005: Frontend Scaffold And API Boundary

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Agent: `cd_dev_worker`
Role: frontend-specialist
Owns: `frontend/package.json`, `frontend/package-lock.json`, `frontend/next.config.*`, `frontend/tsconfig.json`, `frontend/tailwind.config.*`, `frontend/postcss.config.*`, `frontend/app/layout.tsx`, `frontend/app/page.tsx`, `frontend/src/lib/api.ts`, `frontend/src/lib/schemas.ts`, `frontend/src/lib/env.ts`, frontend test setup
Depends: task 002 endpoint shapes

Acceptance:

- Next App Router TypeScript/Tailwind app builds.
- Frontend uses only `NEXT_PUBLIC_API_BASE_URL`.
- zod validates API responses and exposes parse errors.
- No OpenRouter secret is referenced in frontend code.

Verify:

```powershell
Push-Location frontend
npm install
npm run lint
npm run test
npm run build
Pop-Location
```
