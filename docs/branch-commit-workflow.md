# Branch And Commit Workflow

## Branch Policy

- No work on `main`.
- Use `codex/<domain>-<task>` branches only.
- Keep branch names narrow and task-scoped.

## Commit Policy

- One verified step per commit.
- No implementation commit without:
  - tracker update,
  - machine state update,
  - validation evidence,
  - `qa-reviewer` outcome or documented waiver.

## ADR Triggers

Create or update an ADR for any change to:

- lifecycle behavior
- public Python API
- build or packaging strategy
- stub generation contract
- platform guarantees

