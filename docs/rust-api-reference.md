# Rust API Reference (`pycro_cli`)

This document is the Rust-facing reference for the current single-crate architecture in `pycro_cli`.

## Scope and Module Map

Source files:

- `src/lib.rs`: crate surface and re-exports.
- `src/main.rs`: CLI entrypoint and desktop loop wiring.
- `src/runtime.rs`: RustPython lifecycle orchestration and direct API bridge.
- `src/api.rs`: canonical Python metadata, registration plan, and stub rendering.
- `src/backend.rs`: backend contract and Macroquad implementation.

High-level ownership:

- `runtime`: owns script lifecycle (`load_main`, `update(dt)`) and VM adapter boundary.
- `api`: owns canonical Python API metadata and deterministic `.pyi` generation.
- `backend`: owns frame-loop boundary and platform-facing calls (render/input/timing/assets/camera).

## Runtime Module (`src/runtime.rs`)

Key contract types:

- `RuntimeConfig { entry_script: String }`: script path config for runtime load.
- `RuntimeError`: lifecycle/dispatch error surface (`NotLoaded`, `MissingUpdateFunction`, `ScriptLoad`, `FunctionCall`).
- `RuntimeValue`: runtime argument shape currently supports `Float(f32)` for `update(dt)`.
- `ModuleInstallPlan`: runtime install plan derived from API metadata.
- `PythonVm` trait: abstraction for VM lifecycle primitives:
  - `install_module`
  - `load_script`
  - `has_function`
  - `call_function`
- `ScriptRuntime<Vm: PythonVm>`: lifecycle coordinator.
- `RustPythonVm`: production VM adapter (`rustpython_vm::Interpreter`) with direct backend bindings.

Current lifecycle behavior (`ScriptRuntime`):

1. Build module install plan from `api::registration_plan()`.
2. Install `pycro` module into RustPython.
3. Load configured script source.
4. Require `update(dt)`; fail with `RuntimeError::MissingUpdateFunction` if missing.
5. On each frame, call `update(dt)`.

Color contract note:

- `Color` is a normalized RGBA tuple contract (`0..1` per channel), not byte-based `0..255`.
- Examples:
  - black: `(0, 0, 0, 1)`
  - white: `(1, 1, 1, 1)`

Phase-4 import compatibility behavior (`RustPythonVm::load_script`):

- Adds entry-script directory to `sys.path`.
- Installs runtime stdlib compatibility modules for `math` and `os`.
- Preloads sidecar modules from the entry-script directory so local files still win on module-name collisions.

Direct bridge status (current architecture):

- Runtime installs callable Python functions that directly invoke `MacroquadBackendContract` through a shared mutex (`Arc<Mutex<...>>`).
- Return values propagate directly for `is_key_down -> bool`, `frame_time -> float`, `load_texture -> TextureHandle(str)`.
- Python argument shape/type errors are surfaced as `RuntimeError::FunctionCall` with RustPython exception details.

## API Module (`src/api.rs`)

Canonical metadata constants:

- `MODULE_NAME = "pycro"`
- `ENTRYPOINT_SCRIPT = "main.py"`
- `UPDATE_FUNCTION = "update"`

Key API metadata types:

- `ApiFamily`: `Render | Input | Timing | Assets | Camera`.
- `PlatformSupportLevel`: `Supported | Planned` (currently rendered as labels).
- `PlatformMatrix`: per-target support declaration.
- `PythonAlias`, `PythonArg`, `PythonFunction`, `ModuleSpec`.
- `RegistrationFunction`: runtime registration entry derived from metadata.

Public authored API functions (from `FUNCTIONS`):

- `clear_background(color: Color) -> None`
- `draw_circle(position: Vec2, radius: float, color: Color) -> None`
- `is_key_down(key: KEY) -> bool`
- `frame_time() -> float`
- `load_texture(path: str) -> TextureHandle`
- `draw_texture(texture: TextureHandle, position: Vec2, size: Vec2) -> None`
- `set_camera_target(target: Vec2) -> None`
- `draw_text(text: str, position: Vec2, font_size: float, color: Color) -> None`
- `submit_render(commands: list[tuple[object, ...]]) -> None`

