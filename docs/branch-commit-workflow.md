# Branch And Commit Workflow

## Branch Policy

- Never implement directly on `main`.
- Use `codex/<phase>-<task>` for implementation branches.
- `develop` is the default integration branch for day-to-day delivery.
- Merge implementation branches into `develop` only after validations, QA gate, and explicit user approval.
- `main` is release-only and must be updated only through a manual ready-for-release pull request from `develop`.
- Repository default branch should be configured as `develop` in Git hosting settings.
- User approval is required per phase closeout and per merge target (`develop` or `main`). The orchestrator must ask before each merge and wait for a direct confirmation.
- Merge/push after implementation is blocked until formal phase closeout is recorded (`docs/phases/<NN-slug>/closeout.md`) and tracker/state reflect `qa=pass` (or explicit waiver).
- Default rule: do not merge into `develop` until the active phase is finalized (`closeout.md` + `qa=pass`/waiver + tracker/state sync). The only allowed bypass is an explicit programmer request.

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

Commit subjects must follow Conventional Commits so release automation can parse history:

- valid examples: `feat(runtime): add direct bridge cache`, `fix(ci): handle optional pygame import`, `chore(docs): sync phase tracker`
- avoid non-conventional prefixes such as `phase05 closeout: ...` or free-form headers with spaces before the first `:`

After required validations pass, `commit-steward` creates a checkpoint commit immediately.
Before any push or merge, local CI-equivalent preflight must pass at minimum:

- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`

For automated releases, repository settings must allow PR creation by automation:

- `Settings -> Actions -> General -> Workflow permissions`: `Read and write permissions`
- `Settings -> Actions`: allow workflows to create pull requests
- if org policy blocks `GITHUB_TOKEN`, provide `RELEASE_PLEASE_TOKEN` secret (PAT or app token with repo contents+PR write)

## ADR Triggers

Create/update an ADR for changes to:

- lifecycle behavior
- public Python API
- build/packaging strategy
- stub generation contract
- platform guarantees
- governance workflow contracts (agents, phases, or validation process)
