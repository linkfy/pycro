# Task Implementation

## Execution Steps
1. Confirm startup gate validity and synchronize phase 20 docs/tracker/state.
2. Implement development-mode `.py` change detection across project subdirectories.
3. Wire reload flow into runtime lifecycle with explicit embedded-mode exclusion.
4. Implement graphical error overlay rendering for startup and runtime exceptions.
5. Add scenario/test evidence for reload + error recovery and record checkpoint readiness.

## Task Board

| Task ID | Owner | Parallel Team | Status | Branch | Worktree | Validation Gate |
| --- | --- | --- | --- | --- | --- | --- |
| dev-hot-reload-runtime-error-overlay | architecture-orchestrator | runtime-worker, platform-worker, api-worker, example-scenario-worker, docs-tracker, qa-reviewer, commit-steward | complete | codex/20-dev-hot-reload-runtime-overlay | .worktrees/20-dev-hot-reload-runtime-overlay-orchestrator | hot reload + error overlay scenario pass + standard preflight |

## Resume Checkpoint

- kickoff_date: 2026-03-27
- active_branch: `codex/20-dev-hot-reload-runtime-overlay`
- startup_gate: requirements + design validated, implementation opened
- next_slice:
  - implement file-change watcher/debounce for source-mode `.py` tree
  - add runtime reload path and keep embedded payload mode opt-out
  - add window error overlay path with terminal parity

## Delegated Slice Plan

| Slice | Owner | Scope | Exit Evidence |
| --- | --- | --- | --- |
| reload detection core | runtime-worker | `.py` recursive change detection + debounce + trigger state | runtime tests or deterministic harness evidence |
| reload lifecycle integration | runtime-worker | reload execution + safe failure state transitions | successful reload + no hard-exit on script error |
| error overlay rendering | platform-worker | draw overlay in macroquad loop for startup/runtime exceptions | visual/manual evidence with sample exception |
| API/docs contract sync | api-worker, docs-tracker | runtime contract docs and operator usage notes | tracker/state/docs synchronized |
| playable validation | example-scenario-worker, qa-reviewer | update/create scenario for live reload and overlay recovery | recorded pass evidence |

## Validation Gates

- Governance sync: `python3 scripts/validate_governance.py`
- Mandatory preflight:
  - `cargo fmt --all --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- API/stub/docs:
  - `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi`
  - `python3 -m mypy --config-file pyproject.toml`
  - `cargo doc --no-deps`
- New behavior gate:
  - dedicated hot-reload + overlay scenario evidence under `examples/`

## Checkpoint

- Create checkpoint commit immediately after required validations pass and tracker/state sync is complete.

## Progress Log

- 2026-03-27 (slice: runtime+platform core):
  - added recursive `**/*.py` source-mode change monitor with debounce (`PYCRO_RELOAD_DEBOUNCE_MS`, default `250ms`, clamped to `50..2000ms`)
  - wired source-only hot reload by rebuilding runtime VM on change detection
  - disabled hot reload automatically for embedded payload mode
  - replaced hard-exit-on-runtime-error flow with in-window error overlay rendering while preserving stderr output
  - added manual scenario `examples/phase20_hot_reload_error_overlay_lab.py`
  - added runtime tests covering monitor behavior (embedded opt-out + source debounce trigger)

## Validation Evidence (2026-03-27)

- `python3 scripts/validate_governance.py` (pass)
- `cargo fmt --all --check` (pass)
- `cargo clippy --all-targets -- -D warnings` (pass)
- `cargo test` (pass)
- `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi` (pass)
- `python3 -m mypy --config-file pyproject.toml` (pass)
- `cargo doc --no-deps` (pass)
