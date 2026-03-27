# Requirements

phase_objective: Expose current runtime window dimensions through a stable pycro API surface using `get_window_size()`.

## Acceptance Criteria

- New API entrypoint `pycro.get_window_size()` is available to Python scripts.
- Return contract is deterministic and documented (tuple/vector with width and height as numeric values).
- API registration metadata, runtime dispatch, and generated stubs remain synchronized.
- At least one example script demonstrates usage and verifies values are usable in frame logic.
- API behavior is validated across the desktop runtime path and does not regress existing API contracts.

## Constraints

- Keep naming exactly `get_window_size()` (snake_case).
- Do not change existing API signatures unless required and documented.
- If return type choices require API-wide consistency decisions, record via ADR before implementation closeout.
