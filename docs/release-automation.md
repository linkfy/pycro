# Release Automation Setup

This project uses Release Please to open release PRs from `main` and a separate workflow to publish `pycro` release artifacts.

## Required Repository Settings

1. `Settings -> Actions -> General -> Workflow permissions`:
   - set `Read and write permissions`.
2. `Settings -> Actions`:
   - allow GitHub Actions to create pull requests.
3. If organization policy restricts `GITHUB_TOKEN` for PR creation:
   - create repository secret `RELEASE_PLEASE_TOKEN` (PAT or app token),
   - grant `contents:write` and `pull_requests:write`.

`release-please.yml` uses `RELEASE_PLEASE_TOKEN` when present and falls back to `GITHUB_TOKEN`.

## Conventional Commit Requirement

Release Please parses commit history. Commit subjects must follow Conventional Commits:

- `feat(runtime): ...`
- `fix(ci): ...`
- `chore(docs): ...`

Commits like `phase05 closeout: ...` are not valid Conventional Commits and may break changelog parsing.

This repository enforces commit format in CI via `.github/workflows/commitlint.yml`.

## Legacy History Bootstrap

`release-please-config.json` includes `bootstrap-sha` to start parsing from a known-safe point and avoid legacy non-conventional commits in initial rollout.
