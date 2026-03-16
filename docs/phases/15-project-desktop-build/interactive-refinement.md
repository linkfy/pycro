# Interactive Refinement

Use this document to record requirement and scope adjustments discovered during execution.

## 2026-03-17 - Embedded Payload Pivot

- user_feedback: the planned `dist/`-style desktop packaging is not acceptable because it does not scale to a consistent wasm/Android/iOS embedding strategy.
- decision: phase 15 must pivot from loose packaged project files toward a source-assisted builder that produces a desktop artifact with embedded Python payload.
- implication: desktop becomes the first implementation of a cross-target embedded payload architecture rather than a desktop-only filesystem layout.
- implementation_status:
  - provisional loose-file `dist/desktop` packaging is rejected and not part of phase-15 contract.
  - embedded payload foundation is implemented: `build.rs` payload generation, desktop build orchestration via `cargo build --release`, and runtime extraction for embedded startup.
- resume_anchor: implementation should resume from `docs/phases/15-project-desktop-build/implementation.md` `Resume Checkpoint` section.

## Update Rule

When refinement changes phase scope or task ordering:

1. Update this file first.
2. Update `implementation.md` task board.
3. Sync `docs/task-tracker.txt`.
4. Sync `state/repo-state.json`.

No change is considered active until all four artifacts are synchronized.
