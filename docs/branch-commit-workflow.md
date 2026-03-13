# Branch And Commit Workflow

## Branch Policy

- No direct implementation work on `main`.
- Use `codex/<domain>-<task>` branches for implementation.
- `main` is the verified integration branch: merge changes there only after expected validations pass and behavior is confirmed.
- Keep branch names narrow and task-scoped.
- Before finishing a work block, the agent must ask the user whether to merge into `main`.

## Commit Policy

- One verified step per commit.
- No implementation commit without:
  - tracker update,
  - machine state update,
  - validation evidence,
  - `qa-reviewer` outcome or documented waiver.

## Tracker Merge Notation

- For in-progress items shown as `[-]` in `docs/task-tracker.txt`, append `-> (Not merged)` when the latest work for that item is still on a task branch and not yet merged into `main`.

## ADR Triggers

Create or update an ADR for any change to:

- lifecycle behavior
- public Python API
- build or packaging strategy
- stub generation contract
- platform guarantees
