# Closeout

status: closed
closeout_date: 2026-03-19

## Outcome

Phase 18 objective is complete:

- `pycro project build --project <path> --target android` is implemented end-to-end.
- Android build orchestration uses `cargo quad-apk build --release --bin pycro`.
- Artifacts are copied from repository `target/android-artifacts/release/apk` to `<project>/dist/android/apk`.
- Embedded payload contract is preserved via `PYCRO_EMBED_PROJECT_ROOT` without runtime dependency on host project filesystem.
- Android packaging helper `scripts/android_build_project.sh` documents and automates toolchain/env setup.

## Validation Evidence

- `python3 scripts/validate_governance.py`
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi`
- `python3 -m mypy --config-file pyproject.toml`
- `cargo doc --no-deps`
- `ANDROID_HOME="$HOME/Library/Android/sdk" ANDROID_SDK_ROOT="$HOME/Library/Android/sdk" NDK_HOME="$HOME/Library/Android/sdk/ndk/21.4.7075529" CARGO="$(rustup which cargo)" python3 scripts/phase18_android_embedded_smoke.py` (PASS)

## QA Outcome

- `qa-reviewer`: pass (no blocker findings for phase 18 closeout scope; remaining cautions are documented as follow-ups).

## Follow-up

- Next queued phase remains phase 19 (`project-ios-build`).
- Keep Android helper script/env instructions as canonical operator entrypoint for local packaging on macOS.
