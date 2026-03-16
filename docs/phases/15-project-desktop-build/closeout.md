# Closeout

status: closed
closeout_date: 2026-03-17

## Outcome

Phase 15 objective is complete:

- `pycro project build --target desktop` now produces a desktop artifact with embedded project payload;
- desktop output defaults to `dist/desktop/game` (`game.exe` on Windows);
- optional executable override is supported via `--exe <name>` for desktop target builds;
- payload build pipeline is implemented with build-time generation (`build.rs`) and compile-time embedding;
- runtime startup supports embedded payload execution by default when present (temporary staging + preserved relative asset paths);
- user-visible phase scenario was added for manual validation: `examples/phase15_embedded_payload_lab.py` with sidecar module import support.

## Validation Evidence

- `python3 scripts/phase15_desktop_embedded_smoke.py` (PASS)
- `python3 scripts/validate_governance.py`
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi`
- `python3 -m mypy --config-file pyproject.toml`
- `cargo doc --no-deps`

## QA Outcome

- `qa-reviewer`: pass (desktop build contract, `--exe` naming path, embedded startup path, smoke + scenario evidence validated).

## Follow-up

- Phase 16 implements `pycro project build --target web` on top of the shared embedded payload architecture.
