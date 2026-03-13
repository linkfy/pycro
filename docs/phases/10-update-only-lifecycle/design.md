# Design

## Implementation Approach

- Remove `SETUP_FUNCTION` from runtime lifecycle dispatch path and associated public constants if no longer needed.
- Keep script module load and top-level execution unchanged.
- Update runtime tests that currently assert setup behavior.
- Refresh docs that currently describe `setup()` as part of engine lifecycle.
- Validate example compatibility: scripts may keep their own init helper functions but engine only guarantees `update(dt)` dispatch.

## ADR And Contract Alignment

- This phase changes lifecycle contract wording; update ADR/docs if contract boundary statements require formal amendment.
