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
- `05-platform-input-texture` (in progress)
- `06-ci-visual-payload` (planned)
- `07-manual-playtest-evidence` (planned)

## Required Files Per Phase

Each phase folder must contain:

- `README.md`
- `requirements.md`
- `design.md`
- `implementation.md`
- `interactive-refinement.md`

When phase status changes to `closed`, add:

- `closeout.md`
