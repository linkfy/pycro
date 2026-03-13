# pycro

`pycro` is a docs-first game engine workspace that fixes the runtime split early:

- Macroquad owns the frame loop, rendering, input, assets, timing, and camera-facing platform layer.
- RustPython owns Python script loading and lifecycle dispatch.
- The public Python API is defined once in Rust metadata and projected into both runtime registration plans and `python/pycro/__init__.pyi`.

This repository is intentionally at the infrastructure milestone. It establishes governance, agent contracts, validation gates, and module boundaries before implementation expands into a playable engine.

Start with [AGENTS.md](./AGENTS.md) and the canonical docs in `docs/`.

## Mini Quickstart

Run a baseline scenario:

```bash
cargo run -- examples/phase01_basic_main.py
```

Fast smoke run (3 frames):

```bash
PYCRO_FRAMES=3 cargo run -- examples/phase01_basic_main.py
```

Python API quick reference:

- [`docs/python-stub-cheatsheet.md`](./docs/python-stub-cheatsheet.md)

Initialize a new pycro project scaffold:

```bash
cargo run --bin pycro -- init my_game
```

This creates `my_game/main.py` and `my_game/pycro.pyi`.

## Run Playable Examples

Use:

```bash
cargo run -- examples/<scenario>.py
```

See [`examples/README.md`](./examples/README.md) for scenario list and manual test checklist.
