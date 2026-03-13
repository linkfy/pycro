# Interactive Refinement

Use this document to record requirement and scope adjustments discovered during execution.

## Update Rule

When refinement changes phase scope or task ordering:

1. Update this file first.
2. Update `implementation.md` task board.
3. Sync `docs/task-tracker.txt`.
4. Sync `state/repo-state.json`.

No change is considered active until all four artifacts are synchronized.

## 2026-03-15 Refinement

- User requested explicit Release Please implementation plus release artifacts named `pycro` for Linux/macOS/Windows on both x64 and ARM.
- User requested CI stability fix for mypy failure caused by optional `pygame` benchmark import.
- Implementation/docs were expanded to include both requirements and validation evidence for these additions.

## 2026-03-15 Release Hardening Refinement

- Release-please failure analysis added two mandatory controls:
  - conventional commit enforcement in CI (`commitlint`) so release parsing stays stable,
  - explicit release token policy (`RELEASE_PLEASE_TOKEN` fallback) when repository policy blocks PR creation from `GITHUB_TOKEN`.
- Release configuration includes `bootstrap-sha` to avoid legacy non-conventional commit parsing in initial rollout.
