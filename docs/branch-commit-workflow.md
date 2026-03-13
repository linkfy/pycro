# Branch And Commit Workflow

## Branch Policy

- Never implement directly on `main`.
- Use `codex/<phase>-<task>` for implementation branches.
- `main` remains the verified integration branch.
- Merge to `main` only after validations, QA gate, and explicit user approval.
- User approval is required per phase closeout. The orchestrator must ask before each merge and wait for a direct confirmation.

## Worktree Policy

Use worktrees for parallel slices and collision prevention.

- Path format: `.worktrees/<phase>-<task>-<agent>`
- One active task ownership per worktree.
- Keep branch/worktree mapping updated in tracker/state.

## Commit Policy

One verified step per commit.

Every implementation commit must include:

- synchronized tracker update
- synchronized machine-state update
- validation evidence
- `qa-reviewer` outcome or explicit waiver

After required validations pass, `commit-steward` creates a checkpoint commit immediately.
Before any push or merge, local CI-equivalent preflight must pass at minimum:

- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`

## ADR Triggers

Create/update an ADR for changes to:

- lifecycle behavior
- public Python API
- build/packaging strategy
- stub generation contract
- platform guarantees
- governance workflow contracts (agents, phases, or validation process)
