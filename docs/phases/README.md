# Phase Index

This directory stores the canonical sequential delivery phases.

## Numbering Rule

- Phase folders are strictly consecutive and zero-padded: `NN-<slug>`.
- The active phase is tracked in both `docs/task-tracker.txt` and `state/repo-state.json`.
- Non-linear streams (benchmark studies, cross-cutting docs, incident tracks) live in `docs/streams/`.

## Phase Folders

- `00-foundation` (closed)
- `01-basic-example` (closed)
- `02-api-direct-bridge` (closed)
- `03-python-module-imports` (closed)
- `04-runtime-stdlib` (closed)
- `05-platform-input-texture` (closed)
- `06-ci-visual-payload` (complete)
- `07-manual-playtest-evidence` (closed)
- `08-git-cd-release-automation` (complete)
- `09-project-init-quickstart` (complete)
- `10-update-only-lifecycle` (complete)
- `11-develop-integration-artifacts` (complete)
- `12-vec2-color-float-coercion` (complete)
- `13-cli-generate-stubs-command` (complete)
- `14-project-build-foundation` (complete)
- `15-project-desktop-build` (complete)
- `16-spec-driven-agent-workflow-hardening` (in_progress)
- `17-project-web-build` (planned)
- `18-project-android-build` (planned)
- `19-project-ios-build` (planned)

## Required Files Per Phase

Each phase folder must contain:

- `README.md`
- `requirements.md`
- `design.md`
- `implementation.md`
- `interactive-refinement.md`

When phase status changes to `closed`, add:

- `closeout.md`

## Phase Execution Order (Mandatory)

The orchestrator must enforce this order in every phase:

1. `requirements.md` validation
2. `design.md` validation against requirements
3. `implementation.md` execution with delegated team ownership
4. `interactive-refinement.md` updates when scope changes

Implementation cannot begin until requirements and design validation are complete.
If a phase lacks concrete requirements, the orchestrator must switch to planning mode and produce `requirements.md`, `design.md`, and `implementation.md` execution steps before implementation can start.
