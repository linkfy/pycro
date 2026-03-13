# Requirements

phase_objective: Move daily delivery to `develop` and make release promotion to `main` an explicit manual gate.

## Acceptance Criteria

- Governance docs state that implementation branches merge into `develop` by default.
- Governance docs state that `main` is release-only and receives changes through manual ready-for-release PRs from `develop`.
- CI and commitlint workflows run on `develop` pushes.
- A dedicated workflow publishes downloadable artifacts for each push to `develop`.
- Release Please remains tied to `main` release flow and is documented in the new develop-first model.
- Tracker/state/phase docs are synchronized for phase 11 activation.

## Constraints

- Keep sequential phase governance under `docs/phases/`.
- Keep branch naming and delegation contracts unchanged for implementation slices (`codex/<phase>-<task>`).
- Do not remove release artifact publishing on release events.
