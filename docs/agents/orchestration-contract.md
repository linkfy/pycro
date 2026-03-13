# Orchestration Contract

## Core Rule

Every implementation task runs with an active orchestrator and delegated ownership.

- No "god agent" execution for end-to-end implementation.
- The orchestrator is integration-focused and context-thin.
- Domain workers own implementation slices and report concise summaries.

## Phase Start Gate (Mandatory)

At the beginning of every phase, the orchestrator must execute this startup sequence:

1. Validate that phase requirements are present and concrete in `docs/phases/<NN-slug>/requirements.md`.
2. Validate that design intent is aligned to those requirements in `docs/phases/<NN-slug>/design.md`.
3. Only then open implementation execution in `docs/phases/<NN-slug>/implementation.md`.

No implementation work should start before requirements and design are explicitly validated by the orchestrator.
If requirements are missing, weak, or ambiguous, the orchestrator must switch to planning mode and run a requirements-first planning cycle before any implementation delegation.

## Model Routing Rule (Mandatory)

- Planning mode is executed by the official orchestrator on ChatGPT 5.4.
- Default model for implementation/review/delegation operations is Codex 5.3 medium.
- Smaller models may be used only for low-risk mechanical tasks and require an explicit reason in the orchestrator summary.

## Natural Flow (Mandatory)

For every phase, execution order is fixed:

- requirements
- design
- implementation
- interactive refinement (when scope changes)

When refinement updates requirements or design, implementation must be re-synchronized before continuing.

Planning-mode fallback is mandatory for every phase:

- missing requirements -> planning mode
- planning outputs -> requirements, design, implementation steps
- execution resumes only after those artifacts are synchronized in phase docs + tracker + state

## Mandatory Delegation

For implementation work, orchestration must include:

- at least one domain worker (`runtime-worker`, `platform-worker`, or `api-worker`)
- all domain workers required by the phase scope (full team when scope is cross-domain)
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

## Local CI Preflight Rule (Mandatory)

Before any push, merge request, or phase closeout checkpoint, run local CI-equivalent minimum checks:

- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`

No gate is considered green until these pass locally and evidence is recorded.
