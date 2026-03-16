# Task Implementation

## Execution Steps
1. Run designated playable scenarios with user.
2. Record outcome and follow-ups.
3. Close phase only after acceptance evidence is synchronized.
4. If scenario behavior regresses, patch the scenario/runtime contract, then re-run manual acceptance.

## Task Board

| Task ID | Owner | Parallel Team | Status | Branch | Worktree | Validation Gate |
| --- | --- | --- | --- | --- | --- | --- |
| manual-playtest-evidence-gate | example-scenario-worker | architecture-orchestrator, docs-tracker, qa-reviewer, commit-steward | complete | codex/07-manual-playtest-evidence-gate | .worktrees/07-manual-playtest-example | pass (`docs/phases/07-manual-playtest-evidence/closeout.md`) |

## Reporting Contract

All contributors report summary-only payloads to the orchestrator:

- changed files
- validation evidence
- risks
- follow-ups
- ADR refs
- tracker refs
