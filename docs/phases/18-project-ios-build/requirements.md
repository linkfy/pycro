# Requirements

phase_objective: Produce an iOS build flow for external `pycro` projects using the canonical project contract and shared target architecture.

## Acceptance Criteria

- `pycro project build --project <path> --target ios` is defined as the supported iOS build command.
- The build consumes the canonical project structure defined in phase 14.
- The phase documents a single concrete v1 output contract. Recommended v1: generated Xcode project rather than final signed app artifact.
- Project scripts and assets are packaged into iOS-compatible bundle resources.
- The iOS target does not assume arbitrary host filesystem access at runtime.
- Toolchain prerequisites and host constraints are documented explicitly, including macOS/Xcode dependencies.

## Constraints

- Do not redefine the canonical project structure or bundle contract in this phase unless a blocking incompatibility is discovered and recorded.
- Do not broaden the phase into Android or web support.
