# ADR 0003: Docs-First Governance And Review Workflow

## Status

Accepted

## Decision

The repository is governed by canonical docs under `docs/`, a human-readable tracker in `docs/task-tracker.txt`, and a machine-readable state snapshot in `state/repo-state.json`. The orchestrator consumes worker summaries only, and `qa-reviewer` is the mandatory gate before implementation commits.

## Consequences

- Tracker and state drift is a validation failure.
- Review outcomes become part of commit readiness, not optional commentary.
- Branch discipline and ADR updates are enforced as operational policy.
