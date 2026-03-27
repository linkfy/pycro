# Design

## Implementation Approach

- Extend `api` metadata with `get_window_size()` and `draw_rectangle()` including signatures, docs, and platform support declaration.
- Route dispatch to backend window-size provider in runtime bridge.
- Route `draw_rectangle()` dispatch through the existing graphics command path using the same numeric/color coercion conventions as other draw primitives.
- Ensure return values map cleanly to Python numeric types with minimal conversion overhead.
- Regenerate and validate stub output from canonical metadata source.

## Return Contract Proposal

- Preferred v1 shape: `(float, float)` representing `(width, height)`.
- Values represent the active rendering window dimensions for the current frame.
- If backend reports integer values internally, conversion to Python floats should remain explicit and documented.

## Rectangle Draw Contract Proposal

- Preferred v1 signature: `draw_rectangle(x: float, y: float, w: float, h: float, color: ColorLike)`.
- Numeric parameters follow existing coercion behavior used by other draw APIs.
- Rectangle draw semantics match backend immediate draw behavior for the current frame.

## Validation Strategy

- Unit coverage for metadata + dispatch + result type (window-size + rectangle draw command path).
- Example scenario under `examples/` that renders a rectangle while reading and using window dimensions in-frame.
- Standard governance/preflight/stub/type/doc gates.
