# Design

## Implementation Approach

- Implement an iOS target adapter that consumes the shared embedded project payload and routes `ProjectBuildTarget::Ios` to a dedicated build path.
- Package the payload into iOS bundle resources rather than depending on host filesystem layouts or ad hoc source copying.
- Generate a deterministic Xcode project under `<project>/dist/ios/xcode/` as the v1 build artifact.
- Keep iOS toolchain orchestration isolated from desktop/web/Android build paths.
- Treat host requirements and simulator/device validation as first-class design inputs.
- Use `cargo apple` (cargo-mobile2) plus Xcode toolchain integration for the v1 orchestration path, with build-time payload embedding remaining the only project-content packaging mechanism.

## V1 Output Contract

- Source project contract:
  - `main.py` is required;
  - root sidecar `.py` files, optional `assets/**`, and optional `pycro-project.toml` are embedded into the generated iOS bundle resources.
- Output contract:
  - generated project path: `<project>/dist/ios/xcode/`;
  - generated project is suitable for opening in Xcode or driving `xcodebuild` on a macOS host;
  - signing, distribution packaging, and App Store delivery remain follow-up concerns outside v1.
- Host contract:
  - macOS host required;
  - Xcode command line tooling required;
  - `cargo apple` (cargo-mobile2) available on PATH for the iOS build orchestration flow.
  - for deterministic embedded runtime behavior in generated dist workspaces, Xcode builds should run with `PYCRO_EMBED_PROJECT_ROOT=<project_root>` (or rely on the generated `.pycro-embed-project-root` hint file).

## Compatibility Design Notes

- Generated dist workspace must remain self-contained enough for Xcode script phases:
  - include/link workspace metadata required by cargo-mobile2 (`Cargo.toml`, `Cargo.lock`, `mobile.toml`, `build.rs`, `src/`, `gen/`, `third_party/`) under `<project>/dist/ios/xcode/`;
  - provide `<project>/dist/target -> <repo>/target` linking for cargo-apple staticlib resolution.
- Xcode script phase must avoid strict failure on unset optional vars:
  - use `"${GCC_PREPROCESSOR_DEFINITIONS:-}"` to avoid shell aborts when undefined.
- Build script must prioritize embedded payload correctness over host filesystem assumptions:
  - payload is the source of truth on iOS runtime startup;
  - any fallback that triggers cargo-mobile2 interactive template initialization in dist output is considered a contract violation.

## ADR And Contract Alignment

- iOS platform guarantees and packaging assumptions must remain ADR-compatible with the shared `project` roadmap.
