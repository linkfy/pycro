# Design

## Implementation Approach

- Update runtime scalar parsing to use Python numeric coercion (`__float__`/real-number path) instead of float-only extraction.
- Reuse the existing Vec2/Color parsing pipeline so all render APIs benefit consistently.
- Introduce a typed Python `KEY` enum in module bootstrap/stubs and route `is_key_down` through enum values.
- Extend backend key mapping with mouse button inputs used by `KEY` (`MOUSE_LEFT`, `MOUSE_RIGHT`, `MOUSE_MIDDLE`).
- Initialize RustPython with stdlib module inits and stdlib path (`rustpython_pylib::LIB_PATH`) so imports like `typing` and `dataclasses` resolve.
- Add regression coverage for int-based Vec2/Color components in direct draw calls.
- Add a playable example in `examples/` demonstrating integer-based position/color usage.

## ADR And Contract Alignment

- This phase changes effective public API behavior (accepted argument forms), so an ADR update is required.
