# Phase 18-project-android-build - Project Android Build

status: closed
objective: Add Android-target project builds on top of the shared embedded project payload and target orchestration contracts.
tracked_tasks: project-android-build

## Links

- `docs/task-tracker.txt`
- `state/repo-state.json`

## Android Build Instructions

Run from repository root (`/Users/linkfy/Code/pycro`):

1. Recommended one-shot command:
   - `scripts/android_build_project.sh <project_path>`
   - example:
     - `scripts/android_build_project.sh /Users/linkfy/Downloads/pycrotest/example`
2. Manual command (if needed):
   - `ANDROID_HOME="$HOME/Library/Android/sdk" ANDROID_SDK_ROOT="$HOME/Library/Android/sdk" NDK_HOME="$HOME/Library/Android/sdk/ndk/21.4.7075529" CARGO="$(rustup which cargo)" ./target/release/pycro project build --project <project_path> --target android`
3. Output location:
   - `<project_path>/dist/android/apk/*.apk`

## Notes

- `pycro-release.apk` may appear in `target/android-artifacts/release/apk/`, but it is not a valid Android package for install in this flow.
- Install/distribute `pycro.apk` from `<project_path>/dist/android/apk/`.
- Android 14 devices commonly require `arm64-v8a`. Verify the APK contains it:
  - `unzip -l <project_path>/dist/android/apk/pycro.apk | rg 'lib/.*/libpycro.so'`
  - expected entries include `lib/arm64-v8a/libpycro.so`.

## Troubleshooting

1. `failed to parse the edition key`:
   - cause: old Cargo binary is being used.
   - fix:
     - `rustup update stable`
     - `export CARGO="$(rustup which cargo)"`
     - rerun the build command.
2. `Please set the path to the Android NDK with the $NDK_HOME environment variable`:
   - fix:
     - `export ANDROID_HOME="$HOME/Library/Android/sdk"`
     - `export ANDROID_SDK_ROOT="$ANDROID_HOME"`
     - `export NDK_HOME="$ANDROID_HOME/ndk/21.4.7075529"`
     - rerun the build command.
3. Apple Silicon + NDK 21 compatibility shims (required by `cargo-quad-apk` expectations):
   - `ln -sfn "$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/lib64/clang/9.0.9/lib/linux/libclang_rt.builtins-aarch64-android.a" "$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/lib64/clang/9.0.9/lib/linux/aarch64/libunwind.a"`
   - `ln -sfn "$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android-ld" "$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/ld"`

## Evidence And Root Cause Summary (2026-03-19)

Observed evidence from local/device validation:
- build reached green and copied artifact to `<project>/dist/android/apk/pycro.apk`
- install succeeded on Android 14 device (`adb install -r .../pycro.apk`)
- app process remained alive after launch (`adb shell pidof rust.pycro` returned a pid)
- no new native crash trace (`F/DEBUG`, `Fatal signal`) after final packaging flow

Main failure reasons found during incident:
- old Cargo in path caused `edition = 2024` parse errors
- missing `NDK_HOME` prevented `cargo-quad-apk` from starting
- Apple Silicon + NDK linker expectations required `libunwind.a` and `ld` shims
- confusion between `pycro-release.apk` (non-installable in this flow) and valid `pycro.apk`
- Android runtime path drift caused `main.py` not found in some debug iterations
- frame loop termination behavior on Android could trigger premature exit

Corrections that must stay in place:
- always run with Rustup Cargo (`CARGO="$(rustup which cargo)"`)
- always export `ANDROID_HOME`, `ANDROID_SDK_ROOT`, `NDK_HOME`
- use `scripts/android_build_project.sh` to apply env + Apple Silicon shims automatically
- install only `<project>/dist/android/apk/pycro.apk`
- preserve Android build path contract:
  - `pycro project build --project <path> --target android`
  - artifacts copied from `target/android-artifacts/release/apk` to `<project>/dist/android/apk`
  - embedded payload built with `PYCRO_EMBED_PROJECT_ROOT`
