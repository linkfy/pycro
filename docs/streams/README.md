# Streams Index

This directory holds non-linear workstreams that should not break sequential phase numbering.

## Active Streams

- `performance/phase-03-plan.md`: performance investigation protocol and execution slices.
- `performance/phase-03-closeout.md`: closeout status, pending items, and follow-on proposals.
- `compare-benchmark.md`: isolated pycro vs pygame benchmark stream status.
- `backend-selection-policy.md`: future backend policy proposal stream.
- `windows-input-fix.md`: Windows keyboard input reliability investigation and patch plan.
- `incident-resolutions.md`: cross-stream incident log (root cause + resolution + rollback triggers).

## Stream Closeout Checklist (Mandatory)

Before marking a stream `complete`:

1. Document outcome and residual risks in the stream document.
2. Record resolved incidents or root-cause entries in `incident-resolutions.md` (or link explicit waiver).
3. Synchronize stream status/owner/doc in `docs/task-tracker.txt` and `state/repo-state.json`.
4. Confirm active phase remains canonical in tracker/state when the stream is parallel.
5. Record validation evidence and rollback trigger in tracker/state notes.
