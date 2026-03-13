# Design

## Implementation Approach

- Add CLI command parser with explicit modes:
  - `init <project_name>`
  - default script execution mode (current behavior)
- Implement project scaffold writer in `src/main.rs` to avoid introducing extra crate complexity for this phase.
- Write deterministic template files for `main.py` and `pycro.pyi`.
- Use compile-time inclusion for canonical stub content (`include_str!`) so generated scaffold always matches repo version at build time.

## ADR And Contract Alignment

- This phase changes user CLI behavior; if command lifecycle or public bootstrap contract expands further, add/update ADR.
- Keep this design aligned with requirements and task board.
