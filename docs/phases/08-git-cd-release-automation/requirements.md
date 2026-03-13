# Requirements

phase_objective: Add phase-launch continuous-development automation, Release Please per-phase release workflow, and readable Python API artifacts.

## Acceptance Criteria

- Every new phase launch follows a deterministic Git CD bootstrap flow:
  - create/switch `codex/<phase>-<task>` branch,
  - allocate required worktrees for parallel slices,
  - register branch/worktree ownership in tracker/state before implementation.
- Release Please (Google) is configured so merged phase changes are grouped into release PRs/tags with reproducible changelog output.
- Release automation policy is explicit: phase closeout requires a release automation check and evidence in tracker/state.
- Python API artifact readability is improved and documented so contributors can quickly scan exported symbols and signatures.
- Orchestrator model-routing rule is enforced:
  - planning mode uses ChatGPT 5.4,
  - implementation/review defaults to Codex 5.3 medium,
  - simpler low-risk tasks may use smaller models with explicit reason logged.

## Constraints

- Keep requirements synchronized with `docs/task-tracker.txt` and `state/repo-state.json`.
- Preserve current orchestrator-first and no-god-agent governance rules.
- If scope changes, update `interactive-refinement.md` before implementation continues.
