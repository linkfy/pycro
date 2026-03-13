# Requirements

phase_objective: Add phase-launch continuous-development automation, Release Please per-phase release workflow, and readable Python API artifacts.

## Acceptance Criteria

- Every new phase launch follows a deterministic Git CD bootstrap flow:
  - create/switch `codex/<phase>-<task>` branch,
  - allocate required worktrees for parallel slices,
  - register branch/worktree ownership in tracker/state before implementation.
- Release Please (Google) is configured so merged phase changes are grouped into release PRs/tags with reproducible changelog output.
- Release workflow publishes a `pycro` binary artifact for each target platform/architecture pair:
  - Linux: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`
  - macOS: `x86_64-apple-darwin`, `aarch64-apple-darwin`
  - Windows: `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`
- Release automation policy is explicit: phase closeout requires a release automation check and evidence in tracker/state.
- Python API artifact readability is improved and documented so contributors can quickly scan exported symbols and signatures.
- GitHub CI typing smoke does not fail on optional benchmark-only dependency imports (for example `pygame`) while strict typing remains enabled for project-managed modules.
- Orchestrator model-routing rule is enforced:
  - planning mode uses ChatGPT 5.4,
  - implementation/review defaults to Codex 5.3 medium,
  - simpler low-risk tasks may use smaller models with explicit reason logged.

## Constraints

- Keep requirements synchronized with `docs/task-tracker.txt` and `state/repo-state.json`.
- Preserve current orchestrator-first and no-god-agent governance rules.
- If scope changes, update `interactive-refinement.md` before implementation continues.
