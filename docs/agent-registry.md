# Agent Registry

This file defines ownership and boundaries. Skill activation rules live in `docs/agents/agent-skills.md`.

| Agent | Role | Owns | Inputs | Outputs |
| --- | --- | --- | --- | --- |
| architecture-orchestrator | orchestrator | phase selection, delegation, integration decisions, gate readiness | worker summaries, tracker state, ADR refs | integrated decisions, go/no-go, delegated execution plan |
| phase-planner | planner | phase requirement/design decomposition before implementation starts | phase objective, constraints, prior phase docs | phase-local requirement/design updates |
| runtime-worker | worker | RustPython embedding, lifecycle dispatch, runtime errors/reporting | architecture plan, runtime ADRs, task brief | runtime code changes, runtime tests, summary evidence |
| platform-worker | worker | Macroquad loop, render/input/assets and platform capability boundaries | platform matrix, phase requirements | backend/platform changes and capability evidence |
| api-worker | worker | `pycro` API metadata, registration contract, stub compatibility | architecture plan, API ADRs | API surface changes, stub evidence, typing evidence |
| example-scenario-worker | worker | playable scenarios under `examples/*.py` for user-visible features | feature brief, existing examples, asset constraints | scenario scripts, run instructions, expected behavior checklist |
| docs-tracker | worker | synchronized tracker/state updates, phase doc hygiene, and incident-resolution log upkeep | worker summaries, review outcomes, phase updates | `docs/task-tracker.txt` + `state/repo-state.json` sync updates + incident log references |
| interactive-refinement-recorder | worker | requirement/task refinements inside phase docs and sync triggers | user feedback, orchestrator decisions | `interactive-refinement.md` updates + sync checklist |
| flow-visualizer | worker | lifecycle/dispatch Mermaid diagrams | architecture plan, runtime/API changes | refreshed flow diagrams with concise notes |
| qa-reviewer | reviewer | post-implementation review gate and waiver decisions | diffs, validation evidence, tracker links | findings list or explicit waiver |
| commit-steward | steward | checkpoint commit creation after green validations + conventional commitlint subject enforcement | qa outcome, validation report, tracker/state sync, pending commit subjects | commit SHA or explicit block reason (including commitlint violations) |
| worktree-manager | worker | create/remove worktrees for parallel slices, prevent collisions | orchestrator parallelization plan | worktree map and branch/worktree assignment evidence |
| merge-integrator | worker | controlled merge into `develop` by default, and manual ready-for-release promotion from `develop` to `main` | green validations, qa pass, branch status, user "merge now" confirmation + target branch | merge commit and post-merge sync evidence |

## Summary Contract

All worker outputs to the orchestrator must use:

- `changed_files`
- `validation_evidence`
- `risks`
- `follow_ups`
- `adr_refs`
- `tracker_refs`

## Model Policy

- Official orchestrator planning mode uses ChatGPT 5.4.
- Default model for execution/review/sync is Codex 5.3 medium.
- Smaller models are allowed only for low-risk mechanical tasks with explicit rationale.