Deterministic outputs:

- `module_spec()`: canonical module schema.
- `registration_plan()`: runtime install plan derived from schema.
- `render_stub(spec)`: deterministic `python/pycro/__init__.pyi` content.

Note: `parse_backend_dispatch_record` and `dispatch_backend_record` still exist for typed record parsing/dispatch utility and tests; runtime API execution path is direct-call bridge.

Argument binding note:

- `is_key_down` accepts `key` as positional or keyword argument (`is_key_down(KEY.ESCAPE)` and `is_key_down(key=KEY.ESCAPE)`).

## Backend Module (`src/backend.rs`)

Core contract types:

- `Vec2`, `Color`, `TextureHandle`.
- `BackendDispatch`: typed log entries for backend calls.
- `EngineBackend` trait:
  - `clear_background`
  - `draw_circle`
  - `is_key_down`
  - `frame_time`
  - `load_texture`
  - `draw_texture`
  - `set_camera_target`
  - `draw_text`

Frame loop boundary:

- `FrameLoopConfig`:
  - `PYCRO_FRAME_DT` controls fixed dt.
  - `PYCRO_FRAMES` controls frame budget.
- `DesktopFrameLoop::run(...) -> Result<DesktopLoopReport, String>` drives Macroquad frame updates.
- `window_conf()` provides Macroquad window setup.

Concrete implementation:

- `MacroquadBackendContract` implements `EngineBackend`.
- Maintains `frame_time`, `dispatch_log`, and texture map.
- `load_texture` returns a handle even when file bytes are missing; draw falls back to marker rectangle if texture is unresolved.

## Main Entrypoint (`src/main.rs`)

Runtime boot sequence:

1. Resolve script path from CLI arg, default `main.py` in the working directory.
2. Build `RuntimeConfig`.
3. Print module/install summary (`module_spec` + `registration_plan`).
4. `ScriptRuntime::load_main()`.
5. Run desktop frame loop and call `runtime.update(dt)` every frame.
6. Print frame count + backend dispatch count.

## Practical Snippets

Run a playable script:

```bash
cargo run -- examples/phase01_basic_main.py
```

Run a deterministic short loop:

```bash
PYCRO_FRAMES=2 PYCRO_FRAME_DT=0.016 cargo run -- examples/phase01_basic_main.py
```

Regenerate Python stubs from Rust metadata:

```bash
cargo run --bin generate_stubs -- python/pycro/__init__.pyi
```

Check stub drift only:

```bash
cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi
```

Build local Rust API docs:

```bash
cargo doc --no-deps
```

## Validation Commands

Baseline validation policy commands (see `docs/validation-policy.md`):

```bash
python3 scripts/validate_governance.py
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi
python3 -m mypy --config-file pyproject.toml
```

## Phase Commit Checklist

Documentation agents must re-validate and recompile docs before each phase commit.

Required doc refresh checklist:

1. Rebuild Rust docs:
   ```bash
   cargo doc --no-deps
   ```
2. Re-check generated stubs against metadata:
   ```bash
   cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi
   ```
3. Re-run full validation gates for commit readiness:
   ```bash
   python3 scripts/validate_governance.py
   cargo fmt --all --check
   cargo clippy --all-targets -- -D warnings
   cargo test
   python3 -m mypy --config-file pyproject.toml
   ```

Record evidence in tracker/state summary fields before commit (`validation_evidence`, risks/follow-ups, and any waivers).

## Current Compilation Evidence

Latest run in this workspace:

```text
$ cargo doc --no-deps
Checking pycro_cli v0.1.2 (/Users/linkfy/Code/pycro)
Documenting pycro_cli v0.1.2 (/Users/linkfy/Code/pycro)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.39s
Generated /Users/linkfy/Code/pycro/target/doc/pycro_cli/index.html and 3 other files
```
