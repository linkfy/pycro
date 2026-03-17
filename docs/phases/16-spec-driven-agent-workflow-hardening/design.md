# Design

## Implementation Approach

- Insert a new sequential phase 16 dedicated to workflow hardening and shift future build phases by +1.
- Treat this phase as a governance + tooling phase with small runtime/CLI ergonomics where they remove operational ambiguity.
- Add machine-checkable validation for cross-file phase consistency (`docs/phases/README.md`, `docs/task-tracker.txt`, `state/repo-state.json`).
- Normalize "active phase" vs "active stream" handling so parallel fix streams do not overwrite the canonical sequential phase target.
- Promote orchestration from preference to explicit phase-level contract recorded in docs/state.
- Normalize write-constrained worker operation as first-class workflow: workers analyze/propose, orchestrator integrates/edits, tracker/state record fallback mode.
- Add a reproducible kickoff contract:
  - canonical branch naming,
  - startup gate record,
  - synchronized doc/state activation.
- Add small operator-facing fixes that reduce false diagnosis during artifact testing:
  - real CLI `--help`,
  - documented artifact smoke expectations,
  - stream closeout checklist.

## ADR And Contract Alignment

- If this phase changes governance workflow contracts beyond current docs wording, add/update ADR before merge.
- Release automation, tracker/state synchronization, and agent playbook changes must remain aligned.
