# Task Implementation

## Execution Steps
1. Confirm startup gate remains valid (`requirements` + `design` synchronized with Macroquad Android build guidance).
2. Implement Android CLI adapter:
   - add `run_android_project_build(project_root)` and route `ProjectBuildTarget::Android` to it;
   - keep `--exe` rejected for non-desktop targets.
3. Implement Android build orchestration:
   - execute `cargo quad-apk build --release --bin pycro` from repository root;
   - pass `PYCRO_EMBED_PROJECT_ROOT=<project_root>`;
   - support actionable failure messages for missing toolchain (`cargo-quad-apk`, SDK/NDK env).
4. Implement v1 Android artifact contract:
   - source artifacts: `target/android-artifacts/release/apk`;
   - output artifacts: `<project>/dist/android/apk/*.apk`;
   - preserve deterministic copy/cleanup behavior across repeated builds.
5. Add/verify Android Cargo metadata baseline in root `Cargo.toml` (`[package.metadata.android]` + activity attributes/API defaults).
6. Add smoke validation (`scripts/phase18_android_embedded_smoke.py`) covering APK output presence + embedded payload marker checks.
7. Run full validation gates and synchronize tracker/state before checkpoint handoff.

## Task Board

| Task ID | Owner | Parallel Team | Status | Branch | Worktree | Validation Gate |
| --- | --- | --- | --- | --- | --- | --- |
| project-android-build | architecture-orchestrator | runtime-worker, platform-worker, api-worker, docs-tracker, qa-reviewer, commit-steward | complete | codex/18-project-android-build | .worktrees/18-project-android-build-orchestrator | android packaging smoke + standard preflight |

## Resume Checkpoint

- kickoff_date: 2026-03-18
- active_branch: `codex/18-project-android-build`
- startup_gate: requirements + design validated, implementation closed
- closeout: `docs/phases/18-project-android-build/closeout.md`
- next_slice:
  - phase 19 startup gate preparation (`project-ios-build`)

## Delegated Slice Plan

| Slice | Owner | Scope | Exit Evidence |
| --- | --- | --- | --- |
| CLI/android adapter | platform-worker | `src/main.rs` target dispatch + Android command execution + artifact copy contract | `pycro project build --target android` produces `dist/android/apk/*.apk` |
| Embedded runtime guard | runtime-worker | verify Android path still respects embedded payload extraction assumptions | regression tests or documented fallback for runtime extraction constraints |
| Metadata/stub parity | api-worker | confirm platform declarations/stub metadata remain coherent for Android status | stub check + API metadata review notes |
| Sync + QA + checkpoint | docs-tracker, qa-reviewer, commit-steward | docs/state sync, findings gate, checkpoint commit | tracker/state sync + QA pass/waiver + checkpoint SHA |

## Validation Gates

- Android packaging smoke: `python3 scripts/phase18_android_embedded_smoke.py`
- Governance sync: `python3 scripts/validate_governance.py`
- Mandatory preflight:
  - `cargo fmt --all --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- API/stub/docs:
  - `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi`
  - `python3 -m mypy --config-file pyproject.toml`
  - `cargo doc --no-deps`

## Reporting Contract

All contributors report summary-only payloads to the orchestrator:

- changed files
- validation evidence
- risks
- follow-ups
- ADR refs
- tracker refs
