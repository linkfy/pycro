# Requirements

phase_objective: Produce an Android build flow for external `pycro` projects using the canonical project contract and shared embedded payload architecture.

## Acceptance Criteria

- `pycro project build --project <path> --target android` is defined as the supported Android build command.
- The build consumes the canonical project structure defined in phase 14.
- The phase documents a single concrete v1 output contract. Recommended v1: generated Android project or APK flow compiled from `pycro` sources with embedded project payload.
- Project scripts and assets are packaged into Android-compatible resources/assets through the shared embedded payload strategy rather than loose copied source files.
- The Android target does not assume arbitrary host filesystem access at runtime.
- Toolchain prerequisites and host constraints are documented explicitly.

## Constraints

- Do not change desktop or web packaging contracts in this phase.
- Do not broaden the phase into iOS support.
