# Closeout

status: closed
closeout_date: 2026-03-18

## Outcome

Phase 17 objective is complete:

- `pycro project build --target web` now produces real web artifacts under `dist/web/`:
  - `pycro.wasm`
  - `gl.js`
  - `index.html`
- web startup runs against embedded payload packaging (entry script + sidecars + assets) instead of loose project Python files beside `dist/web`.
- compatibility shims in vendored `gl.js` handle missing wasm import modules/functions (`__wbindgen_placeholder__` class of imports) in the current Macroquad/Miniquad web loader model.
- wasm-side texture loading now resolves `assets/...` from embedded payload bytes, eliminating white fallback quads when assets are present in the project.
- non-web regression guard remains passing (desktop build + embedded runtime smoke preserved).

## Validation Evidence

- `python3 scripts/validate_governance.py`
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi`
- `python3 -m mypy --config-file pyproject.toml`
- `cargo doc --no-deps`
- `python3 scripts/phase17_web_embedded_smoke.py` (PASS)
- `python3 scripts/phase15_desktop_embedded_smoke.py` (PASS)

## QA Outcome

- `qa-reviewer`: pass (web build contract, embedded payload startup model, wasm compatibility shim behavior, and desktop non-regression evidence validated).

## Scope Note

- This phase remains an implementation-oriented WASM POC baseline for local/staging web delivery.
- Production-grade wasm-bindgen ecosystem interoperability remains future hardening scope and does not block this phase closeout.

## Follow-up

- next queued target: phase 18 (`project-android-build`) on top of the same embedded payload architecture.
