# ADR Log

## Active ADRs

- `0001`: Runtime ownership and lifecycle contract
- `0002`: Python API single source of truth and stub generation
- `0003`: Docs-first governance and review workflow
- `0004`: Single-crate simplification and module-first structure
- `0005`: Python text rendering API (`draw_text`)
- `0006`: Entry-script local import resolution
- `0007`: Runtime stdlib compatibility for gameplay imports (`math`, `os`)
- `0008`: macOS ARM backend default to OpenGL (temporary)
- `0009`: Batched render submission API (`submit_render`)
- `0010`: Phase-folder governance, orchestrated delegation, and worktree-first parallelism
- `0011`: Update-only lifecycle contract (`update(dt)` only, no framework `setup()` dispatch)
- `0012`: Develop-first integration and manual release promotion to `main`
- `0013`: Vec2/Color numeric coercion for Python runtime calls
- `0014`: `pycro generate_stubs` default output path set to project-local `pycro.pyi`
- `0015`: `pycro project build` CLI foundation and canonical external project contract
- `0016`: Embedded project payload build strategy for multi-target packaging
- `0017`: Phase orchestrator ownership and write-constrained worker handoff contract

## Process

- ADR IDs are zero-padded and monotonic.
- Update this index whenever a new ADR is added.
- ADR-required changes cannot merge without the corresponding document update.
