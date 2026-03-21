# Task Implementation

## Execution Steps
1. Confirm the startup gate remains valid and the phase 19 docs are synchronized on the iOS v1 output contract.
2. Implement target parsing and build orchestration for `--target ios`.
3. Generate the v1 Xcode-project output under `<project>/dist/ios/xcode/` and package project payload/assets into the target layout.
4. Add smoke validation for generated-project startup assumptions and embedded bundle resources.
5. Close the phase only after tracker/state sync, validation evidence, and a checkpoint handoff are recorded.

## Task Board

| Task ID | Owner | Parallel Team | Status | Branch | Worktree | Validation Gate |
| --- | --- | --- | --- | --- | --- | --- |
| project-ios-build | architecture-orchestrator | runtime-worker, platform-worker, api-worker, docs-tracker, qa-reviewer, commit-steward | complete | codex/19-project-ios-build | .worktrees/19-project-ios-build-orchestrator | `scripts/phase19_ios_embedded_smoke.py` + standard preflight |

## Resume Checkpoint

- kickoff_date: 2026-03-19
- active_branch: `codex/19-project-ios-build`
- startup_gate: requirements + design validated, implementation opened
- next_slice:
  - install/enable `cargo apple` (cargo-mobile2) on host and rerun iOS smoke gate
  - capture a passing run of `scripts/phase19_ios_embedded_smoke.py` and sync evidence in tracker/state
  - prepare checkpoint commit once smoke gate is green

## Delegated Slice Plan

| Slice | Owner | Scope | Exit Evidence |
| --- | --- | --- | --- |
| iOS contract lock | architecture-orchestrator | finalize `<project>/dist/ios/xcode/` v1 output contract and host/toolchain requirements | requirements/design sync, tracker/state sync |
| CLI/iOS adapter | platform-worker | route `ProjectBuildTarget::Ios` into iOS build orchestration with embedded payload parity | `pycro project build --target ios` produces the documented output layout |
| Embedded payload guard | runtime-worker | confirm the iOS build path preserves embedded payload assumptions and avoids host filesystem dependence | regression note or smoke evidence referencing embedded markers |
| Metadata/API parity | api-worker | ensure build-command docs and public API metadata remain coherent with the new target contract | API/docs review notes |
| Sync + QA + checkpoint | docs-tracker, qa-reviewer, commit-steward | docs/state sync, findings gate, checkpoint handoff | tracker/state sync + QA pass/waiver + checkpoint reference |

## Reporting Contract

All contributors report summary-only payloads to the orchestrator:

- changed files
- validation evidence
- risks
- follow-ups
- ADR refs
- tracker refs

## Validation Gates

- iOS packaging smoke: `python3 scripts/phase19_ios_embedded_smoke.py`
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

- Create a checkpoint commit immediately after the validation gates pass and the tracker/state/doc trio is synchronized.

## Execution Evidence (2026-03-19)

- Implemented:
  - `src/main.rs`: iOS adapter `run_ios_project_build(...)` wired from `ProjectBuildTarget::Ios`.
  - `scripts/phase19_ios_embedded_smoke.py`: phase smoke for output contract + embedded payload markers.
  - `scripts/ios_build_project.sh`: operator helper for local iOS builds.
- Validation:
  - pass: `python3 scripts/validate_governance.py`
  - pass: `cargo fmt --all --check`
  - pass: `cargo clippy --all-targets -- -D warnings`
  - pass: `cargo test`
  - pass: `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi`
  - pass: `python3 -m mypy --config-file pyproject.toml`
  - pass: `cargo doc --no-deps`
  - pass: `bash -n scripts/ios_build_project.sh`
  - pass: `python3 -m py_compile scripts/phase19_ios_embedded_smoke.py`
  - blocked: `python3 scripts/phase19_ios_embedded_smoke.py` (`cargo apple` from cargo-mobile2 not installed on host)

## Execution Evidence (2026-03-21)

