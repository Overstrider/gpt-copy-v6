# task-006: Frontend Chat Experience

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Agent: `cd_dev_worker`
Role: frontend-specialist
Owns: `frontend/src/components/`, `frontend/src/hooks/useChatStream.ts`, component tests
Depends: tasks 004 and 005

Acceptance:

- UI has sidebar conversations, transcript, composer, user/assistant bubbles, loading/error/empty states, and mobile behavior.
- lucide-react icons are used for common actions.
- Assistant markdown renders through `react-markdown` and `remark-gfm`.
- Native streaming fetch updates assistant text incrementally.

Verify:

```powershell
Push-Location frontend
npm run lint
npm run test
npm run build
Pop-Location
```
