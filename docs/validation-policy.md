# Validation Policy

Implementation work is not commit-ready until all applicable checks pass and evidence is recorded in the tracker.

## Mandatory Gates

- `python3 scripts/validate_governance.py`
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi`
- `python3 -m mypy --config-file pyproject.toml`
- reviewer findings resolved or explicitly waived

## Additional Expectations

- Lifecycle changes require runtime tests.
- Public API changes require stub drift checks and typing smoke.
- Platform guarantee changes require a platform matrix update and an ADR.
- User-visible interactive features require a playable `examples/` scenario per feature and explicit user feedback recorded in tracker evidence.
