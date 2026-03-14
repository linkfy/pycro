# Design

## Implementation Approach

- Implement an iOS target adapter that consumes the shared project bundle.
- Package the bundle into iOS bundle resources rather than depending on host filesystem layouts.
- Keep iOS toolchain orchestration isolated from desktop/web/Android build paths.
- Treat host requirements and simulator/device validation as first-class design inputs.

## ADR And Contract Alignment

- iOS platform guarantees and packaging assumptions must remain ADR-compatible with the shared `project` roadmap.
