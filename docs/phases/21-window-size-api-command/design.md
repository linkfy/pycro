# Design

## Implementation Approach

- Extend `api` metadata with `get_window_size()` including signature, docs, and platform support declaration.
- Route dispatch to backend window-size provider in runtime bridge.
- Ensure return values map cleanly to Python numeric types with minimal conversion overhead.
- Regenerate and validate stub output from canonical metadata source.

## Return Contract Proposal

- Preferred v1 shape: `(float, float)` representing `(width, height)`.
- Values represent the active rendering window dimensions for the current frame.
- If backend reports integer values internally, conversion to Python floats should remain explicit and documented.

## Validation Strategy

- Unit coverage for metadata + dispatch + result type.
- Example scenario under `examples/` that renders/prints dimensions in-frame.
- Standard governance/preflight/stub/type/doc gates.
