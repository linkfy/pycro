# Design

## Implementation Approach

- Implement a web target adapter that consumes the canonical project bundle from phase 14.
- Emit a web-specific output layout (HTML/bootstrap + runtime artifacts + packaged project resources).
- Route project resource access through the runtime/provider abstraction rather than assuming host filesystem paths.
- Keep target-specific behavior isolated so desktop/mobile targets remain unaffected.

## ADR And Contract Alignment

- If web target support changes platform guarantees or runtime packaging assumptions, capture that through ADR before merge.
