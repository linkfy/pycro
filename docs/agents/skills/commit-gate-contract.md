# Skill: commit-gate-contract

purpose: Define minimal execution rules for commit-gate-contract.

## Activation

Use this skill when `commit-steward` is closing a checkpoint commit or preparing push/merge handoff.

## Output Contract

- concise summary only
- explicit risks and follow-ups
- links to tracker and ADR references when applicable

## Required Checks

1. Confirm validation gates are green (`fmt`, `clippy`, `test`, plus phase-required checks).
2. Confirm tracker/state synchronization is staged with the commit.
3. Confirm commit subjects pass commitlint contract:
   - allowed types: `build`, `chore`, `ci`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert`, `style`, `test`, `merge`
   - format: `<type>(<optional-scope>): <subject>` or `<type>: <subject>`
4. If any commit subject violates the contract, stop and require local rewrite before push/merge.
