# ADR 0015: `pycro project build` CLI foundation and canonical project contract

status: accepted
date: 2026-03-16

## Context

Phase 14 introduces build orchestration for external game projects, but target packaging is split into later phases (desktop/web/android/ios).

Without a canonical project contract and normalized pre-target representation, each platform phase could drift in input expectations (entry script location, sidecar modules, assets, manifest) and duplicate validation behavior.

## Decision

Define the phase-14 build foundation with a stable CLI and contract surface:

- Add command namespace: `pycro project build`.
- Add root alias: `pycro build`.
- Accept project path as either positional (`<path>`) or explicit (`--project <path>`).
- For `pycro build`, default target is `desktop` when `--target` is omitted.
- Require explicit target on `pycro project build` via `--target <desktop|web|android|ios>`.
- Validate canonical external project contract before any target packaging:
  - required `main.py`
  - supported local `.py` sidecar modules in project root
  - optional `assets/`
  - reserved `pycro-project.toml`
- Produce a normalized `ProjectBundle` + resource provider plan as the canonical input for phases 15-18.

Phase 14 does not implement target packaging itself; successful command execution reports validated contract/bundle and explicit “not implemented yet” messaging.

## Consequences

- Build and packaging strategy is now anchored to one shared project/bundle contract.
- Downstream target phases can focus on packaging logic instead of redefining project discovery rules.
- Existing run/init/stub workflows remain compatible.
- Documentation must reflect both entry surfaces:
  - `pycro project build ...`
  - `pycro build ...`
