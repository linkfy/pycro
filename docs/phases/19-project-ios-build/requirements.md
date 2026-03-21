# Requirements

phase_objective: Produce an iOS build flow for external `pycro` projects using the canonical project contract and shared embedded payload architecture.

## Acceptance Criteria

- `pycro project build --project <path> --target ios` is defined as the supported iOS build command.
- The build consumes the canonical project structure defined in phase 14.
- The phase documents one concrete iOS v1 output contract:
  - build output is a generated Xcode project under `<project>/dist/ios/xcode/`;
  - the generated project is the handoff artifact for Xcode/xcodebuild-based follow-up, not a final signed `.ipa` or App Store submission artifact.
- Project scripts and assets are packaged into iOS-compatible bundle resources through the shared embedded payload strategy rather than loose copied source files.
- The iOS target does not assume arbitrary host filesystem access at runtime.
- Toolchain prerequisites and host constraints are documented explicitly, including macOS, Xcode / `xcodebuild`, and `cargo apple` (cargo-mobile2) dependencies.
- Operator-facing helper commands/scripts use generic paths (no user-specific absolute routes) so they are reusable across machines and projects.

## Constraints

- Do not redefine the canonical project structure or embedded payload contract in this phase unless a blocking incompatibility is discovered and recorded.
- Do not change the desktop, web, or Android build contracts while locking the iOS v1 contract.
- Do not broaden the phase into Android or web support.
