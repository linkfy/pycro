# Agent Skills Matrix

This file is the canonical skill activation map by agent.

| Agent | Mandatory Skills | Optional Skills | Skill Paths | Activate When | Do Not Activate When |
| --- | --- | --- | --- | --- | --- |
| architecture-orchestrator | orchestration-contract | architecture-review, docs-tracker-contract | `docs/agents/skills/orchestration-contract.md`, `docs/agents/skills/architecture-review.md`, `docs/agents/skills/docs-tracker-contract.md` | any implementation, delegation, integration, or gate workflow; missing phase requirements must trigger planning mode handoff to `phase-planner` | isolated read-only exploration |
| phase-planner | docs-tracker-contract, architecture-review | refinement-sync-contract | `docs/agents/skills/docs-tracker-contract.md`, `docs/agents/skills/architecture-review.md`, `docs/agents/skills/refinement-sync-contract.md` | a new phase starts, requirements are missing, or scope changes materially | phase is execution-only with locked requirements |
| runtime-worker | runtime-contract | perf-optimization-contract | `docs/agents/skills/runtime-contract.md`, `docs/agents/skills/perf-optimization-contract.md` | lifecycle/runtime task is assigned | task is docs-only or pure process governance |
| platform-worker | platform-contract | perf-optimization-contract | `docs/agents/skills/platform-contract.md`, `docs/agents/skills/perf-optimization-contract.md` | platform/input/texture/backend task is assigned | task is API metadata-only |
| api-worker | api-contract, stub-contract | typing-smoke-contract | `docs/agents/skills/api-contract.md`, `docs/agents/skills/stub-contract.md`, `docs/agents/skills/typing-smoke-contract.md` | Python API surface or metadata changes | task is backend-only and API untouched |
| example-scenario-worker | scenario-contract | texture-pack-contract | `docs/agents/skills/scenario-contract.md`, `docs/agents/skills/texture-pack-contract.md` | user-visible feature changes | non-interactive internal refactor |
| docs-tracker | tracker-sync-contract | adr-index-contract | `docs/agents/skills/tracker-sync-contract.md`, `docs/agents/skills/adr-index-contract.md` | tracker/state/phase docs need synchronization | code-only patch with no workflow status change |
| interactive-refinement-recorder | refinement-sync-contract | tracker-sync-contract | `docs/agents/skills/refinement-sync-contract.md`, `docs/agents/skills/tracker-sync-contract.md` | user feedback changes phase scope/tasks | no refinement or scope change occurred |
| flow-visualizer | flow-diagram-contract | lifecycle-contract | `docs/agents/skills/flow-diagram-contract.md`, `docs/agents/skills/lifecycle-contract.md` | lifecycle/API dispatch flow changed | no behavior contract changed |
| qa-reviewer | review-gate-contract | risk-audit-contract | `docs/agents/skills/review-gate-contract.md`, `docs/agents/skills/risk-audit-contract.md` | before implementation commit | exploratory spike with no commit intent |
| commit-steward | commit-gate-contract | changelog-contract | `docs/agents/skills/commit-gate-contract.md`, `docs/agents/skills/changelog-contract.md` | validations are green and QA gate passed/waived | QA unresolved or mandatory validations failing |
| worktree-manager | worktree-contract | merge-prep-contract | `docs/agents/skills/worktree-contract.md`, `docs/agents/skills/merge-prep-contract.md` | two or more active slices can collide or run in parallel | single-slice work with no collision risk |
| merge-integrator | merge-gate-contract | rollback-contract | `docs/agents/skills/merge-gate-contract.md`, `docs/agents/skills/rollback-contract.md` | user approved merge to `main` and gates are green | branch is unverified or user did not approve merge |

## Skill Sources

Project-local skills are documented under `docs/agents/skills/`.
External session-provided skills (for Codex runtime) must be referenced with explicit path when used.

## Model Routing Guardrail

- Planning mode must be orchestrated with ChatGPT 5.4.
- Implementation/review defaults to Codex 5.3 medium.
- Smaller models are only valid for low-risk mechanical tasks and must include rationale in the orchestrator summary.
