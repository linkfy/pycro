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
| project-web-build | architecture-orchestrator | runtime-worker, platform-worker, api-worker, docs-tracker, qa-reviewer, commit-steward, example-scenario-worker | planned | codex/16-project-web-build | .worktrees/16-project-web-build-orchestrator | web smoke + standard preflight |

## Reporting Contract

All contributors report summary-only payloads to the orchestrator:

- changed files
- validation evidence
- risks
- follow-ups
- ADR refs
- tracker refs
