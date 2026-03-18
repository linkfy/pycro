# Phase 17-project-web-build - Project Web Build

status: closed
objective: Add `pycro project build --target web` on top of the shared embedded project payload and target architecture.
tracked_tasks: project-web-build

## Links

- `docs/task-tracker.txt`
- `state/repo-state.json`

## Current Scope (WASM POC)

- This phase is a preliminary WASM proof-of-concept for `pycro project build --target web`.
- Goal: stable local web build + smoke-level runtime behavior with embedded payload files (`main.py`, sidecars, `assets/`).
- Non-goal in this phase: production-grade wasm-bindgen integration or full parity with desktop/mobile target guarantees.

## Build And Run (Web)

1. Build:
   - `cargo run --bin pycro -- project build --project '<PROJECT_PATH>' --target web`
2. Serve:
   - `cd <PROJECT_PATH>`
   - `python3 -m http.server 8000`
3. Open:
   - `http://localhost:8000/dist/web/`

## Build And Run (Desktop / Non-Web)

1. Build:
   - `cargo run --bin pycro -- project build --project '<PROJECT_PATH>' --target desktop`
2. Run artifact:
   - `<PROJECT_PATH>/dist/desktop/game` (or custom `--exe` name)

## Troubleshooting (Known In This Phase)

- `No __wbindgen_placeholder__ ...` lines:
  - Expected in this POC flow. `gl.js` compatibility stubs patch missing wasm imports.
- `gl.js: applied compatibility stubs for missing WASM imports`:
  - Informational, expected.
- Texture quads render white in web:
  - Ensure texture path is inside project `assets/`.
  - Rebuild after asset/path changes.
  - Current web runtime resolves `load_texture("assets/...")` from embedded payload bytes.
- Black screen with no obvious crash:
  - First isolate with minimal script:
    - `clear_background(...)` + one `draw_circle(...)`.
  - Then re-enable project logic.
- Script pitfalls seen in real example:
  - Avoid `exit()` in web update loop.
  - Guard FPS division (`dt <= 0.0`).
  - Use valid format string (`f"{fps:.2f}"`, not `str(1/dt):.2`).
- `'apple-m1' is not a recognized processor for this target` during wasm build:
  - Comes from host-native `target-cpu=native` rustflags leaking into `wasm32`.
  - Restrict native rustflags to non-wasm targets in `.cargo/config.toml`.
