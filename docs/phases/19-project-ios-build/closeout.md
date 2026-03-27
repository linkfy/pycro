# Closeout

status: closed
closeout_date: 2026-03-21

## Outcome

Phase 19 objective is complete:

- `pycro project build --project <path> --target ios` is implemented end-to-end.
- iOS output contract is stable under `<project>/dist/ios/xcode/`.
- Embedded payload contract is preserved on iOS runtime startup (no dependency on host project filesystem at app runtime).
- Dist-workspace compatibility issues were resolved (`workspace root not found`, unset script vars, embed-root propagation).
- Operator scripts are now path-generic for cross-machine use:
  - `scripts/ios_build_project.sh`
  - `scripts/ios_e2e_project.sh`
  - `scripts/android_build_project.sh` (generic path usage normalized in this closeout pass)

## Validation Evidence

- `python3 scripts/validate_governance.py`
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi`
- `python3 -m mypy --config-file pyproject.toml`
- `cargo doc --no-deps`
- `python3 scripts/phase19_ios_embedded_smoke.py`
- End-to-end simulator verification:
  - `scripts/ios_e2e_project.sh --project /path/to/project`
  - clean dist-workspace build with explicit embed root:
    - `PYCRO_EMBED_PROJECT_ROOT=/path/to/project xcodebuild -workspace /path/to/project/dist/ios/xcode/pycro.xcodeproj/project.xcworkspace -scheme pycro_iOS -sdk iphonesimulator -configuration release -derivedDataPath /tmp/pycro-ios-dd clean build`
  - simulator install/launch/screenshot confirms rendered project content.

## QA Outcome

- `qa-reviewer`: pass (iOS build contract, embedded runtime startup, generated Xcode handoff layout, and operator command path genericity verified for closeout scope).

## Follow-up

- Next queued work returns to active streams / next sequential phase planning.
- Keep `scripts/ios_e2e_project.sh` as the canonical local iOS verification path for future regressions.
