# Design

## Implementation Approach

- Add a dev-runtime file-change detector rooted at the current project/script directory and filtered to `**/*.py`.
- Gate the detector behind runtime mode checks:
  - enabled for source mode (`pycro` running local scripts);
  - disabled for embedded payload mode.
- On change detection, rebuild/reload Python runtime state with explicit error capture boundaries.
- Add a runtime error presentation layer in backend rendering:
  - draw a simple full-screen panel with error headline + normalized message;
  - include compact traceback lines when available and truncation rules to avoid overflow.
- Keep terminal logging unchanged for diagnostics parity.

## Reload Contract

- Trigger: any `.py` write/create/remove under the project root tree.
- Debounce: reload only after a short quiet window to avoid repeated reload storms during save operations.
- Safety: if reload fails, engine remains alive and renders overlay instead of hard-exiting.

## Error Overlay Contract

- Error overlay is shown for startup and frame update errors that currently surface only via stderr.
- Overlay text must be legible and include the core Python exception message (e.g., `name 'Player' is not defined`).
- Overlay must update on next successful reload and disappear automatically when runtime recovers.

## Validation Strategy

- Add a dedicated dev scenario in `examples/`:
  - starts normally;
  - then intentionally introduces a script error;
  - verifies overlay appears;
  - then fixes file and verifies hot reload recovery.
- Preserve standard preflight gates and add focused hot-reload behavior tests where feasible.
