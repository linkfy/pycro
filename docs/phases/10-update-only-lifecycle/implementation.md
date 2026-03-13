# Task Implementation

## Execution Steps
1. Remove setup auto-dispatch from runtime lifecycle.
2. Change CLI default script resolution to `main.py` in project root.
3. Update `pycro init` scaffold to copy the current `pycro` executable into the generated project directory.
4. Update tests for update-only lifecycle and new bootstrap defaults.
5. Update docs and architecture references.
6. Run full validation gates and sync tracker/state.

## Task Board

| Task ID | Owner | Parallel Team | Status | Branch | Worktree | Validation Gate |
| --- | --- | --- | --- | --- | --- | --- |
| lifecycle-update-only | architecture-orchestrator | runtime-worker, api-worker, docs-tracker, qa-reviewer, commit-steward | complete | codex/10-lifecycle-update-only | /Users/linkfy/Code/pycro | pass |

## Reporting Contract

All contributors report summary-only payloads to the orchestrator:

- changed files
- validation evidence
- risks
- follow-ups
- ADR refs
- tracker refs
