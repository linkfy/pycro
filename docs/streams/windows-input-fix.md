# Windows Input Reliability Stream

id: stream-windows-input-fix
status: in_progress
owner: architecture-orchestrator
branch: codex/stream-windows-input-fix-plan
scope: non-sequential fix stream (does not change phase numbering)

## Objective

Deliver a deterministic and reproducible fix path for Windows keyboard input failures where `pycro.is_key_down(...)` reports `False` for all tested keys in release artifacts.

## Immediate Start (After Context Reset)

Read and execute in this order:

1. This document (`docs/streams/windows-input-fix.md`).
2. `docs/task-tracker.txt` section `Immediate Next Workstream`.
3. `state/repo-state.json` stream entry `stream-windows-input-fix`.

Kickoff commands:

```bash
git checkout codex/stream-windows-input-fix-plan
python3 scripts/validate_governance.py
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test
```

First implementation slice to open:

- apply temporary `[patch.crates-io]` for `miniquad` Windows raw input test path;
- run Windows diagnostic scenario and capture evidence in tracker/state.

## Problem Statement

Observed user-facing behavior on Windows:

- Arrow keys and letter keys can report `False` persistently even when the game window is visible.
- The same script behaves correctly on macOS.

Current backend dependency chain:

- `pycro` input polling delegates to `macroquad::input::is_key_down`.
- `macroquad` delegates to `miniquad` platform input implementation.

## Evidence Captured

Upstream signals collected:

1. `miniquad` issue on Windows RAWINPUT target reliability (`hwndTarget = NULL`):
   - https://github.com/not-fl3/miniquad/issues/430
2. `miniquad` keyboard API reliability/rework thread:
   - https://github.com/not-fl3/miniquad/issues/517
3. Related macroquad keyboard inconsistency threads:
   - https://github.com/not-fl3/macroquad/issues/357
   - https://github.com/not-fl3/macroquad/issues/686

Version snapshot in this repo:

- `macroquad = 0.4.14`
- `miniquad = 0.4.8`

Source check against `miniquad` master still shows Windows RAWINPUT registration with `hwndTarget = NULL` at investigation time.

## Requirements (Fix Stream Gate)

1. Reproduce the Windows failure with a minimal script and deterministic run instructions.
2. Isolate whether failure reproduces in:
   - pure `macroquad` sample,
   - `pycro` runtime sample.
3. Validate if dependency patching `miniquad` resolves behavior.
4. Keep release line patch-only (`0.4.x`) and document rollback path.

## Design Strategy

Primary path:

1. Add a temporary `[patch.crates-io]` override for `miniquad` using a fork/commit with Windows RAWINPUT target adjustment (`hwndTarget = hwnd`).
2. Build Windows artifact with patched dependency.
3. Run scripted interactive validation (onscreen key-state HUD) and collect pass/fail evidence.

Fallback path:

1. If patch does not fix, add runtime telemetry mode (`PYCRO_INPUT_DEBUG=1`) to surface per-frame key state and focus data.
2. Use telemetry to distinguish:
   - no event arrival,
   - key mapping mismatch,
   - focus/activation issue.

## Implementation Plan

1. `runtime-worker`: add dependency patch wiring + guarded config docs.
2. `example-scenario-worker`: add `examples/windows_input_diagnostic.py` with visual key matrix.
3. `qa-reviewer`: verify with explicit Windows artifact run checklist.
4. `docs-tracker`: sync tracker/state + release notes preparation.

## Progress Update (2026-03-17)

- slice_1_status: complete (local implementation + non-Windows validation)
- applied temporary `[patch.crates-io]` override:
  - `Cargo.toml` -> `miniquad = { path = "patches/miniquad-0.4.8-windows-rawinput-fix" }`
  - patched file: `patches/miniquad-0.4.8-windows-rawinput-fix/src/native/windows.rs`
  - change: `rawinputdevice.hwndTarget = hwnd` (from `NULL`)
- added Windows diagnostic scenario:
  - `examples/windows_input_diagnostic.py`
  - displays real-time key-state HUD (`LEFT/RIGHT/UP/DOWN/SPACE/ESCAPE`) and movement cue
- local validation (host): `governance`, `fmt`, `clippy -D warnings`, `test`, `stub --check`, `mypy`, `cargo doc`, and short runtime smoke (`PYCRO_FRAMES=2`).

## Windows Tester Checklist (Pending Manual Gate)

1. Build a Windows desktop artifact from a project using this branch.
2. Run `pycro` with `examples/windows_input_diagnostic.py`.
3. Click inside the window to force focus.
4. Hold each key (`LEFT`, `RIGHT`, `UP`, `DOWN`, `SPACE`, `ESCAPE`) and confirm HUD switches to `True` immediately.
5. Confirm arrow keys move the cyan dot continuously while held.
6. Record pass/fail evidence in tracker/state before marking stream `complete`.

## Validation Checklist

- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `python3 scripts/validate_governance.py`
- `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi`
- `python3 -m mypy --config-file pyproject.toml`
- Windows manual validation evidence attached to tracker/state for this stream

## Risks

- Upstream patch drift when `macroquad`/`miniquad` versions change.
- Temporary fork maintenance burden.
- False positives from focus issues if validation does not enforce click-to-focus step.

## Exit Criteria

This stream can move to `complete` only when:

1. A Windows tester confirms key polling works in release artifact.
2. Evidence is recorded in tracker/state.
3. The fix path is documented as either:
   - upstream version adoption, or
   - temporary dependency patch with explicit removal trigger.
