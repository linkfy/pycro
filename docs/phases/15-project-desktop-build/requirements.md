# Requirements

phase_objective: Build a desktop artifact from an external `pycro` project path using the shared project contract from phase 14, but with the final runtime embedding the project Python payload instead of shipping loose source files.

## Acceptance Criteria

- `pycro project build --project <path> --target desktop` is defined as the supported desktop build command.
- The command consumes a project directory matching the phase 14 project contract plus a local `pycro` source checkout/toolchain capable of compiling the runtime.
- The phase chooses a single v1 desktop contract and documents it unambiguously.
- Recommended v1 contract: final desktop executable compiled from `pycro` sources with the project Python payload embedded into the produced artifact.
- The final desktop artifact does not rely on loose project `.py` files at runtime.
- The embedded payload supports the project entrypoint, local module imports, and manifest metadata.
- Asset loading uses the same shared packaging architecture so future web/Android/iOS targets do not depend on host filesystem assumptions established only for desktop.
- Existing runtime execution flows remain unaffected when not using `pycro project`.

## Constraints

- Do not broaden this phase into web/mobile target support.
- Do not assume that a desktop-only `dist/` directory layout is the canonical cross-target packaging model.
- If the shared project bundle from phase 14 needs to evolve into an embedded payload contract, record that through ADR + refinement before implementation resumes.
