# Requirements

phase_objective: Simplify pycro script lifecycle by removing framework-owned setup hook.

## Acceptance Criteria

- Runtime no longer auto-calls `setup()` during script load.
- `update(dt)` remains the only required lifecycle callback.
- Existing scripts that define `setup()` still run (function may remain user-defined but not framework-dispatched).
- Docs/stubs/architecture references are updated to reflect update-only lifecycle.
- Tests are updated to validate update-only behavior and no setup dispatch.

## Constraints

- Keep tracker/state/phase docs synchronized.
- Preserve orchestrator-first workflow and QA gate.
- If scope changes during implementation, update `interactive-refinement.md` first.