- Implemented and validated:
  - iOS dist workspace context mirroring under `<project>/dist/ios/xcode/` so Xcode script phases resolve the Rust workspace contract.
  - generated embed-root hint file support (`.pycro-embed-project-root`) plus script-phase env fallback.
  - shell hardening for optional Xcode vars (`GCC_PREPROCESSOR_DEFINITIONS` default expansion) to avoid null/unset aborts.
  - generic operator scripts:
    - `scripts/ios_e2e_project.sh` (new, parameterized E2E flow)
    - `scripts/ios_build_project.sh` usage normalized to generic paths
    - `scripts/android_build_project.sh` usage normalized to generic paths
- End-to-end iOS verification:
  - pass: `CARGO="$(rustup which cargo)" /tmp/pycro-target/release/pycro project build --project <project> --target ios`
  - pass: `PYCRO_EMBED_PROJECT_ROOT=<project> xcodebuild -workspace <project>/dist/ios/xcode/pycro.xcodeproj/project.xcworkspace -scheme pycro_iOS -sdk iphonesimulator -configuration release -derivedDataPath /tmp/pycro-ios-dd clean build`
  - pass: simulator install/launch + screenshot evidence (`/tmp/pycro-ios-e2e-final-ok.png`)
  - pass: runtime launched embedded payload path and rendered project content (no Bevy bootstrap screen)

## Problems Found, Root Cause, Solution

| Problem | Root Cause | Solution Applied | Preventive Measure |
| --- | --- | --- | --- |
| `iOS workspace root not found; expected Cargo.toml ...` in Xcode | Generated dist workspace lacked required Rust workspace metadata near `SRCROOT` | Mirror/link required files and directories into `<project>/dist/ios/xcode/` in `prepare_ios_xcode_rust_context(...)` | Keep dist workspace self-contained for cargo-mobile2 script assumptions; enforce in smoke checks |
| Xcode script aborted with `GCC_PREPROCESSOR_DEFINITIONS: parameter null or not set` | Shell script treated optional Xcode var as mandatory | Use `"${GCC_PREPROCESSOR_DEFINITIONS:-}"` in generated script invocation | Keep all optional shell expansions guarded with defaults |
| Runtime launched black screen with `embedded runtime requested but payload is not present` | Build path sometimes compiled without embedded project root context | Build dist workspace with `PYCRO_EMBED_PROJECT_ROOT=<project>` and generated hint fallback | Standardize E2E command to set embed root explicitly |
| Bevy logo/template behavior appeared in generated output | cargo-mobile2 interactive bootstrap triggered in wrong directory when workspace context was missing | Prevent interactive fallback by ensuring `Cargo.toml` + workspace context exist under dist workspace | Detect/forbid interactive template generation in automation path |
| `simctl` instability (`CoreSimulatorService connection invalid`) | Simulator daemon transient instability | Open Simulator app first, then boot/install/launch via `simctl` | E2E script now sequences `open -a Simulator` and boot checks before install |
| `tracing-oslog` target triple failure (`arm64-apple-ios-sim` invalid) | Upstream build script expected canonical Rust target triple naming | Local patching/alignment to supported iOS-sim target naming and toolchain path | Keep patched dependency checks in iOS smoke path and monitor lockfile drift |

## Standardized Commands (Generic)

- iOS dist generation:
  - `CARGO="$(rustup which cargo)" ./target/release/pycro project build --project /path/to/project --target ios`
- iOS E2E (recommended, generic):
  - `scripts/ios_e2e_project.sh --project /path/to/project`
- Android dist generation:
  - `scripts/android_build_project.sh /path/to/project`

## Future Measures Before Formal Closeout

1. Add a dedicated non-interactive regression check that fails if cargo-mobile2 attempts template initialization under dist output.
2. Extend `scripts/phase19_ios_embedded_smoke.py` to assert embed-root hint integrity and script-phase workspace root discoverability.
3. Capture one clean CI-style host runbook for macOS/Xcode simulator prerequisites (including simulator boot recovery).
4. Keep operator scripts path-agnostic and avoid user-specific paths in docs/examples.
5. Preserve periodic build-cache hygiene guidance (safe cleanup for large `target/` growth) to reduce host-level instability and disk pressure during repeated mobile builds.
