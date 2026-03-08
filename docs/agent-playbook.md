# Agent Playbook

## Core Rule

The orchestrator ingests summaries, not raw logs. Every worker reports only:

- changed files
- validation evidence
- open risks
- follow-up tasks
- ADR or tracker references

## Required Flows

### Feature Delivery

- `architecture-orchestrator`
- one or two domain workers
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

## Documentation Discipline

- `docs-tracker` updates the task tracker, ADR references, and machine state summary.
- `flow-visualizer` updates Mermaid diagrams so lifecycle and dispatch behavior are reviewable without reading implementation code first.
- The machine state file is a compact snapshot for agents. Do not append transcripts or verbose logs.
