# Architecture Plan

## Runtime Ownership

- `pycro_cli` is the only crate for now.
- `backend` module owns the Macroquad-side contract: frame loop ownership and render/input/time/assets/camera interface.
- `runtime` module owns the RustPython-side contract: VM lifecycle, `main.py` loading, `setup()` dispatch, `update(dt)` dispatch, and exception formatting.
- `api` module owns the Python public surface: function metadata, module registration plan, and deterministic stub rendering.
- `main` in `pycro_cli` is single-command entrypoint and delegates execution to `runtime`.

## Lifecycle Contract

- Entry script: `main.py`
- Optional setup hook: `setup() -> None`
- Required frame hook: `update(dt: float) -> None`

Load order:

1. Resolve and load `main.py`.
2. Register the `pycro` module from the `api` registry.
3. Execute module top-level code.
4. Call `setup()` if present.
5. Call `update(dt)` once per frame.

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

## First Execution Objective

The first implementation objective after bootstrap is not a generic engine slice. It is a concrete vertical path:

1. `pycro_cli` runs `examples/basic_main.py`
2. `runtime` loads that script through RustPython
3. `backend` owns the live Macroquad desktop loop
4. `api` maps the thin public Python surface into the backend contract

This objective is only complete when `setup()` runs once, `update(dt)` runs every frame, and the example uses only the stubbed public `pycro` surface.
