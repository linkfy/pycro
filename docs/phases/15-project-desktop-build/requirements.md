# Requirements

phase_objective: Build a desktop distributable from an external `pycro` project path using the shared `project bundle` contract defined in phase 14.

## Acceptance Criteria

- `pycro project build --project <path> --target desktop` is defined as the supported desktop build command.
- The command consumes a project directory matching the phase 14 project contract.
- The build produces a deterministic desktop output under `dist/`.
- The phase chooses a single v1 packaging contract and documents it unambiguously.
- Recommended v1 contract: desktop distributable directory rather than single-file binary packaging.
- The packaged output includes the runtime plus the project scripts/assets required to launch the project.
- The resulting desktop package launches the project entrypoint and supports local module imports and `assets/` loading.
- Existing runtime execution flows remain unaffected when not using `pycro project`.

## Constraints

- Do not broaden this phase into web/mobile target support.
- Do not change the canonical project structure or bundle contract defined in phase 14 unless a blocking issue is discovered and recorded through refinement.
