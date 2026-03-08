# Project Vision

## Outcome

`pycro` is a Rust workspace for a Python-scriptable game engine with a fixed architectural split:

- Macroquad owns the frame loop and all platform-facing render/input/time/assets/camera behavior.
- RustPython owns Python script execution.
- Game code imports `pycro`, the engine loads `main.py`, runs optional `setup()`, and runs `update(dt)` every frame.

## Milestone Focus

This repository is in the infrastructure milestone. The deliverable is a stable repo skeleton with governance, branch discipline, technical contracts, and validation hooks that allow implementation to start safely.

## Product Constraints

- Documentation language is English only.
- Public Python APIs are defined once and projected into both runtime registration and `.pyi` stubs.
- The initial Python API stays intentionally thin and cross-platform-safe: render, input, timing, textures/assets, and camera.
- Platform parity is a tracked product requirement, not a best-effort aspiration.

## Readiness Standard

Work is ready to leave this milestone only when:

- the docs tracker and machine state remain in sync,
- the workspace compiles and tests,
- the stub generator/check path is deterministic,
- example scripts type-check against the generated stubs,
- branch/review/ADR policies are encoded in docs and CI.

