# Task Implementation

## Execution Steps
1. Validate phase 21 startup gate and ownership assignment after phase 20 closeout.
2. Add API metadata and runtime/backend dispatch wiring for `get_window_size()`.
3. Regenerate stub contract and add tests/examples.
4. Run full validation gates and capture checkpoint evidence.

## Task Board

| Task ID | Owner | Parallel Team | Status | Branch | Worktree | Validation Gate |
| --- | --- | --- | --- | --- | --- | --- |
| window-size-api-command | architecture-orchestrator | api-worker, runtime-worker, platform-worker, example-scenario-worker, docs-tracker, qa-reviewer, commit-steward | planned | codex/21-window-size-api-command | .worktrees/21-window-size-api-command-orchestrator | API test pass + stub drift check + example validation |

## Resume Checkpoint

- kickoff_date: 2026-03-27
- active_branch: `codex/21-window-size-api-command`
- startup_gate: requirements + design validated, implementation queued
- next_slice:
  - finalize return type shape decision and doc contract
  - implement API wiring and backend adapter
  - validate stubs + examples

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
