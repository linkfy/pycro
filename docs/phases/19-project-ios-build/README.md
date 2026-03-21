# Phase 19-project-ios-build - Project iOS Build

status: closed
objective: Add iOS-target project builds on top of the shared embedded project payload and target orchestration contracts, with a v1 Xcode-project output contract.
tracked_tasks: project-ios-build

## Links

- `docs/task-tracker.txt`
- `state/repo-state.json`
- `docs/phases/19-project-ios-build/requirements.md`
- `docs/phases/19-project-ios-build/design.md`
- `docs/phases/19-project-ios-build/implementation.md`
- `docs/phases/19-project-ios-build/interactive-refinement.md`

## Current Outcome Snapshot (2026-03-21)

- `pycro project build --project <path> --target ios` generates the expected output under `<project>/dist/ios/xcode/`.
- End-to-end simulator verification is now reproducible when Xcode build is executed from the generated dist workspace with embedded-project root explicitly set.
- Major compatibility blockers discovered during implementation are documented with mitigations and follow-up guardrails in `implementation.md`.
