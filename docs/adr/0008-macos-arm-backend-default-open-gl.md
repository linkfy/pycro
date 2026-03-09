# ADR 0008: macOS ARM Backend Default to OpenGL (Temporary)

## Status

Accepted (temporary)

## Decision

On `macOS` with `aarch64` architecture, `window_conf()` forces `AppleGfxApi::OpenGl` as the backend default.

This is a temporary operational decision to keep rendering behavior predictable while backend-selection policy is formalized in roadmap task `platform-backend-selection-policy`.

## Consequences

- Backend selection on Apple Silicon is explicit instead of implicit driver default.
- Metal remains a candidate backend, but switching requires explicit policy/evidence updates.
- Future backend policy work must define:
  - default backend per platform/architecture,
  - override mechanism,
  - scenario-based validation evidence for artifact/pacing behavior.
