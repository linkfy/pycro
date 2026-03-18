# ADR 0013: Vec2/Color Numeric Coercion For Python Runtime Calls

## Status

Accepted

## Context

The Python-facing API documents `Vec2` and `Color` as float-based tuple aliases.
In practice, Python users commonly pass integer literals for positions/sizes/colors in interactive scripts.
The previous runtime path required values to arrive as Python `float` objects, which surfaced avoidable errors such as:

- `runtime update error: failed to call update: ValueError: draw_texture position: expected float at index 0`

This created friction for valid numeric payloads while providing little safety benefit.

## Decision

Adopt runtime numeric coercion for Vec2/Color component parsing and typed key input:

- accept real-number inputs that Python can convert to float (`int` and `float`);
- coerce accepted numeric values to engine `f32`;
- expose a Python `KEY` enum and type `is_key_down` to consume `KEY` values;
- map mouse button keys (`MOUSE_LEFT`, `MOUSE_RIGHT`, `MOUSE_MIDDLE`) alongside keyboard keys;
- keep strict errors for non-numeric inputs and malformed tuple/list shapes.

Public API metadata and stubs remain unchanged (`Vec2`/`Color` continue to be represented as float tuple aliases).

## Consequences

- Usability improves for common gameplay scripts that use integer literals.
- Existing float-based scripts remain fully compatible.
- Error messages now target non-numeric payloads specifically while preserving context (`draw_texture position`, etc.).
- Runtime tests and an example scenario must cover numeric coercion behavior.
