# Interactive Refinement

Use this document to record requirement and scope adjustments discovered during execution.

## Update Rule

When refinement changes phase scope or task ordering:

1. Update this file first.
2. Update `implementation.md` task board.
3. Sync `docs/task-tracker.txt`.
4. Sync `state/repo-state.json`.

No change is considered active until all four artifacts are synchronized.

## Refinement Log

- 2026-03-19: Locked the phase 19 v1 contract to a generated Xcode project under `<project>/dist/ios/xcode/`, with macOS/Xcode/`cargo apple` (cargo-mobile2) host requirements and a dedicated `scripts/phase19_ios_embedded_smoke.py` gate.
- 2026-03-19: Implemented CLI orchestration for `--target ios` in `src/main.rs` and added phase scripts (`scripts/phase19_ios_embedded_smoke.py`, `scripts/ios_build_project.sh`). Smoke execution reached the iOS toolchain gate and failed with missing `cargo apple` (cargo-mobile2) on host.
- 2026-03-20: Resolved Xcode script hard-fail on undefined optional preprocess definitions by defaulting `GCC_PREPROCESSOR_DEFINITIONS` expansion in the generated build script.
- 2026-03-20: Added dist-workspace context mirroring (`Cargo.toml`, `Cargo.lock`, `mobile.toml`, `build.rs`, `src`, `gen`, `third_party`) and target path linkage to stop `workspace root not found` failures in Xcode-launched builds.
- 2026-03-21: Added embed-root hint propagation (`.pycro-embed-project-root`) and script fallback so generated Xcode projects can recover embedded project root without user-specific absolute script edits.
- 2026-03-21: Confirmed runtime black-screen variant was a payload absence case (`embedded runtime requested but payload is not present`) and standardized deterministic E2E path: clean `xcodebuild` from dist workspace with `PYCRO_EMBED_PROJECT_ROOT=<project_root>`.
- 2026-03-21: Added generic operator script `scripts/ios_e2e_project.sh` and normalized path examples in iOS/Android helper scripts to avoid user-specific paths.
