# task-010: PR, Review, And Final Gates

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

Agent: `cd_review_spec`, `cd_review_security`, `cd_test_reviewer`, `cd_review_classifier`, `cd_review_validator`
Role: PR/review/final gate coordinator
Owns: `.codedungeon/reviews/adv-review/review-manifest.json`, persona `findings-*.json`, validator/classifier outputs, generated `review.md`, generated `review.json`
Depends: task 009

Acceptance:

- Branch is pushed to a GitHub PR.
- Review evidence is generated from persona JSON and `review run`.
- `review post` succeeds.
- Project Rules status is checked and compacted if stale only due requested source creation.
- Final status comes only from `codedungeon run finalize`.
- No merge is performed.

Verify:

```powershell
./.codex/bin/codedungeon rules status --human
./.codex/bin/codedungeon qa run --phase 6 --fresh --cmd "Push-Location backend; cargo fmt -- --check; Pop-Location"
./.codex/bin/codedungeon qa run --phase 6 --cmd "Push-Location backend; cargo check; cargo clippy --all-targets -- -D warnings; cargo test; Pop-Location"
./.codex/bin/codedungeon qa run --phase 6 --cmd "Push-Location frontend; npm run lint; npm run test; npm run build; npx playwright test; Pop-Location"
./.codex/bin/codedungeon review run --dir .codedungeon/reviews/adv-review
./.codex/bin/codedungeon review post --dir .codedungeon/reviews/adv-review
./.codex/bin/codedungeon run finalize
```
