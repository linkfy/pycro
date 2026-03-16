# Skill: merge-gate-contract

purpose: Define minimal execution rules for merge-gate-contract.

## Activation

Use this skill only when the owning agent requires it per .

## Output Contract

- concise summary only
- explicit risks and follow-ups
- links to tracker and ADR references when applicable

## Rules

- Block merge into `develop` when the active phase is not finalized.
- Require `closeout.md` + tracker/state sync + `qa=pass` (or explicit waiver) before merge.
- Allow bypass only when the programmer explicitly requests it.
