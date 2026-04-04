# Task Implementation

## Execution Steps

1. Validate phase 23 startup gate and ownership assignment.
2. Add API metadata + runtime/backend bridge for `get_mouse_position()`.
3. Expand keyboard alias mapping and `KEY` enum coverage end-to-end.
4. Add regression tests and run validation gates.
5. Synchronize tracker/state and prepare checkpoint commit.

## Task Board

| Task ID | Owner | Parallel Team | Status | Branch | Worktree | Validation Gate |
| --- | --- | --- | --- | --- | --- | --- |
| input-mouse-position-full-keyboard-keys | architecture-orchestrator | runtime-worker, api-worker, platform-worker, docs-tracker, qa-reviewer, commit-steward, example-scenario-worker | complete | codex/23-mouse-position-full-keyboard-enum | .worktrees/23-mouse-position-full-keyboard-enum-orchestrator | runtime tests + stub drift + preflight |

## Resume Checkpoint

- kickoff_date: 2026-03-28
- active_branch: `codex/23-mouse-position-full-keyboard-enum`
- startup_gate: requirements + design validated, implementation completed
- completion_summary:
  - keyboard alias + enum exposure contract completed end-to-end
  - macOS left-click latency resolved with native binding path + window-bounds gating
  - closeout artifacts synchronized (phase docs + tracker + state)

## Issue Resolution

- Left-click input delay/intermittent miss on macOS was resolved in phase scope.
- Resolution path:
  - native macOS left-click polling in binding path (no backend fallback for left click on macOS)
  - cursor-inside-window gate so clicks outside the pycro window are ignored
  - patched `readmouse` dependency state source to improve in-window reliability
- Status: resolved and user-validated.

## Diagnostic Scenarios

These scripts provide consistent edge-detection logging (press/release) plus on-screen state, using:
`get_mouse_position`, `put_pixel`, and `draw_line`.

- `examples/input_click_left_diagnostic.py`
- `examples/input_click_right_diagnostic.py`
- `examples/input_click_middle_diagnostic.py`

Run (one at a time):

```bash
python3 -m pycro_cli examples/input_click_left_diagnostic.py
python3 -m pycro_cli examples/input_click_right_diagnostic.py
python3 -m pycro_cli examples/input_click_middle_diagnostic.py
```

### Investigation References

- macroquad issue thread (input/click behavior context): https://github.com/not-fl3/macroquad/issues/422
- macroquad commit reviewed during investigation (enter/leave mouse handling): https://github.com/not-fl3/macroquad/commit/6b0de95a8b36cae5503107549eb6a7894bd45b7e
- upstream miniquad PR linked from investigation path: https://github.com/not-fl3/miniquad/pull/545

## Validation Evidence

- 2026-03-28:
  - `cargo fmt --all --check`
  - `cargo test -q`
  - `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi`
  - `python3 scripts/validate_governance.py`
- 2026-04-04:
  - `cargo test --lib backend::tests::key_code_from_name_supports_all_non_mouse_key_enum_values -- --nocapture`
  - `cargo test --lib backend::tests::key_code_from_name_rejects_unknown_keys -- --nocapture`
  - `cargo test --lib runtime::tests::is_key_down_accepts_key_enum_values -- --nocapture`
  - `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi`

## Validation Gates

- Governance sync: `python3 scripts/validate_governance.py`
- Mandatory preflight:
  - `cargo fmt --all --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- API/stub checks:
  - `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi`
