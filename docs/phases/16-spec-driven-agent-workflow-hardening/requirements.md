# Requirements

phase_objective: Improve repository contracts so agents can execute future phases with less drift, better automation, and clearer operational recovery paths.

## Acceptance Criteria

- The repository defines a canonical phase-16 improvement scope focused on spec-driven workflow hardening rather than platform packaging.
- Tracker/state/phase registry validation is tightened so phase numbering/status/paths cannot silently drift.
- Phase kickoff mechanics are documented or automated so activating a new phase is reproducible and low-risk.
- Stream/incident handling is clarified so active phase work and parallel streams do not create ambiguous "current" state.
- Release/test ergonomics are improved where they directly reduce agent/operator confusion:
  - explicit CLI help path,
  - artifact smoke expectations,
  - stream closeout expectations.
- The phase leaves phases 17-19 positioned as:
  - 17 web,
  - 18 android,
  - 19 ios.

## Constraints

- Do not broaden this phase into shipping the web target itself.
- Keep public packaging/runtime contracts stable unless an ADR is required.
- Any governance workflow contract change must be synchronized in docs/state and captured through ADR if the contract materially changes.
