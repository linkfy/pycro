# Interactive Refinement

Use this document to record requirement and scope adjustments discovered during execution.

## Update Rule

When refinement changes phase scope or task ordering:

1. Update this file first.
2. Update `implementation.md` task board.
3. Sync `docs/task-tracker.txt`.
4. Sync `state/repo-state.json`.

No change is considered active until all four artifacts are synchronized.

## 2026-03-18 - Android Build Planning Refinement

- Trigger:
  - Cross-check of phase 18 planning depth against official Macroquad Android guidance before implementation delegation launch.
- Refinement:
  - Locked v1 Android output contract to APK artifacts copied into `<project>/dist/android/apk`.
  - Added explicit toolchain modes (local SDK/NDK + Docker `notfl3/cargo-apk`) and cargo-quad-apk command path.
  - Added concrete delegated slice plan and Android smoke gate requirement.
- Synchronization:
  - updated `requirements.md`
  - updated `design.md`
  - updated `implementation.md`
  - updated `docs/task-tracker.txt`
  - updated `state/repo-state.json`
