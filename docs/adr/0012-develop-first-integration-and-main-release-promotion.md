# ADR 0012: Develop-First Integration And Main Release Promotion

## Status

Accepted

## Context

The previous governance model treated `main` as the default integration branch for day-to-day work.
The team now needs:

- frequent testable artifacts for each integration push,
- a stable release gate before `main`,
- explicit manual promotion control for release readiness.

## Decision

Adopt a develop-first workflow:

- `develop` becomes the default integration branch.
- Implementation branches (`codex/<phase>-<task>`) merge into `develop` after validation gates.
- `main` becomes release-only.
- Promotion to `main` happens only through a manual ready-for-release PR from `develop`.
- CI artifact workflow publishes downloadable binaries on every push to `develop`.
- Release Please remains tied to `main` for release PR/tag lifecycle.

## Consequences

- Governance and agent docs must encode the `develop` default merge target.
- `ci` and `commitlint` branch triggers must include `develop`.
- A dedicated `develop` artifact workflow is required for tester downloads.
- Release stability improves because `main` only advances through explicit promotion.
