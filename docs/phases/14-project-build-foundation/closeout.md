# Closeout

status: closed
closeout_date: 2026-03-16

## Outcome

Phase 14 objective is complete:

- `pycro project build` is now a first-class CLI namespace command;
- project path is supported in both forms: positional (`pycro project build . --target desktop`) and explicit flag (`--project <path>`);
- short alias `pycro build` is supported and defaults to `desktop` when target is omitted;
- external project contract is validated before build orchestration (`main.py` required, local `.py` modules discovered, optional `assets/`, reserved `pycro-project.toml`);
- shared `ProjectBundle` and resource provider plan are defined as canonical inputs for phases 15-18;
- existing behavior for `pycro`, `pycro <script>`, `pycro init`, and `pycro generate_stubs` remains preserved.

## Validation Evidence

- `python3 scripts/validate_governance.py`
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi`
- `python3 -m mypy --config-file pyproject.toml`
- `cargo doc --no-deps`

## QA Outcome

- `qa-reviewer`: pass (CLI parser and compatibility paths validated; project contract and bundle tests included).

## Follow-up

- Phase 15 implements real desktop packaging on top of the phase-14 bundle contract.
