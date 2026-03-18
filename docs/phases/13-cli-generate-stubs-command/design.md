# Design

## Implementation Approach

- Extend the main `pycro` CLI with a dedicated `generate_stubs` command rather than requiring the standalone helper binary for normal usage.
- Reuse the existing canonical stub rendering path so output remains deterministic and aligned with API metadata.
- Keep the command narrowly scoped to stub generation to avoid coupling it with future `project` build work.

## ADR And Contract Alignment

- This phase changes the CLI surface but not the API metadata contract itself; ADR is only required if the stub generation contract changes materially.
