# Task Implementation

## Execution Steps
1. Validate phase 21 startup gate and ownership assignment after phase 20 closeout.
2. Add API metadata and runtime/backend dispatch wiring for `get_window_size()` and `draw_rectangle()`.
3. Regenerate stub contract and add tests/examples.
4. Run full validation gates and capture checkpoint evidence.

## Task Board

| Task ID | Owner | Parallel Team | Status | Branch | Worktree | Validation Gate |
| --- | --- | --- | --- | --- | --- | --- |
| window-size-api-command | architecture-orchestrator | api-worker, runtime-worker, platform-worker, example-scenario-worker, docs-tracker, qa-reviewer, commit-steward | complete | codex/21-window-size-api-command | .worktrees/21-window-size-api-command-orchestrator | API tests pass for `get_window_size()` + `draw_rectangle()`, stub drift check, and example validation |

## Resume Checkpoint

- kickoff_date: 2026-03-27
- active_branch: `codex/21-window-size-api-command`
- startup_gate: requirements + design validated, implementation queued
- next_slice:
  - phase closeout recorded (`closeout.md`)
  - tracker/state synchronized to complete + QA waiver
  - merge into `develop` after local preflight

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

## Checkpoint

- Create checkpoint commit once all gates are green and tracker/state synchronization is complete.

## Validation Evidence (2026-03-27)

- `cargo fmt --all --check` -> pass
- `cargo clippy --all-targets -- -D warnings` -> pass
- `cargo test` -> pass
- `cargo run --bin generate_stubs -- --write python/pycro/__init__.pyi` -> pass
- `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi` -> pass
- `python3 -m mypy --config-file pyproject.toml` -> pass
- `cargo doc --no-deps` -> pass

## Open Risk

- Manual runtime smoke for `examples/phase21_window_size_rectangle_lab.py` is currently blocked in this environment by an upstream miniquad Apple panic (`apple_util.rs` null pointer dereference).
