# Design

## Implementation Approach

- Implement the first concrete target adapter: `desktop`.
- Reuse the shared project contract from phase 14, but pivot the downstream packaging model toward a canonical embedded project payload that future targets can also consume.
- Treat desktop as the first proving ground for a source-assisted builder workflow: compile `pycro` with project payload embedded rather than copying loose `.py` sources beside the executable.
- Keep the current runtime path intact for direct script execution and only activate embedded-payload logic inside the `pycro project` build path.
- Define desktop validation around startup, embedded import resolution, and embedded/resource-backed asset access.

## ADR And Contract Alignment

- Desktop packaging now changes the broader build strategy, so the embedded-payload contract must be captured through ADR before implementation resumes.
