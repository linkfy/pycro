# Agent Playbook

## Core Rule

The orchestrator consumes summaries, not raw logs.
Every implementation task must be delegated; no end-to-end "god agent" execution.

## Phase Startup Checklist (Mandatory)

Before any phase implementation starts, the orchestrator must confirm:

1. `requirements.md` exists and contains concrete acceptance criteria.
2. `design.md` exists and is consistent with the requirements.
3. `implementation.md` task board is initialized with owners, branch/worktree plan, and validation gates.

Required sequence for every phase:

- requirements -> design -> implementation

If requirements/design change during delivery, the orchestrator pauses implementation and re-syncs docs before resuming.
If requirements are not defined for a phase, the orchestrator must force planning mode and produce phase-ready requirements, design, and implementation steps before opening execution.

## Model Routing Policy (Mandatory)

- Planning mode (requirements/design/implementation planning) must run with the official orchestrator on ChatGPT 5.4.
- Implementation, review, sync, and integration default to Codex 5.3 medium.
- Simpler models are allowed only for low-risk, bounded tasks (for example pure renames, formatting-only doc sync, or mechanical path updates) and the orchestrator must record the reason.

## Standard Delivery Flow

Required agents for implementation work:

- `architecture-orchestrator`
- one or more domain workers (`runtime-worker`, `platform-worker`, `api-worker`)
- full domain team when the phase scope spans runtime + platform + API together
- `docs-tracker`
- `qa-reviewer`
- `commit-steward`

Conditional agents:

- `example-scenario-worker` for user-visible interactive changes
- `flow-visualizer` for lifecycle/dispatch contract updates
- `interactive-refinement-recorder` when scope or requirements shift during execution
- `worktree-manager` when parallel slices can collide
- `merge-integrator` only after user-approved merge to `main`

Mandatory closeout rule:

- At the end of every phase, before any merge action, the orchestrator must explicitly ask the user if they want to merge now.
- No merge is allowed from implicit consent or by default; a direct user "yes" is required for each phase closeout.
- No merge/push is allowed until formal phase closeout is recorded (`closeout.md` + tracker/state sync) and `qa-reviewer` is `pass` (or explicit waiver recorded).

## Worktree Policy

Use worktrees when parallel implementation is requested or collision risk exists.

- branch naming: `codex/<phase>-<task>`
- worktree naming: `.worktrees/<phase>-<task>-<agent>`
- each worker owns a disjoint write scope
- the orchestrator tracks active branch/worktree assignments in tracker/state

## Reporting Contract

Worker report format is fixed:

- `changed_files`
- `validation_evidence`
- `risks`
- `follow_ups`
- `adr_refs`
- `tracker_refs`

## Review And Commit Gates

- `qa-reviewer` is mandatory before implementation commit.
- `commit-steward` is mandatory after required validations are green.
- If any gate is waived, waiver reason must be recorded in both tracker and state.
- Required local preflight before push/merge: `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`.

## Documentation Synchronization

`docs-tracker` keeps these synchronized at all times:

- `docs/task-tracker.txt`
- `state/repo-state.json`
- active phase docs under `docs/phases/<NN-slug>/`

Synchronization is complete only when task status, phase status, ownership, branch/worktree, and validation checkpoints match.

## Manual Playtest Gate

For user-visible interactive features:

- add or update playable scenario under `examples/*.py`
- run user playtest and record feedback evidence in tracker/state
- phase cannot close without this evidence when the gate applies
