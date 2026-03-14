# Requirements

phase_objective: Accept Python numeric values (`int` + `float`) for Vec2 and Color component parsing by coercing to runtime `f32` automatically.

## Acceptance Criteria

- Runtime Vec2 parsing accepts numeric component values without requiring explicit Python float literals.
- Runtime Color parsing accepts numeric component values without requiring explicit Python float literals.
- Python API exposes `KEY` enum values (`KEY.ESCAPE`, `KEY.MOUSE_LEFT`, etc.) mapped to supported backend inputs.
- `is_key_down` accepts a `KEY` enum value as the primary typed contract.
- Runtime initialization includes Python stdlib availability for common imports (e.g. `typing`, `dataclasses`).
- `draw_texture`, `draw_circle`, and `draw_text` avoid `expected float at index ...` failures when provided valid numeric int components.
- Invalid non-numeric inputs still fail with contextual ValueError details.
- Tracker/state/phase docs are synchronized for phase 12 closeout.

## Constraints

- Keep API metadata/stub type aliases unchanged (`Vec2` and `Color` remain float-typed contracts).
- Do not change draw command ordering or render queue semantics.
- Preserve context-rich error messages for malformed payload shapes.
