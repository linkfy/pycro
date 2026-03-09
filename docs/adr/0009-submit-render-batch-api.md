# ADR 0009: Batched Render Submission API (`submit_render`)

## Status

Accepted

## Decision

Add a new public Python API function:

`submit_render(commands: list[tuple[object, ...]]) -> None`

The function batches multiple render commands into one Python-to-runtime call while preserving Python gameplay ownership. It is authored in `src/api.rs` metadata and exposed through:

- runtime direct bridge bindings and command parsing in `src/runtime.rs`
- generated stubs in `python/pycro/__init__.pyi`

Supported command names in the batch payload:

- `clear_background`
- `draw_circle`
- `draw_texture`
- `set_camera_target`
- `draw_text`

## Consequences

- Legacy `draw_*` APIs remain fully supported for compatibility.
- Runtime now supports a lower-crossing render path for Python scripts that opt into batch submission.
- Validation must include parity tests showing `submit_render` preserves draw order and payload equivalence with legacy `draw_*` calls.
