# task-001: PR Workflow Preflight

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Agent: `cd_dev_worker`
Role: workflow preflight coordinator
Owns: no source files
Depends: none

Acceptance:

- `git remote get-url origin` succeeds.
- `gh auth status` succeeds.
- Worker stops before source edits if either prerequisite fails.

Verify:

```powershell
git remote get-url origin
gh auth status
```
