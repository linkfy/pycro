# Pycro

**Pycro - Educational tool to teach python using graphics and videogame capabilities.**

`pycro` is an experimental mini engine focused on learning-by-building: write Python, render graphics, read input, and prototype game-like behavior while keeping a clear engine/runtime architecture in Rust.

This repository is docs-first and governance-heavy by design, so engine behavior, validation gates, and platform contracts stay explicit as features grow.

Start with [AGENTS.md](./AGENTS.md) and canonical docs under `docs/`.

## Mini Quickstart

Starter script example (`my_game/main.py`):

```python
import pycro

BG_COLOR = (0.07, 0.07, 0.1, 1.0)
position_x = 48.0

def update(dt: float) -> None:
    global position_x
    if pycro.is_key_down("Right"):
        position_x += 140.0 * dt
    if pycro.is_key_down("Left"):
        position_x -= 140.0 * dt

    pycro.clear_background(BG_COLOR)
    pycro.draw_circle((position_x, 180.0), 22.0, (0.3, 0.9, 1.0, 1.0))
    pycro.draw_text("Move with Left/Right", (24.0, 48.0), 28.0, (0.95, 0.95, 0.98, 1.0))
```

Texture + fallback style example:

```python
import pycro

tex = None

def update(dt: float) -> None:
    global tex
    if tex is None:
        tex = pycro.load_texture("examples/assets/pattern.png")

    pycro.clear_background((0.05, 0.05, 0.07, 1.0))
    pycro.draw_texture(tex, (24.0, 90.0), (128.0, 128.0))
    pycro.draw_text("Texture sample", (24.0, 56.0), 28.0, (0.9, 0.95, 1.0, 1.0))
```

Initialize a new project first:

```bash
./pycro init my_game
```

Then enter the project folder:

```bash
cd my_game
```

Run it:

```bash
./pycro
```

Run a baseline scenario:

```bash
cargo run -- examples/phase01_basic_main.py
```

Fast smoke run (3 frames):

```bash
PYCRO_FRAMES=3 cargo run -- examples/phase01_basic_main.py
```

Build desktop artifact (`game`) with embedded payload:

```bash
./pycro project build . --target desktop --exe game
./dist/desktop/game
```

API quick reference (copy-ready examples, signatures, and patterns):

- [`docs/python-stub-cheatsheet.md`](./docs/python-stub-cheatsheet.md)

## How the core runtime works

- Macroquad owns frame loop, rendering, input, assets, timing, and camera-facing platform behavior.
- RustPython owns Python script loading and lifecycle dispatch (`update(dt)`).
- The public Python API is defined once in Rust metadata and projected into runtime registration plus `python/pycro/__init__.pyi`.

## CLI Commands

Create a new project scaffold:

```bash
./pycro init my_game
```

Equivalent from Cargo:

```bash
cargo run --bin pycro -- init my_game
```

Regenerate Python stubs from canonical Rust API metadata:

```bash
./pycro generate_stubs
```

By default this writes `pycro.pyi` in the current project directory (next to `main.py`).

Check stub drift without writing changes:

```bash
./pycro generate_stubs --check pycro.pyi
```

Important: the command is `generate_stubs` (underscore), not `generate-stubs`.

Project build (desktop, phase 15):

```bash
./pycro project build . --target desktop
```

Custom artifact name:

```bash
./pycro project build . --target desktop --exe game
```

Desktop build output:

- default artifact path: `./dist/desktop/game` (or `game.exe` on Windows)
- packaging model: embedded project payload (`main.py`, root sidecar `.py`, optional `assets/**`, optional `pycro-project.toml`)

Run the built artifact:

```bash
./dist/desktop/game
```

Behavior contract:

- `./pycro` (CLI binary) runs `main.py` from the current directory when no embedded payload is present.
- `./dist/desktop/game` / `game.exe` (project build output) runs the embedded payload and does not require local `main.py`.

Short alias (desktop default target):

```bash
./pycro build .
```

Explicit target via alias:

```bash
./pycro build . --target desktop
```

Alias with custom artifact name:

```bash
./pycro build . --target desktop --exe game
```

## Run Playable Examples

Use:

```bash
cargo run -- examples/<scenario>.py
```

See [`examples/README.md`](./examples/README.md) for scenario list and manual test checklist.
