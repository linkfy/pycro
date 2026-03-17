# Task Implementation

## Execution Steps
1. Activate the web target phase after the embedded payload contract from phase 15 is stable.
2. Implement target parsing and build orchestration for `--target web`.
3. Produce the required web runtime and packaged output using the shared embedded project payload rather than loose project sources.
4. Add smoke validation for web startup, embedded module loading, and asset access.
5. Close the phase with synchronized tracker/state and validation evidence.

## Task Board

| Task ID | Owner | Parallel Team | Status | Branch | Worktree | Validation Gate |
| --- | --- | --- | --- | --- | --- | --- |
| project-web-build | architecture-orchestrator | runtime-worker, platform-worker, api-worker, docs-tracker, qa-reviewer, commit-steward, example-scenario-worker | in_progress | codex/16-project-web-build | .worktrees/16-project-web-build-orchestrator | web smoke + standard preflight |

## Resume Checkpoint

- kickoff_date: 2026-03-17
- active_branch: `codex/16-project-web-build`
- startup_gate: requirements + design validated, implementation opened
- next_slice:
  - implement web target build orchestration using shared embedded payload contract
  - add web smoke validation scenario and evidence capture

## Reporting Contract

All contributors report summary-only payloads to the orchestrator:

- changed files
- validation evidence
- risks
- follow-ups
- ADR refs
- tracker refs
