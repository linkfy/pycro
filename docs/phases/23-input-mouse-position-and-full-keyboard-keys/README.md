# Phase 23-input-mouse-position-and-full-keyboard-keys - Mouse Position + Full Keyboard KEYS

status: in_progress
objective: Expose mouse-position detection and full keyboard key coverage through `pycro` runtime APIs and the `KEY` enum contract used by the interpreter bridge.
tracked_tasks: input-mouse-position-full-keyboard-keys

## Links

- `docs/task-tracker.txt`
- `state/repo-state.json`
- `docs/phases/23-input-mouse-position-and-full-keyboard-keys/requirements.md`
- `docs/phases/23-input-mouse-position-and-full-keyboard-keys/design.md`
- `docs/phases/23-input-mouse-position-and-full-keyboard-keys/implementation.md`
- `docs/phases/23-input-mouse-position-and-full-keyboard-keys/interactive-refinement.md`

## Current Outcome Snapshot (2026-03-28)

- Startup gate validated (`requirements` + `design`), phase execution opened.
- Branch `codex/23-mouse-position-full-keyboard-enum` created for phase ownership.
- Implementation target defined: add `get_mouse_position()` bridge path and complete keyboard aliases for `KEY` enum exposure/runtime parsing.
