# Task Implementation

## Execution Steps
1. Run designated playable scenarios with user.
2. Record outcome and follow-ups.
3. Close phase only after acceptance evidence is synchronized.

## Task Board

| Task ID | Owner | Parallel Team | Status | Branch | Worktree | Validation Gate |
| --- | --- | --- | --- | --- | --- | --- |
| manual-playtest-evidence-gate | architecture-orchestrator | runtime-worker, platform-worker, api-worker, docs-tracker, qa-reviewer, commit-steward | planned | codex/<phase>-<task> | .worktrees/<phase>-<task>-<agent> | per docs/validation-policy.md |

## Reporting Contract

All contributors report summary-only payloads to the orchestrator:

- changed files
- validation evidence
- risks
- follow-ups
- ADR refs
- tracker refs
