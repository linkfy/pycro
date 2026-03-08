# pycro Agent Instructions

This repository operates from canonical docs under `docs/` and a machine-readable state snapshot in `state/repo-state.json`.

Read these first:

1. `docs/project-vision.md`
2. `docs/architecture-plan.md`
3. `docs/task-tracker.txt`
4. `docs/agent-playbook.md`
5. `docs/agent-registry.md`
6. `docs/adr/README.md`
7. `docs/platform-capability-matrix.md`
8. `docs/validation-policy.md`
9. `docs/branch-commit-workflow.md`
10. `state/repo-state.json`

Operating rules:

- Treat the docs above as the source of truth. If code and docs diverge, stop and reconcile.
- Never work on `main`. Use `codex/<domain>-<task>` branches only.
- One verified step per commit. Every implementation commit must include tracker updates, validation evidence, and a `qa-reviewer` outcome or an explicit waiver.
- Any change to lifecycle, public API, build strategy, stub generation contract, or platform guarantees requires an ADR entry under `docs/adr/`.
- Workers do not hand raw logs to the orchestrator. They update concise summaries only: changed files, validation evidence, risks, follow-ups, and ADR/task references.
- The canonical Python-facing API lives in Rust metadata inside the `api` module of `pycro_cli`. `python/pycro/__init__.pyi` must be generated from that metadata and checked for drift.

Scope reminders:

- `pycro_cli`: single project containing `main` + `runtime` + `api` + `backend` modules.
- `runtime` module: RustPython embedding contract, lifecycle dispatch, exception/reporting surfaces.
- `api` module: public Python module contract, registration metadata, stub generation.
- `backend` module: Macroquad-owner contract boundary with swappable backend interface.
