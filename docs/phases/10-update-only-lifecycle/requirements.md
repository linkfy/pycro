# Requirements

phase_objective: Simplify pycro lifecycle and bootstrap defaults by removing framework-owned setup hook, standardizing default entry to `main.py`, and embedding a local runner copy at init time.

## Acceptance Criteria

- Runtime no longer auto-calls `setup()` during script load.
- `update(dt)` remains the only required lifecycle callback.
- Existing scripts that define `setup()` still run (function may remain user-defined but not framework-dispatched).
- CLI default run mode resolves `main.py` from the target project directory when no script argument is provided.
- `pycro init <project_name>` copies the current `pycro` executable into the generated project directory.
- Docs/stubs/architecture references are updated to reflect update-only lifecycle.
- Tests are updated to validate update-only behavior and no setup dispatch.

## Constraints

- Keep tracker/state/phase docs synchronized.
- Preserve orchestrator-first workflow and QA gate.
- If scope changes during implementation, update `interactive-refinement.md` first.
