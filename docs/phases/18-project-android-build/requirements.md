# Requirements

phase_objective: Produce an Android build flow for external `pycro` projects using the canonical project contract and shared embedded payload architecture.

## Acceptance Criteria

- `pycro project build --project <path> --target android` is defined as the supported Android build command.
- The build consumes the canonical project structure defined in phase 14.
- The phase documents one concrete Android v1 output contract:
  - build command emits release APK artifacts into `dist/android/apk/` under the external project root;
  - APK content is built from `pycro` sources with compile-time embedded project payload (`main.py`, root sidecars, optional `assets/`, optional `pycro-project.toml`).
- Project scripts and assets are packaged into Android-compatible resources/assets through the shared embedded payload strategy rather than loose copied source files.
- The Android target does not assume arbitrary host filesystem access at runtime.
- Toolchain prerequisites and host constraints are documented explicitly.
- Android build orchestration is aligned with Macroquad + cargo-quad-apk guidance:
  - builder path: `cargo quad-apk build --release`;
  - output source path: `target/android-artifacts/release/apk`;
  - supported environment modes documented: Docker (`notfl3/cargo-apk`) and local SDK/NDK installation.
- Local build validation includes an Android packaging smoke check that verifies:
  - APK artifacts are produced in `dist/android/apk/`;
  - generated embedded payload metadata contains at least `main.py` and one sidecar/asset marker from the source project.

## Toolchain Baseline (v1)

- Required build tooling:
  - Rust toolchain + Android targets (`armv7-linux-androideabi`, `aarch64-linux-android`, `i686-linux-android`, optional `x86_64-linux-android`);
  - Rust/Cargo must support `edition = "2024"` (upgrade with `rustup update stable` + `rustup default stable` when host cargo is older);
  - if multiple Cargo binaries exist on host, builds should use Rustup Cargo explicitly (`CARGO="$(rustup which cargo)" ...`);
  - `cargo-quad-apk` (`cargo install cargo-quad-apk`) for local mode or Docker image `notfl3/cargo-apk`;
  - Android SDK + NDK (`ANDROID_HOME`, `NDK_HOME`) for local mode.
- Android metadata baseline is declared in root `Cargo.toml` under `[package.metadata.android]` with explicit package identity, versioning, and activity attributes required for modern API levels.

## Constraints

- Do not change desktop or web packaging contracts in this phase.
- Do not broaden the phase into iOS support.
- APK signing for Play Store and AAB conversion are documented as follow-up/release concerns, not v1 build blockers for this phase.
