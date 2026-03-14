# Task Implementation

## Execution Steps
1. Create the canonical phase docs for the `project` roadmap and sync tracker/state.
2. Define the new `pycro project` CLI namespace and preserve existing run/init semantics by contract.
3. Define the external project directory contract and reserved project manifest (`pycro-project.toml`).
4. Define the shared `project bundle` concept and resource/provider design for future targets.
5. Add planning-level validation notes and phase sequencing for downstream target phases.

## Task Board

| Task ID | Owner | Parallel Team | Status | Branch | Worktree | Validation Gate |
| --- | --- | --- | --- | --- | --- | --- |
| project-build-foundation | architecture-orchestrator | runtime-worker, api-worker, docs-tracker, qa-reviewer, commit-steward | planned | codex/14-project-build-foundation | .worktrees/14-project-build-foundation-orchestrator | requirements + design synchronized before implementation |

## Reporting Contract

All contributors report summary-only payloads to the orchestrator:

- changed files
- validation evidence
- risks
- follow-ups
- ADR refs
- tracker refs
