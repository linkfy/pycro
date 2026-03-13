# pycro Python Stub Cheatsheet

Canonical source: `python/pycro/__init__.pyi`.
This document is a fast reference for the current generated stub API.

Lifecycle contract: the engine loads a configured script path (commonly `examples/*.py`) and calls required `update(dt)` each frame.

## API At A Glance

### Type Aliases

| Name | Definition | Description |
| --- | --- | --- |
| `Color` | `tuple[float, float, float, float]` | Normalized RGBA tuple. |
| `Vec2` | `tuple[float, float]` | Two-dimensional vector tuple. |
| `TextureHandle` | `str` | Opaque texture handle returned by the engine. |

### Functions

| Function | Signature | Family | Summary |
| --- | --- | --- | --- |
| `clear_background` | `(color: Color) -> None` | render | Clear current frame with a normalized RGBA color. |
| `draw_circle` | `(position: Vec2, radius: float, color: Color) -> None` | render | Draw a filled world-space circle. |
| `draw_text` | `(text: str, position: Vec2, font_size: float, color: Color) -> None` | render | Draw screen-space text using baseline anchor. |
| `submit_render` | `(commands: list[tuple[object, ...]]) -> None` | render batch | Queue many render commands in one bridge call. |
| `submit_circle_batch` | `(positions: list[Vec2], radii: list[float], colors: list[Color]) -> None` | circle batch | Queue many circles in one specialized batch call. |
| `is_key_down` | `(key: str) -> bool` | input | Return whether a key is currently held. |
| `frame_time` | `() -> float` | timing | Return last frame delta time in seconds. |
| `load_texture` | `(path: str) -> TextureHandle` | textures/assets | Load texture and return opaque handle. |
| `draw_texture` | `(texture: TextureHandle, position: Vec2, size: Vec2) -> None` | textures/assets | Draw texture at world-space position and size. |
| `set_camera_target` | `(target: Vec2) -> None` | camera | Move active camera target to world-space coordinates. |

## Quickstart

Run an existing scenario:

```bash
cargo run -- examples/phase01_basic_main.py
```

Run only a few frames (fast smoke):

```bash
PYCRO_FRAMES=3 cargo run -- examples/phase01_basic_main.py
```

Minimal starter script:

```python
import pycro

x = 160.0

def update(dt: float) -> None:
    global x
    x += 120.0 * dt
    if x > 1200.0:
        x = 80.0
    pycro.clear_background((0.08, 0.10, 0.14, 1.0))
    pycro.draw_circle((x, 360.0), 24.0, (0.20, 0.80, 1.0, 1.0))
    pycro.draw_text("pycro quickstart", (24.0, 44.0), 28.0, (0.95, 0.97, 1.0, 1.0))
```

Key input + timing starter:

```python
import pycro

pos = [640.0, 360.0]

def update(dt: float) -> None:
    speed = 260.0 if pycro.is_key_down("Space") else 150.0
    if pycro.is_key_down("Left"):
        pos[0] -= speed * dt
    if pycro.is_key_down("Right"):
        pos[0] += speed * dt
    if pycro.is_key_down("Up"):
        pos[1] -= speed * dt
    if pycro.is_key_down("Down"):
        pos[1] += speed * dt

    pycro.clear_background((0.05, 0.06, 0.10, 1.0))
    pycro.draw_circle((pos[0], pos[1]), 20.0, (0.45, 1.0, 0.55, 1.0))
    pycro.draw_text(f"dt={pycro.frame_time():.4f}", (20.0, 30.0), 24.0, (0.9, 0.95, 1.0, 1.0))
```

Texture starter:

```python
import pycro

texture = None

def update(dt: float) -> None:
    global texture
    if texture is None:
        texture = pycro.load_texture("examples/assets/kenney_development_essentials/Gradient/gradient-radial.png")
    pycro.clear_background((0.04, 0.04, 0.06, 1.0))
    if texture is not None:
        pycro.draw_texture(texture, (420.0, 220.0), (440.0, 260.0))
```

## Type Aliases

| Name | Definition | Description |
| --- | --- | --- |
| `Color` | `tuple[float, float, float, float]` | Normalized RGBA tuple. |
| `Vec2` | `tuple[float, float]` | Two-dimensional vector tuple. |
| `TextureHandle` | `str` | Opaque texture handle returned by the engine. |

## Exported API (`__all__`)

`["Color", "Vec2", "TextureHandle", "clear_background", "draw_circle", "is_key_down", "frame_time", "load_texture", "draw_texture", "set_camera_target", "draw_text", "submit_render", "submit_circle_batch"]`

## Functions

### Render

- `clear_background(color: Color) -> None`: Clear the current frame to a normalized RGBA color.
- `draw_circle(position: Vec2, radius: float, color: Color) -> None`: Draw a filled circle using world-space coordinates.
- `draw_text(text: str, position: Vec2, font_size: float, color: Color) -> None`: Draw text in screen space using a baseline anchor.
- `submit_render(commands: list[tuple[object, ...]]) -> None`: Queue multiple render commands in one Python-to-runtime call.
- `submit_circle_batch(positions: list[Vec2], radii: list[float], colors: list[Color]) -> None`: Queue many circles in one specialized batch call.

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
