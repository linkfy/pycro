# Design

## Implementation Approach

- Implement an Android target adapter that consumes the shared embedded project payload.
- Package the payload into Android-compatible resources/assets instead of relying on direct filesystem layouts.
- Keep Android toolchain orchestration target-specific and isolated from other build paths.
- Document build host constraints and validation strategy as first-class parts of the phase design.

## ADR And Contract Alignment

- Android platform guarantees and packaging assumptions must remain ADR-compatible with the shared `project` roadmap.
