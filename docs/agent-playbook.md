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

### Incident Fix

- `architecture-orchestrator`
- one relevant domain worker
- `qa-reviewer`

## Review Gate

`qa-reviewer` is mandatory before an implementation commit. Findings block the commit until resolved or explicitly waived in the tracker and state file.

## Commit Gate

`commit-steward` is mandatory after green validations for implementation work. The steward creates a checkpoint commit immediately, or reports an explicit blocking reason if commit preconditions are not met.

## Manual Playtest Gate

For user-visible engine features (for example: textures, movement, FX, audio, camera), `example-scenario-worker` must add or update a dedicated playable case under `examples/`. The final acceptance gate requires explicit user feedback after running the scenario, because agents cannot fully validate interactive event-loop behavior autonomously.

## Documentation Discipline

- `docs-tracker` updates the task tracker, ADR references, and machine state summary.
- `flow-visualizer` updates Mermaid diagrams so lifecycle and dispatch behavior are reviewable without reading implementation code first.
- The machine state file is a compact snapshot for agents. Do not append transcripts or verbose logs.
