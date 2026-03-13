# Release Automation Setup

This project uses a two-branch release model:

- `develop`: default integration branch; every push publishes downloadable CI artifacts for validation.
- `main`: release branch; Release Please opens release PRs and final tagged releases are cut from this branch.

## Required Repository Settings

1. `Settings -> Actions -> General -> Workflow permissions`:
   - set `Read and write permissions`.
2. `Settings -> Actions`:
   - allow GitHub Actions to create pull requests.
3. `Settings -> Branches -> Default branch`:
   - set default branch to `develop`.
4. If organization policy restricts `GITHUB_TOKEN` for PR creation:
   - create repository secret `RELEASE_PLEASE_TOKEN` (PAT or app token),
   - grant `contents:write` and `pull_requests:write`.

`release-please.yml` uses `RELEASE_PLEASE_TOKEN` when present and falls back to `GITHUB_TOKEN`.

## Branch Flow

1. Merge validated implementation work into `develop`.
2. Download and test per-push artifacts generated on `develop`.
3. When stable, open a manual ready-for-release PR from `develop` to `main`.
4. After merge to `main`, Release Please updates release PR/changelog and release workflows publish final assets.

## Why Manual "Run workflow" May Not Create Release Assets

This is expected with current workflow behavior:

- `release-artifacts` uploads assets to a GitHub Release only on `release.published`.
- On `workflow_dispatch`, it uploads workflow artifacts, but not release assets.

So if you run it manually, seeing only `Source code` in Releases is normal unless the job was triggered by an actual published release event.

## Conventional Commit Requirement

Release Please parses commit history. Commit subjects must follow Conventional Commits:

- `feat(runtime): ...`
- `fix(ci): ...`
- `chore(docs): ...`

Commits like `phase05 closeout: ...` are not valid Conventional Commits and may break changelog parsing.

This repository enforces commit format in CI via `.github/workflows/commitlint.yml`.

Important: docs-only commits (`docs(...)`) usually do not create a new release. Use `fix(...)` or `feat(...)` when you intentionally need a new release PR/version bump.

## Legacy History Bootstrap

`release-please-config.json` includes `bootstrap-sha` to start parsing from a known-safe point and avoid legacy non-conventional commits in initial rollout.
