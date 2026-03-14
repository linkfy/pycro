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

- User requested a new phase to avoid runtime failures when Vec2/Color payloads contain Python ints.
- Scope set to runtime numeric coercion with regression tests and a playable scenario.
- Scope expanded: add typed `KEY` enum (`KEY.ESCAPE`, `KEY.MOUSE_LEFT`, etc.) and route `is_key_down` through this contract.
- Error handling remains strict for non-numeric values and malformed payload shapes.
