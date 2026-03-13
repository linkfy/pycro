# Design

## Implementation Approach

- Add a phase-bootstrap contract for Git CD kickoff (branch/worktree allocation + tracker/state registration).
- Integrate Release Please configuration and workflow wiring as the release source of truth for phase-delivered changes.
- Define a readable Python API artifact layout policy for `python/pycro/__init__.pyi` and related docs.
- Enforce model routing in orchestration contracts and playbooks with explicit fallback guidance for lightweight tasks.

## ADR And Contract Alignment

- Add/update ADR if release/governance workflow contracts change.
- Keep this design aligned with phase requirements and the active task table.
