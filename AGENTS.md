# codedungeon for Codex CLI

Use codedungeon as the deterministic workflow kernel. Preserve the phase flow, local DB state, handoff schema, review JSON, and task contracts.

Project artifacts:
- Workflow skills: `.agents/skills/codedungeon/`, `.agents/skills/main-quest/`, `.agents/skills/side-quest/`, `.agents/skills/one-shot/`, `.agents/skills/code-review/`
- Editable command playbooks for reference: `.codedungeon/commands/`
- Phase instructions: `.codedungeon/phases/`
- Codex subagents: `.codex/agents/`
- Codex skills: `.agents/skills/`
- Local binary and DB: `./.codex/bin/codedungeon`, `.codedungeon/codedungeon.db`
- `.codedungeon/codedungeon.db` is runtime-local state. Preserve it on disk during local workflow runs, but do not commit it to PRs; recreate it with `./.codex/bin/codedungeon db init` or workflow commands when needed.

Default workflow:
- Invoke the promoted workflow router as `$codedungeon --full|--lite|--oneshot|--auto|--rules <prompt>`.
- `$codedungeon` without a mode flag selects automatically and prints `CODEDUNGEON_MODE_SELECTED: <mode> - <reason>` before dispatch.
- Run `$codedungeon --rules` before the first real task to deep-read the repo, draft `.codedungeon/project-rules.md`, get user confirmation, and generate `.codedungeon/project-rules.compact.md`.
- Read `.codedungeon/project-rules.compact.md` when present before planning, executing, reviewing, or reporting completion.
- Include `PROJECT_RULES_STATUS`, `PROJECT_RULES_DIGEST`, and `PROJECT_RULES_READ` in every plan, task, review, phase handoff, and final report.
- Keep compatibility aliases available: `$main-quest`, `$side-quest`, `$one-shot`.
- Use `$code-review`, `$codedungeon-test-loop`, and `$cleanup-tasks` for standalone review/test/cleanup flows.
- If Codex rejects a custom `agent_type`, run `codex features enable multi_agent_v2` or restart Codex with `--enable multi_agent_v2`.
- Use `./.codex/bin/codedungeon phase info` before changing phase state.
- Use `./.codex/bin/codedungeon spawn-prompt <phase>` to compose runtime phase context.
- Preserve the emitted `agent_type` when using Codex subagents. Record `model` and `reasoning_effort` in telemetry/prompt context; do not force model overrides when Codex rejects that combination.
- Record every subagent with `./.codex/bin/codedungeon trace agent-start` before spawn and `./.codex/bin/codedungeon trace agent-end` after it returns; telemetry is informational and does not replace gates.
- Do not write review reports manually. Generate review evidence with `./.codex/bin/codedungeon review run` from `review-manifest.json` and persona output files.
- Do not write final reports manually. Start final verification with `./.codex/bin/codedungeon qa run --phase 6 --fresh --cmd "<first cmd>"`, post review evidence with `./.codex/bin/codedungeon review post`, then use `./.codex/bin/codedungeon run finalize`; READY_FOR_USER_REVIEW can only come from CodeDungeon finalization/report rendering.
- Close completed phases with `./.codex/bin/codedungeon phase done`.
- Treat `.codedungeon/commands/` as reference playbooks, not Codex CLI slash commands.
- Keep provider-specific instructions in Codex files; do not copy Claude-only syntax into Codex prompts.

## codedungeon

Codex CLI pipeline available. Editable command playbooks live in `.codedungeon/commands/`.

Promoted workflow surface: `$codedungeon [--full|--lite|--oneshot|--auto|--rules] <prompt>`. Without a flag, `$codedungeon` selects automatically and prints `CODEDUNGEON_MODE_SELECTED: <mode> - <reason>` before dispatch. Run `$codedungeon --rules` before first real task to discover and approve project rules.

| Skill | Use when |
|-------|----------|
| `$codedungeon --oneshot` | Small tasks: plan, code, PR, review; no task split. |
| `$codedungeon --lite` | Simple planned tasks, single-repo. Requires `.codedungeon/plans/*.md`. |
| `$codedungeon --full` | Complex features, multi-repo, full phase pipeline. |
| `$codedungeon --rules` | Deep-read this repo, draft `.codedungeon/project-rules.md`, wait for user confirmation, then approve/compact rules. |
| `code-review` | Standalone adversarial review on current branch. |

Compatibility aliases remain installed: `$one-shot`, `$side-quest`, and `$main-quest`.

Project Rules: workflows read `.codedungeon/project-rules.compact.md` when approved and include `PROJECT_RULES_STATUS`, `PROJECT_RULES_DIGEST`, and `PROJECT_RULES_READ` in handoffs.

Agents in `.codex\agents/`, skills in `.agents\skills/`, commands/phases/mutable state in `.codedungeon/`. CLI binary at `.codex\bin/codedungeon`.
