# Task Implementation

## Execution Steps
1. Add phase-launch Git CD bootstrap automation contract and evidence hooks.
2. Integrate Release Please and phase release evidence gates.
3. Add release asset matrix for `pycro` (Linux/macOS/Windows on x64 + ARM).
4. Improve Python API artifact readability and document maintenance rules.
5. Fix CI typing noise from optional benchmark dependencies while keeping strict typing.
6. Validate model-routing policy enforcement in orchestrator documents.

## Task Board

| Task ID | Owner | Parallel Team | Status | Branch | Worktree | Validation Gate |
| --- | --- | --- | --- | --- | --- | --- |
| git-cd-release-automation | architecture-orchestrator | phase-planner, worktree-manager, api-worker, docs-tracker, qa-reviewer, commit-steward | in_progress | codex/08-git-cd-release-automation | .worktrees/08-git-cd-release-orchestrator | per docs/validation-policy.md |

## Reporting Contract

All contributors report summary-only payloads to the orchestrator:

- changed files
- validation evidence
- risks
- follow-ups
- ADR refs
- tracker refs
