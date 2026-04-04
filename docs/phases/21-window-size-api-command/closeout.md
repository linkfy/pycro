# Closeout

status: complete
closed_on: 2026-03-27
owner: architecture-orchestrator
qa_outcome: pass_with_waiver

## Delivered Scope

- Added `pycro.get_window_size() -> Vec2` to the canonical API metadata, runtime direct bridge, and generated stubs.
- Added `pycro.draw_rectangle(x, y, width, height, color)` to metadata, runtime draw queue/flush path, and backend dispatch.
- Extended backend contract with `get_window_size` and `draw_rectangle` implementations.
- Added phase scenario `examples/phase21_window_size_rectangle_lab.py` and updated docs references.

## Validation Evidence

- `python3 scripts/validate_governance.py` -> pass
- `cargo fmt --all --check` -> pass
- `cargo clippy --all-targets -- -D warnings` -> pass
- `cargo test` -> pass
- `cargo run --bin generate_stubs -- --write python/pycro/__init__.pyi` -> pass
- `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi` -> pass
- `python3 -m mypy --config-file pyproject.toml` -> pass
- `cargo doc --no-deps` -> pass

## QA Waiver

- Manual runtime smoke for `examples/phase21_window_size_rectangle_lab.py` is blocked on this machine by upstream miniquad Apple panic (`patches/miniquad-0.4.8-windows-rawinput-fix/src/native/apple/apple_util.rs` null pointer dereference).
- Waiver scope: manual interactive scenario confirmation only.
- Remaining implementation and automated gates are green.

## Next Phase

- Phase 22 (`22-texture-load-cache`) remains planned for cache-hit reuse in repeated `load_texture(path)` calls.
