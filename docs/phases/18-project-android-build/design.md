# Design

## Implementation Approach

- Implement `run_android_project_build(project_root)` in CLI build orchestration, parallel to desktop/web adapters.
- Keep the shared compile-time payload path (`PYCRO_EMBED_PROJECT_ROOT`) as the only project content packaging mechanism.
- Invoke `cargo quad-apk build --release --bin pycro` from repository root and copy generated APK outputs from `target/android-artifacts/release/apk` into `<project>/dist/android/apk/`.
- Emit deterministic operator feedback: selected Android mode (docker/local), source artifact directory, destination artifact directory, and failure diagnostics.
- Keep Android toolchain orchestration target-specific and isolated from desktop/web implementations.
- Add a dedicated phase smoke script (`scripts/phase18_android_embedded_smoke.py`) that checks artifact layout and embedded payload metadata markers.

## Target-Specific Build Modes

- Local mode:
  - requires installed `cargo-quad-apk`, Android SDK/NDK, and configured `ANDROID_HOME` + `NDK_HOME`;
  - preferred when host already has Android build toolchain.
- Docker mode:
  - uses `notfl3/cargo-apk` image to avoid host SDK/NDK drift;
  - command mounts repository root and project root to preserve embedded payload environment contract.
- v1 implementation can default to local mode with explicit error guidance, then optionally expose a Docker fallback switch if needed by validation results.

## Cargo Metadata Contract

- Add/validate `[package.metadata.android]` defaults in root `Cargo.toml`:
  - package identity (`package_name`, `label`);
  - SDK levels (`android_version`, `target_sdk_version`, `min_sdk_version`);
  - build targets (`build_targets`);
  - modern activity compatibility (`[package.metadata.android.activity_attributes]` with `"android:exported" = "true"` for API 31+ compatibility).
- Keep metadata stable and phase-scoped; any lifecycle/public API/governance contract change still follows ADR policy.

## ADR And Contract Alignment

- Android platform guarantees and packaging assumptions must remain ADR-compatible with the shared `project` roadmap.
- If Android output contract shifts from APK-first to AAB-first within this phase, record the change in an ADR before merge.
