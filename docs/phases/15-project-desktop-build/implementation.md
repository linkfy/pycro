# Task Implementation

## Execution Steps
1. Record the embedded-payload refinement and ADR before further implementation.
2. Define the shared embedded project payload contract that desktop/web/Android/iOS can all consume.
3. Implement the desktop target adapter as a source-assisted builder that compiles a desktop artifact with embedded project Python payload.
4. Add smoke validation for startup, embedded local imports, and asset access through the cross-target packaging abstraction.
5. Close the phase with synchronized tracker/state and validation evidence.

## Task Board

| Task ID | Owner | Parallel Team | Status | Branch | Worktree | Validation Gate |
| --- | --- | --- | --- | --- | --- | --- |
| project-desktop-build | architecture-orchestrator | runtime-worker, platform-worker, api-worker, docs-tracker, qa-reviewer, commit-steward, example-scenario-worker | complete | codex/15-project-desktop-build | .worktrees/15-project-desktop-build-orchestrator | embedded desktop artifact smoke + standard preflight |

## Resume Checkpoint

- active_branch: `codex/15-project-desktop-build`
- worktree: `.worktrees/15-project-desktop-build-orchestrator`
- accepted_direction: desktop/web/Android/iOS must share one embedded project payload strategy; loose `dist/` packaging is rejected
- code_status: embedded payload build foundation is now implemented (`build.rs` payload generation + desktop `cargo build --release` orchestration + embedded runtime staging path)
- next_slice:
  - phase closeout completed in `closeout.md`; next direct build target now follows hardening phase 16 and resumes at phase 17 (`project-web-build`)

## Reporting Contract

All contributors report summary-only payloads to the orchestrator:

- changed files
- validation evidence
- risks
- follow-ups
- ADR refs
- tracker refs
