# Agent Playbook

## Core Rule

The orchestrator ingests summaries, not raw logs. Every worker reports only:

- changed files
- validation evidence
- open risks
- follow-up tasks
- ADR or tracker references

The main thread should avoid carrying large implementation context directly; delegate domain implementation to the team and keep the orchestrator focused on integration and gate decisions.

## Required Flows

### Feature Delivery

- `architecture-orchestrator`
- one or two domain workers
- `example-scenario-worker` when a new runtime/platform/API feature needs a playable scenario in `examples/`
- `qa-reviewer`
- `docs-tracker`
- `flow-visualizer` when the change affects lifecycle, dispatch, or integration sequencing
- `commit-steward` to create the checkpoint commit once validations are green and gates are satisfied

### Research Or Spike

- `architecture-orchestrator`
- explorer agents only
- no repo mutations

### Performance Slice (FPS)

- `architecture-orchestrator`
- one or two runtime/platform workers for Rust-only optimizations
- benchmark investigators (explorer) in parallel
- `docs-tracker`
- `perf-study-recorder` in parallel after every positive result to append `mejoras_a_esutdiar.md`
- `perf-study-recorder` is a hard gate for FPS work: after every positive benchmark delta, spawn it immediately before starting the next optimization slice.
- `qa-reviewer` before any implementation commit

### Incident Fix

- `architecture-orchestrator`
- one relevant domain worker
- `qa-reviewer`

## Review Gate

`qa-reviewer` is mandatory before an implementation commit. Findings block the commit until resolved or explicitly waived in the tracker and state file.

## Commit Gate

`commit-steward` is mandatory after green validations for implementation work. The steward creates a checkpoint commit immediately, or reports an explicit blocking reason if commit preconditions are not met.

## Integration Branch Rule

- `main` is the repository integration branch.
- Agents must implement on `codex/<domain>-<task>` branches and merge into `main` only after validation gates are satisfied.
- Before requesting/performing merge to `main`, confirm expected behavior with runnable evidence (tests/build + benchmark or scenario checks when relevant).
- Before ending a delivery block, ask the user explicitly whether they want the branch merged into `main`.
- If the branch is not merged, tracker quick-index entries that remain `[-]` must include `-> (Not merged)` for that work item.

## Manual Playtest Gate

For user-visible engine features (for example: textures, movement, FX, audio, camera), `example-scenario-worker` must add or update a dedicated playable case under `examples/`. The final acceptance gate requires explicit user feedback after running the scenario, because agents cannot fully validate interactive event-loop behavior autonomously.
Example layout rule: scenario scripts live at `examples/*.py` (flat). Shared assets live at `examples/assets/`.
Texture policy for `example-scenario-worker`: texture-driven scenarios must default to `examples/assets/kenney_development_essentials/` assets and only use ad-hoc assets when the Kenney pack cannot cover the scenario.
Scenario documentation policy: every new texture scenario must state which pack files it uses in `examples/README.md`.

## Documentation Discipline

- `docs-tracker` continuously maintains sync between `docs/task-tracker.txt` and `state/repo-state.json` (objective status, task status, task order, and quick-index roadmap entries).
- `docs-tracker` keeps future phase roadmap items visible as unchecked `[ ]` quick-index entries until those phases are activated.
- `docs-tracker` updates tracker-linked ADR references and machine state summary fields without copying raw execution logs.
- Tracked task closure is mandatory: when a tracked task is delivered and merged into `main`, mark it as complete in both tracker and state before ending the work block.
- `perf-study-recorder` appends every measured positive FPS delta to `mejoras_a_esutdiar.md` in tutorial format: technique, why it helps, risk, validation protocol, and benchmark evidence.
- `flow-visualizer` updates Mermaid diagrams so lifecycle and dispatch behavior are reviewable without reading implementation code first.
- The machine state file is a compact snapshot for agents. Do not append transcripts or verbose logs.
- Before every phase commit, documentation must be refreshed and recompiled (for example `cargo doc --no-deps` and stub/cheatsheet refresh checks), with concise evidence captured in tracker/state.

## Benchmark Integrity Rule (Python Gameplay)

- Agent-delivered performance claims must reflect the real pycro user model: gameplay logic authored and executed in Python.
- Do not present FPS gains as canonical if they come from moving user gameplay loops/entities/simulation logic into Rust internals.
- Engine/runtime overhead optimizations are valid when equivalent gameplay remains in Python scripts.
- Rust-side automation of gameplay logic is allowed only as diagnostic instrumentation and must be reported separately from canonical benchmark outcomes.
