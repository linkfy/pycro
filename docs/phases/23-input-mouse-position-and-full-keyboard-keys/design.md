# Design

## Architecture Touchpoints

- `api` module: add metadata entry for `get_mouse_position()` and ensure `KEY` enum export remains canonical.
- `runtime` module: parse and dispatch mouse-position command return path; preserve direct-bridge error surfaces.
- `backend` module: provide mouse-position query and complete key-name to backend key-code mapping coverage.

## Proposed API Surface

1. `pycro.get_mouse_position() -> Vec2`
2. `pycro.KEY.<...>` includes full keyboard set for supported desktop target contract.

## Implementation Approach

1. Extend API metadata first so runtime dispatch + stub generation remain source-of-truth aligned.
2. Add backend primitives for mouse position and normalized key alias coverage.
3. Wire runtime direct-bridge return parsing and enum/key argument parsing to the expanded backend contract.
4. Add regression tests before finishing tracker/state synchronization.

## Runtime/Backend Contract

1. Backend exposes mouse position as a deterministic pair of floats.
2. Runtime converts backend mouse position to Python return value format consistent with existing `Vec2` pathways.
3. Runtime key parsing stays case/alias stable and rejects unknown names explicitly.

## Testing Strategy

1. Add unit tests for full key alias table coverage (positive + unknown-key rejection).
2. Add runtime direct-bridge regression test for mouse-position return shape/type.
3. Run stub drift check to validate metadata/stub synchronization.

## Risks

1. Key alias expansions may diverge across backends/platforms.
2. Missing alias normalization can create silent runtime mismatches.
3. Enum growth can regress docs/stub readability if not structured consistently.

## Mitigations

1. Keep one canonical alias table and reuse it for parsing + enum generation.
2. Preserve explicit invalid-key errors and test them.
3. Gate changes with stub drift checks and runtime tests.
