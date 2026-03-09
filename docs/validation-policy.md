# Validation Policy

Implementation work is not commit-ready until all applicable checks pass and evidence is recorded in the tracker.

## Mandatory Gates

- `python3 scripts/validate_governance.py`
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi`
- `python3 -m mypy --config-file pyproject.toml`
- `cargo doc --no-deps`
- reviewer findings resolved or explicitly waived

## Additional Expectations

- Lifecycle changes require runtime tests.
- Public API changes require stub drift checks and typing smoke.
- Platform guarantee changes require a platform matrix update and an ADR.
- User-visible interactive features require a playable `examples/` scenario per feature and explicit user feedback recorded in tracker evidence.

## Benchmark Integrity Rule (Python Gameplay)

- Performance improvements must preserve the Python gameplay execution model as the measured subject.
- Do not claim runtime FPS gains from moving user gameplay loops/entities/simulation logic from Python scripts into Rust internals.
- Optimizations are valid when they improve engine/runtime overhead while keeping equivalent gameplay logic authored and executed from Python.
- If any benchmark requires Rust-side automation of gameplay logic for diagnosis, record it as a separate diagnostic metric and not as the canonical pycro user-facing performance result.
- Benchmark evidence must include runtime stack metadata: Python version, pygame variant/version, and SDL version.
- For cross-runtime comparisons, run direct per-runtime commands first (without combined harness output) and keep matrix/harness runs as secondary confirmation.
- Canonical pycro performance claims must be taken from `cargo run --release` runs (not debug profile).

## Phase Pre-Commit Documentation Checklist (Mandatory)

- Refresh `docs/rust-api-reference.md` when runtime/api/backend contracts change.
- Recompile Rust docs with `cargo doc --no-deps` after the refresh.
- When Python API metadata/signatures change, regenerate stubs with `cargo run --bin generate_stubs -- --write python/pycro/__init__.pyi`.
- Refresh `docs/python-stub-cheatsheet.html` from the regenerated `python/pycro/__init__.pyi` surface in the same commit.
- Verify no drift after refresh with `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi`.
- Re-run typing smoke with `python3 -m mypy --config-file pyproject.toml` after stub/docs refresh.
- Record the refresh + recompile evidence in tracker/state before commit.
- Before each phase commit, documentation artifacts must be refreshed/recompiled (Rust docs and Python stub-facing docs) and evidence recorded.
