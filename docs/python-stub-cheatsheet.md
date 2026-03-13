# pycro Python Stub Cheatsheet

Canonical source: `python/pycro/__init__.pyi`.
This document is a fast reference for the current generated stub API.

Lifecycle contract: the engine loads a configured script path (commonly `examples/*.py`), runs optional `setup()`, then required `update(dt)` each frame.

## Type Aliases

| Name | Definition | Description |
| --- | --- | --- |
| `Color` | `tuple[float, float, float, float]` | Normalized RGBA tuple. |
| `Vec2` | `tuple[float, float]` | Two-dimensional vector tuple. |
| `TextureHandle` | `str` | Opaque texture handle returned by the engine. |

## Exported API (`__all__`)

`["Color", "Vec2", "TextureHandle", "clear_background", "draw_circle", "is_key_down", "frame_time", "load_texture", "draw_texture", "set_camera_target", "draw_text", "submit_render"]`

## Functions

### Render

- `clear_background(color: Color) -> None`: Clear the current frame to a normalized RGBA color.
- `draw_circle(position: Vec2, radius: float, color: Color) -> None`: Draw a filled circle using world-space coordinates.
- `draw_text(text: str, position: Vec2, font_size: float, color: Color) -> None`: Draw text in screen space using a baseline anchor.
- `submit_render(commands: list[tuple[object, ...]]) -> None`: Queue multiple render commands in one Python-to-runtime call.

### Input + Timing

- `is_key_down(key: str) -> bool`: Return whether a named key is held on the current frame.
- `frame_time() -> float`: Return the last frame delta time in seconds.

### Textures / Assets

- `load_texture(path: str) -> TextureHandle`: Load a texture asset and return an opaque handle.
- `draw_texture(texture: TextureHandle, position: Vec2, size: Vec2) -> None`: Draw a texture at a world-space position and size.

### Camera

- `set_camera_target(target: Vec2) -> None`: Move the active camera target to the provided world position.

## Circle Rendering Behavior

Current runtime behavior for circle drawing:

- `draw_circle(position, radius, color)` does not accept an `options` argument.
- Circle-as-sprite rasterization optimization is disabled by default.
- It is only enabled when `PYCRO_CIRCLE_SPRITE=1`.

| Environment Variable | Purpose |
| --- | --- |
| `PYCRO_CIRCLE_SPRITE` | Enable sprite-based circle rasterization when set to `1`. |
| `PYCRO_CIRCLE_SPRITE_SIZE` | Optional sprite atlas/texture size tuning for circle rasterization. |
| `PYCRO_CIRCLE_SPRITE_FILTER` | Optional texture filter tuning for circle sprite sampling. |

Shell example:

```bash
PYCRO_CIRCLE_SPRITE=1 \
PYCRO_CIRCLE_SPRITE_SIZE=256 \
PYCRO_CIRCLE_SPRITE_FILTER=linear \
cargo run --bin pycro_cli -- --script examples/circle_demo.py
```

## Regenerate + Verify (Required)

Always regenerate first whenever `src/api.rs` metadata or Python API signatures change.

```bash
cargo run --bin generate_stubs -- --write python/pycro/__init__.pyi
```

Refresh this cheatsheet from the regenerated stub (`docs/python-stub-cheatsheet.md`) in the same commit.

Then run drift/type verification:

```bash
cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi
python3 -m mypy --config-file pyproject.toml
```

## Pre-Commit Phase Checklist

Phase A (refresh/recompile):

- Recompile generated stub file with `--write`.
- Refresh `docs/python-stub-cheatsheet.md` from the current stub API.

Phase B (verify):

- Confirm no stub drift via `--check`.
- Run typing smoke with `mypy`.
- Run full policy gates from `docs/validation-policy.md` before implementation commits.
