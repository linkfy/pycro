# Task Implementation

## Execution Steps
1. Activate the iOS target phase after phases 14-17 stabilize the shared project and multi-target model.
2. Implement target parsing and build orchestration for `--target ios`.
3. Generate the chosen iOS output format and package project scripts/assets into the target layout.
4. Add smoke validation for generated-project startup assumptions and packaged bundle resources.
5. Close the phase with synchronized tracker/state and validation evidence.

## Task Board

| Task ID | Owner | Parallel Team | Status | Branch | Worktree | Validation Gate |
| --- | --- | --- | --- | --- | --- | --- |
| project-ios-build | architecture-orchestrator | runtime-worker, platform-worker, api-worker, docs-tracker, qa-reviewer, commit-steward | planned | codex/18-project-ios-build | .worktrees/18-project-ios-build-orchestrator | ios packaging smoke + standard preflight |

## Reporting Contract

All contributors report summary-only payloads to the orchestrator:

- changed files
- validation evidence
- risks
- follow-ups
- ADR refs
- tracker refs
