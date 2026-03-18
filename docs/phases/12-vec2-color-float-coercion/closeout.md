# Closeout

status: closed
closeout_date: 2026-03-15

## Outcome

Phase 12 objective is complete:

- runtime Vec2/Color component parsing now coerces numeric Python values (`int`/`float`) to engine `f32`;
- draw calls no longer fail on valid int components with `expected float at index ...` errors;
- `KEY` enum was introduced in Python stubs/module bootstrap and `is_key_down` now consumes typed key values from that enum;
- `is_key_down` now supports explicit keyword binding (`is_key_down(key=KEY.ESCAPE)`) in addition to positional calls;
- stdlib encoding support was extended so codec lookups used by `dataclasses` import paths (e.g. `latin1`) are available;
- regression tests and a playable scenario were added for the new coercion behavior.

## Validation Evidence

- `python3 scripts/validate_governance.py`
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi`
- `python3 -m mypy --config-file pyproject.toml`
- `cargo doc --no-deps`
- manual scenario: `examples/phase12_vec2_color_numeric_coercion.py`
- targeted regression: `runtime::tests::load_main_supports_stdlib_encodings_latin1`
- targeted regression: `runtime::tests::is_key_down_accepts_key_keyword_argument`

## QA Outcome

- `qa-reviewer`: pass (numeric coercion path verified for Vec2/Color, input keyword binding verified, invalid-shape errors preserved).

## Follow-up

- Optional future hardening: support attribute-based vector/color objects (`.x/.y`, `.r/.g/.b/.a`) in addition to sequence payloads if needed.
