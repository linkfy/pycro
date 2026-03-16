# Requirements

phase_objective: Establish a stable multiplatform `pycro project` foundation that extends the CLI and project model while preserving all current runtime and Cargo workflows.

## Acceptance Criteria

- `pycro project ...` is defined as a new CLI namespace distinct from current implicit run and `init` behavior.
- `pycro build ...` is supported as a root-level alias for `pycro project build ...`.
- Existing behavior remains preserved by contract:
  - `pycro` without arguments still runs `main.py`
  - `pycro <script_path>` still runs an explicit script
  - `pycro init <project_name>` remains unchanged
  - `cargo build --release` remains a valid way to compile the runtime itself
- Project builds must accept an external project path (`--project <path>` or positional `<path>`) rather than requiring a game folder inside the `pycro` source tree.
- A canonical project structure is defined for future target-specific phases:
  - required `main.py`
  - supported local `.py` modules
  - optional `assets/`
  - reserved `pycro-project.toml`
- A canonical internal `project bundle` concept is defined as the common input for all future build targets.
- Runtime/resource loading architecture is documented in a way that supports desktop, web, Android, and iOS without redefining the project contract per platform.
- Tracker/state/phase docs are synchronized for the new roadmap phases.

## Constraints

- Do not redefine or deprecate current runtime entrypoint behavior in this phase.
- Do not deliver a functional platform package yet; this phase is architectural and CLI-foundational only.
- Any change to build or packaging governance must remain ADR-compatible for later implementation.
