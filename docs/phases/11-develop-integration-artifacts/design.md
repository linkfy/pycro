# Design

## Implementation Approach

- Update governance docs (`AGENTS.md`, branch workflow docs, agent playbook/registry/skills, release automation docs) to encode a develop-first policy.
- Extend CI and commitlint branch triggers to include `develop`.
- Add a new `develop-artifacts` workflow that builds release binaries and uploads downloadable artifacts on every push to `develop`.
- Keep `release-please` workflow on `main` so formal release PR/tag flow remains release-branch scoped.
- Synchronize phase registry artifacts (`docs/phases/README.md`, `docs/task-tracker.txt`, `state/repo-state.json`) to phase 11 in-progress status.

## ADR And Contract Alignment

- This phase changes governance workflow contracts (default merge branch and release promotion path), so ADR update is required.
