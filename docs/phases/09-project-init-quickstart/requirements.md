# Requirements

phase_objective: Add a first-run project bootstrap command for pycro users.

## Acceptance Criteria

- CLI supports `pycro init <project_name>`.
- Command creates a folder named exactly `<project_name>` (relative to current working directory).
- Generated project includes:
  - `main.py` with:
    - `import pycro`
    - global `text` variable
    - `BG_COLOR` constant using a dark gray/near-black theme value
    - `setup()` defining a simple starter text
    - `update(dt)` calling only `clear_background` and `draw_text`
  - `pycro.pyi` copied from canonical `python/pycro/__init__.pyi`.
- Command fails with a clear message if destination folder already exists.
- Existing script-run behavior remains functional.

## Constraints

- Keep requirements synchronized with `docs/task-tracker.txt` and `state/repo-state.json`.
- Preserve orchestrator-first and delegated workflow rules.
- If scope changes, update `interactive-refinement.md` first.
