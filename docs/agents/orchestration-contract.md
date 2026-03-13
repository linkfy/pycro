# Orchestration Contract

## Core Rule

Every implementation task runs with an active orchestrator and delegated ownership.

- No "god agent" execution for end-to-end implementation.
- The orchestrator is integration-focused and context-thin.
- Domain workers own implementation slices and report concise summaries.

## Mandatory Delegation

For implementation work, orchestration must include:

- at least one domain worker (`runtime-worker`, `platform-worker`, or `api-worker`)
- `docs-tracker` for tracker/state sync
- `qa-reviewer` before implementation commit
- `commit-steward` after validations are green

When user-visible behavior changes:

- include `example-scenario-worker`

When lifecycle/dispatch contract changes:

- include `flow-visualizer`

## Worktree Rule

`worktree-manager` must allocate dedicated worktrees when:

- two or more workers edit disjoint files in parallel
- branch collision risk is non-trivial
- one slice is exploratory while another is commit-bound

Naming pattern:

- branch: `codex/<phase>-<task>`
- worktree path: `.worktrees/<phase>-<task>-<agent>`

## Reporting Rule

Workers report only summary payloads with the schema in `docs/agent-registry.md`.
Raw logs are not sent to the orchestrator.
