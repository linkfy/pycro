# Interactive Refinement

Use this document to record requirement and scope adjustments discovered during execution.

## Update Rule

When refinement changes phase scope or task ordering:

1. Update this file first.
2. Update `implementation.md` task board.
3. Sync `docs/task-tracker.txt`.
4. Sync `state/repo-state.json`.

No change is considered active until all four artifacts are synchronized.

## 2026-03-14

- User requested phase setup for develop-first delivery.
- New governance requirement: all routine merges/pushes target `develop`.
- Release handoff requirement: manual ready-for-release PR from `develop` to `main`.
- Validation requirement: downloadable artifacts must be generated on every `develop` push.
- Operational bootstrapping: initialized and pushed `develop` branch to origin so the new artifact workflow has an active branch target.
