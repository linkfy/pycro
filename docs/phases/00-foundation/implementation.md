# Task Implementation

## Execution Steps
1. Maintain phase docs and tracker/state sync.
2. Validate governance invariants.
3. Hand off to next phase once all gates are green.

## Task Board

| Task ID | Owner | Parallel Team | Status | Branch | Worktree | Validation Gate |
| --- | --- | --- | --- | --- | --- | --- |
| bootstrap-foundation | architecture-orchestrator | runtime-worker, platform-worker, api-worker, docs-tracker, qa-reviewer, commit-steward | closed | codex/<phase>-<task> | .worktrees/<phase>-<task>-<agent> | per docs/validation-policy.md |

## Reporting Contract

All contributors report summary-only payloads to the orchestrator:

- changed files
- validation evidence
- risks
- follow-ups
- ADR refs
- tracker refs
