# Architecture Plan

## Runtime Ownership

- `pycro_cli` is the only crate for now.
- `backend` module owns the Macroquad-side contract: frame loop ownership and render/input/time/assets/camera interface.
- `runtime` module owns the RustPython-side contract: VM lifecycle, `main.py` loading, `update(dt)` dispatch, and exception formatting.
- `api` module owns the Python public surface: function metadata, module registration plan, and deterministic stub rendering.
- `main` in `pycro_cli` is single-command entrypoint and delegates execution to `runtime`.

## Lifecycle Contract

- Entry script: `main.py`
- Required frame hook: `update(dt: float) -> None`

Load order:

1. Resolve and load `main.py`.
2. Add the entry-script directory to RustPython import search path so sidecar modules can be imported (for example `main.py` importing `phase03_player.py`).
3. Register the `pycro` module from the `api` registry.
4. Install runtime stdlib compatibility modules required by phase objectives (`math`, `os`) and preload sidecar modules from the entry-script directory, preserving sidecar precedence on name collisions.
5. Execute module top-level code.
6. Call `update(dt)` once per frame.

## Public API Source Of Truth

The registry in `pycro_cli::api` is the only authored definition of the Python-facing API. That metadata is used to:

- build the runtime registration plan,
- render `python/pycro/__init__.pyi`,
- validate doc completeness,
- enforce platform support declarations.

## Initial Public API Families

- `render`
- `input`
- `timing`
- `textures/assets`
- `camera`

Each public function must include:

- Python signature
- short docstring summary
- platform support matrix
- stub entry generated from registry metadata

## Planned Validation Layers

- Governance validation: AGENTS references and tracker/state consistency
- Rust unit tests: lifecycle, registration metadata, stub completeness
- Stub drift validation: generator `--check` over `python/pycro/__init__.pyi`
- Typing smoke: example scripts resolved against the stub package
- Integration smoke: lifecycle sequencing and Macroquad bridge calls

## Future Improvement Notes (Graphics Backend Policy)

- Current backend implementation is Macroquad-owned; backend API choice (for example OpenGL/Metal on Apple platforms) should become an explicit runtime policy surface instead of ad-hoc code edits.
- Proposed future contract:
  - platform-aware default backend choice,
  - explicit override input (environment/config),
  - startup report of selected backend for reproducible issue diagnosis.
- Any adoption of backend-selection controls should include:
  - docs update in platform matrix,
  - scenario-based comparison evidence,
  - ADR documenting tradeoffs (artifact behavior, pacing, compatibility).

## First Execution Objective

The first implementation objective after bootstrap is not a generic engine slice. It is a concrete vertical path:

1. `pycro_cli` runs `examples/phase01_basic_main.py`
2. `runtime` loads that script through RustPython
3. `backend` owns the live Macroquad desktop loop
4. `api` maps the thin public Python surface into the backend contract

This objective is only complete when `update(dt)` runs every frame and the example uses only the stubbed public `pycro` surface.
