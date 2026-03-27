# Requirements

phase_objective: Enable development-time hot reloading for Python files in source project mode and show startup/runtime failures in-window as a readable graphical error overlay.

## Acceptance Criteria

- `pycro` detects changes in `.py` files under the active project root, including nested subfolders.
- Hot reload is enabled only for non-embedded runtime mode (direct independent use), and is disabled when running embedded/project-build payloads.
- Reload scope includes the entry script and imported sidecar modules when changes are detected.
- Runtime must recover into the new script state without requiring full process restart for supported reload cases.
- If startup or runtime loading fails (for example `NameError: name 'Player' is not defined`), the error must be shown in the pycro render window, not only in terminal logs.
- Terminal error output remains available; in-window error rendering is additive, not a replacement.
- A reproducible scenario is documented under `examples/` for manual validation of both hot reload and in-window error visualization.

## Constraints

- Do not activate hot reload in embedded payload mode (`pycro project build` outputs).
- Do not introduce filesystem watchers that break platform portability without fallback strategy.
- Keep failure overlay safe for repeated errors (no crash loops from rendering the error UI).
