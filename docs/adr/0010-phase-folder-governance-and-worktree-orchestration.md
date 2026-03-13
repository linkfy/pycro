# ADR 0010: Phase-Folder Governance, Orchestrated Delegation, and Worktree-First Parallelism

## Status

Accepted

## Context

The previous documentation layout mixed sequential phases, parallel streams, and long-form execution notes in root-level files. This created visual noise, duplicated state between tracker and machine snapshot, and increased context cost for orchestrator-led work.

The team requires:

- consecutive numbered phases with predictable folder structure,
- root tracker readability without narrative duplication,
- explicit orchestration contracts with no "god agent" behavior,
- parallel-safe task execution with worktree assignment,
- explicit skill activation rules by agent with path references.

## Decision

1. Sequential phases are canonicalized under `docs/phases/NN-<slug>/`.
2. Each phase folder must include `README.md`, `requirements.md`, `design.md`, `implementation.md`, and `interactive-refinement.md`.
3. Non-sequential tracks move to `docs/streams/`.
4. `docs/task-tracker.txt` remains in `docs/` root and uses compact table format.
5. `state/repo-state.json` becomes a compact operational snapshot aligned to tracker rows.
6. Orchestrator-led delegation is mandatory for implementation work; no single-agent end-to-end execution.
7. Worktrees are mandatory when parallel execution can collide; naming convention is standardized.
8. Agent skill activation contracts are centralized in `docs/agents/agent-skills.md` with explicit skill file paths.

## Consequences

Positive:

- reduced visual noise in root docs,
- lower orchestration context overhead,
- explicit ownership boundaries and better parallel throughput,
- deterministic validation of phase structure and tracker/state sync.

Tradeoff:

- additional governance overhead to maintain phase docs and skill matrix,
- stricter workflow rules that require disciplined branch/worktree hygiene.

## Validation

- `scripts/validate_governance.py` enforces required canonical docs, phase folder structure, consecutive numbering, and tracker/state consistency.
