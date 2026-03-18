# Agent Playbook

## Core Rule

The orchestrator consumes summaries, not raw logs.
Every implementation task must be delegated; no end-to-end "god agent" execution.

## Phase Ownership Rule (Mandatory)

Every active phase must have one recorded `architecture-orchestrator` owner in phase docs, tracker, and state.
The orchestrator owns kickoff-to-closeout execution, including startup gate validation, delegation plan, worker intake, integration, and final gate readiness.
No phase marked `planned` or `in_progress` may proceed as ad hoc worker-led execution.

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
- `merge-integrator` only after user-approved merge to `develop` (default) or manual ready-for-release promotion from `develop` to `main`

Mandatory closeout rule:

- At the end of every phase, before any merge action, the orchestrator must explicitly ask the user if they want to merge now.
- No merge is allowed from implicit consent or by default; a direct user "yes" is required for each phase closeout.
- No merge/push is allowed until formal phase closeout is recorded (`closeout.md` + tracker/state sync) and `qa-reviewer` is `pass` (or explicit waiver recorded). Default merge target is `develop`; promotion to `main` is a separate explicit ready-for-release action.
- Mandatory develop gate: do not merge into `develop` while phase status is in progress/planned. Merge is allowed only after phase finalization, unless the programmer explicitly requests an override.

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

## Read-Only Worker Fallback (Mandatory)

When a delegated worker cannot write in the current environment, the worker still executes analysis/review and hands off a summary payload to the orchestrator.
The handoff must include target files, proposed edits, validation intent/evidence, and open risks/questions.
The orchestrator performs the repository edits and records that execution used orchestrator integration from worker handoff.

## Review And Commit Gates

- `qa-reviewer` is mandatory before implementation commit.
- `commit-steward` is mandatory after required validations are green.
- If any gate is waived, waiver reason must be recorded in both tracker and state.
- Required local preflight before push/merge: `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`.
- Required commitlint gate before push/merge:
  - inspect pending subjects with `git log --format=%s origin/develop..HEAD`
  - only allow commitlint types: `build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test|merge`
  - if a non-conforming subject is found, rewrite commit message(s) locally before push.

## Documentation Synchronization

`docs-tracker` keeps these synchronized at all times:

- `docs/task-tracker.txt`
- `state/repo-state.json`
- active phase docs under `docs/phases/<NN-slug>/`
- `docs/streams/incident-resolutions.md` when a cross-stream incident/root-cause fix is discovered

Synchronization is complete only when task status, phase status, ownership, branch/worktree, and validation checkpoints match.

## Incident Resolution Log (Mandatory Usage)

When a failure is diagnosed with a concrete root cause and fix path (especially CI/release/runtime startup failures), agents must:

1. Record it in `docs/streams/incident-resolutions.md`.
2. Link the incident entry from the active phase/stream doc.
3. Include resolution evidence + rollback trigger.

## Manual Playtest Gate

For user-visible interactive features:

- add or update playable scenario under `examples/*.py`
- run user playtest and record feedback evidence in tracker/state
- phase cannot close without this evidence when the gate applies
