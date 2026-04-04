# Requirements

## Problem Statement

Current input coverage is incomplete for pointer position access and full keyboard key exposure. This prevents scripts from relying on a stable, fully documented key input contract.

## Goals

1. Add a Python-facing API to query current mouse position each frame.
2. Expand keyboard key support so `KEY` enum coverage is complete for the intended desktop target contract.
3. Keep direct-bridge semantics deterministic and typed for runtime parsing and stub generation.

## Functional Requirements

1. `pycro.get_mouse_position()` must return a `Vec2`-compatible value (`x`, `y`) with numeric components.
2. `is_key_down(...)` must accept all exposed key aliases documented in the generated `KEY` enum.
3. API metadata and generated `python/pycro/__init__.pyi` must stay synchronized.
4. Unknown or unsupported key names must continue to fail deterministically with explicit errors.

## Non-Goals

1. Touch/multi-touch gestures are out of scope.
2. Mouse button API redesign is out of scope.
3. Controller/gamepad mappings are out of scope.

## Validation Requirements

- `python3 scripts/validate_governance.py`
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo run --bin generate_stubs -- --check python/pycro/__init__.pyi`

## Acceptance Criteria

1. Runtime tests cover new mouse-position bridge behavior and expanded key parsing.
2. Stub generation includes new API/enum coverage without drift.
3. Tracker/state/phase docs are synchronized for phase 23 kickoff.
