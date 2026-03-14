# Requirements

phase_objective: Add a first-class `pycro generate_stubs` CLI command that regenerates `python/pycro/__init__.pyi` using the canonical API metadata pipeline.

## Acceptance Criteria

- `pycro generate_stubs` is defined as a supported CLI command.
- The command regenerates `python/pycro/__init__.pyi` from canonical API metadata rather than from handwritten stub content.
- The command preserves existing direct generator correctness guarantees and deterministic output.
- Existing script execution and `init` behavior remain unchanged.
- The current `cargo run --bin generate_stubs -- ...` workflow can remain available as an internal/dev path.
- Tracker/state/phase docs are synchronized before implementation begins.

## Constraints

- Do not change public API metadata semantics in this phase.
- Do not expand the phase into broader project packaging or multi-target build work.
