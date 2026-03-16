# Requirements

phase_objective: Produce a web-target build flow for external `pycro` projects using the same embedded project payload strategy proven by desktop, without redefining the canonical project contract.

## Acceptance Criteria

- `pycro project build --project <path> --target web` is defined as the supported web build command.
- The command consumes the canonical project structure introduced in phase 14.
- The build produces a web-servable output layout suitable for local or hosted testing.
- The phase explicitly documents that web output is not a single-binary packaging target even though it reuses the same embedded project payload strategy.
- Web packaging uses the shared embedded payload/resource contract rather than introducing a parallel web-only project format.
- The generated web output can load the project entrypoint, local Python modules, and required assets without depending on loose project `.py` files copied next to the served output.

## Constraints

- Do not redefine the shared embedded payload contract in a web-only way.
- Do not introduce Android or iOS assumptions into the web target contract.
