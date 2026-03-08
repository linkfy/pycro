# ADR 0005: Python Text Rendering API (`draw_text`)

## Status

Accepted

## Decision

Add a new public Python API function:

`draw_text(text: str, position: Vec2, font_size: float, color: Color) -> None`

The function is authored in `src/api.rs` metadata and is exposed through:

- runtime direct bridge callables in `src/runtime.rs`
- backend contract execution in `src/backend.rs`
- generated stub output in `python/pycro/__init__.pyi`

## Consequences

- Public API surface now includes screen text rendering for HUD/debug overlays and timer-like scenarios.
- Runtime no longer needs examples to rely on non-text fallbacks for clear numeric feedback.
- Validation must include stub drift + typing checks whenever text API signatures evolve.
