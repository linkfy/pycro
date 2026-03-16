# Closeout

status: closed
closeout_date: 2026-03-16

## Outcome

Phase 13 objective is complete:

- `pycro generate_stubs` is now a first-class CLI command in the main binary;
- the command reuses canonical API metadata rendering (`render_stub(module_spec())`);
- default command behavior regenerates project-local `pycro.pyi`;
- `--check` mode verifies drift from the same command surface;
- existing `pycro` run mode and `pycro init <project_name>` behavior remain unchanged.

## Validation Evidence

- `python3 scripts/validate_governance.py`
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi`
- `python3 -m mypy --config-file pyproject.toml`
- `cargo doc --no-deps`

## QA Outcome

- `qa-reviewer`: pass (parser regression preserved for run/init; generate_stubs write/check paths covered).

## Follow-up

- Phase 14 can now introduce the `pycro project` namespace without coupling it to stub regeneration concerns.
