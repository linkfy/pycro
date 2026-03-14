# Requirements

phase_objective: Produce a web-target build flow for external `pycro` projects without redefining the project contract or desktop packaging contract.

## Acceptance Criteria

- `pycro project build --project <path> --target web` is defined as the supported web build command.
- The command consumes the canonical project structure introduced in phase 14.
- The build produces a web-servable output layout suitable for local or hosted testing.
- The phase explicitly documents that web output is not a single-binary packaging target.
- Web packaging uses the shared project bundle/resource contract rather than introducing a parallel web-only project format.
- The generated web output can load the project entrypoint, local Python modules, and required assets through the web runtime path.

## Constraints

- Do not change desktop packaging semantics in this phase.
- Do not introduce Android or iOS assumptions into the web target contract.
