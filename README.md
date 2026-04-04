# Pycro

**Pycro - Educational tool to teach python using graphics and videogame capabilities.**

`pycro` is an experimental mini engine focused on learning-by-building: write Python, render graphics, read input, and prototype game-like behavior while keeping a clear engine/runtime architecture in Rust.

This repository is docs-first and governance-heavy by design, so engine behavior, validation gates, and platform contracts stay explicit as features grow.

Start with [AGENTS.md](./AGENTS.md) and canonical docs under `docs/`.

## Setup

Pycro requires [Rust](https://rustup.rs/) (1.91+). All engine commands are run via `cargo` from the repo root. No manual binary setup is needed.

> **Contributors:** use `cargo run --bin pycro --` for all engine commands so your changes are always reflected without a separate build step.

---

## Mini Quickstart

Create a new project:

```bash
cargo run --bin pycro -- init my_game
cd my_game
```

`init` scaffolds `main.py`, a type stub (`pycro.pyi`), and copies the engine binary into the folder as `./pycro`. From here, `./pycro` is your runner — no `cargo` needed inside the project.

Run your game:

```bash
./pycro
```

Your `main.py` starts empty. Paste this in as your first script — it draws a circle you can move with the arrow keys:

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

Save the file — Pycro will hot-reload it instantly, no restart needed.

Now that your first script is running, try these next steps one at a time. Replace the contents of `main.py` with each example and save to see it live.

**Load and display a sprite** — swap your drawn shape for a real image file.

First, create an `assets/` folder inside your project and copy any `.png` into it:

```bash
mkdir assets
cp ../examples/assets/kenney_platformer_art_deluxe/Base\ pack/Player/p1_front.png assets/
```

Then update `main.py`:

```python
import pycro

tex = None

def update(dt: float) -> None:
    global tex
    if tex is None:
        tex = pycro.load_texture("assets/p1_front.png")

    pycro.clear_background((0.05, 0.05, 0.07, 1.0))
    pycro.draw_texture(tex, (100.0, 100.0), (64.0, 64.0))
    pycro.draw_text("Sprite from file!", (24.0, 56.0), 28.0, (0.9, 0.95, 1.0, 1.0))
```

> Always keep assets inside your project's `assets/` folder. Paths like `../` work when
> running `./pycro` directly, but the project build (`cargo run --bin pycro -- project build`)
> only embeds files inside the project directory — so `../` assets will be missing from the
> built binary.

> `load_texture` is safe to call every frame — it caches the image after the first load so there is no repeated disk read.

Build and run a distributable desktop artifact:

```bash
cargo run --bin pycro -- project build . --target desktop --exe game
./dist/desktop/game
```

API quick reference (copy-ready examples, signatures, and patterns):

- [`docs/python-stub-cheatsheet.md`](./docs/python-stub-cheatsheet.md)

---

## How the core runtime works

- Macroquad owns frame loop, rendering, input, assets, timing, and camera-facing platform behavior.
- RustPython owns Python script loading and lifecycle dispatch (`update(dt)`).
- The public Python API is defined once in Rust metadata and projected into runtime registration plus `python/pycro/__init__.pyi`.

---

## CLI Commands

All commands below are run from the **repo root** using `cargo`.

> **Inside a project folder** (created via `init`), use `./pycro` instead — the engine binary is already there.

Create a new project scaffold:

```bash
cargo run --bin pycro -- init my_game
```

Regenerate Python stubs from canonical Rust API metadata:

```bash
cargo run --bin pycro -- generate_stubs
```

By default this writes `pycro.pyi` in the current directory (next to `main.py`).

Check stub drift without writing changes:

```bash
cargo run --bin pycro -- generate_stubs --check pycro.pyi
```

Important: the command is `generate_stubs` (underscore), not `generate-stubs`.

Project build (desktop):

```bash
cargo run --bin pycro -- project build . --target desktop
```

Custom artifact name:

```bash
cargo run --bin pycro -- project build . --target desktop --exe game
```

Desktop build output:

- default artifact path: `./dist/desktop/game` (or `game.exe` on Windows)
- packaging model: embedded project payload (`main.py`, root sidecar `.py`, optional `assets/**`, optional `pycro-project.toml`)

Run the built artifact:

```bash
./dist/desktop/game
```

Behavior contract:

- `./pycro` inside a project folder runs `main.py` from that directory (no embedded payload).
- `./dist/desktop/game` / `game.exe` (project build output) runs the embedded payload and does not require a local `main.py`.

Short alias (desktop default target):

```bash
cargo run --bin pycro -- build .
```

Explicit target:

```bash
cargo run --bin pycro -- build . --target desktop
```

Custom artifact name via alias:

```bash
cargo run --bin pycro -- build . --target desktop --exe game
```

---

## Run Playable Examples

Run from the repo root:

```bash
cargo run -- examples/<scenario>.py
```

Fast smoke run (3 frames, useful for CI or quick checks):

```bash
PYCRO_FRAMES=3 cargo run -- examples/phase01_basic_main.py
```

See [`examples/README.md`](./examples/README.md) for the full scenario list and manual test checklist.
