# Task Implementation

## Execution Steps
1. Activate the web target phase after the embedded payload contract from phase 15 is stable.
2. Implement target parsing and build orchestration for `--target web`.
3. Produce the required web runtime and packaged output using the shared embedded project payload rather than loose project sources.
4. Add smoke validation for web startup, embedded module loading, and asset access.
5. Close the phase with synchronized tracker/state and validation evidence.

## Task Board

| Task ID | Owner | Parallel Team | Status | Branch | Worktree | Validation Gate |
| --- | --- | --- | --- | --- | --- | --- |
| project-web-build | architecture-orchestrator | runtime-worker, platform-worker, api-worker, docs-tracker, qa-reviewer, commit-steward, example-scenario-worker | complete | codex/17-project-web-build | .worktrees/17-project-web-build-orchestrator | web smoke + standard preflight |

## Resume Checkpoint

- kickoff_date: 2026-03-17
- active_branch: `codex/17-project-web-build`
- startup_gate: requirements + design validated, implementation opened
- next_slice:
  - phase 17 closeout recorded (`docs/phases/17-project-web-build/closeout.md`)
  - prepare android phase kickoff handoff notes

## Progress Update (2026-03-17)

- implemented:
  - `pycro project build --target web` now executes real web build orchestration instead of placeholder output
  - output contract emits `dist/web/pycro.wasm`, `dist/web/gl.js`, and `dist/web/index.html`
  - web bootstrap uses local `gl.js` runtime and `load("pycro.wasm")`
  - smoke script added: `scripts/phase17_web_embedded_smoke.py` (artifact layout + embedded payload markers)
- validation:
  - pass: `cargo fmt --all --check`
  - pass: `cargo clippy --all-targets -- -D warnings`
  - pass: `cargo test`
  - pass: `python3 scripts/phase17_web_embedded_smoke.py`
- blocker_resolution:
  - `getrandom` wasm backend aligned with current dependency graph for web builds (explicit wasm-js support path)
  - `mimalloc` moved to non-wasm targets to unblock `wasm32-unknown-unknown` toolchain
  - patched web loader (`gl.js`) now provisions missing wasm import modules/functions (including `__wbindgen_placeholder__`) to avoid browser-time instantiate crash

## Progress Update (2026-03-18)

- implemented:
  - hardened `gl.js` compatibility stubs for common `__wbindgen_placeholder__` import signatures (crypto/time/object helpers)
  - web runtime diagnostic clarity improved by reducing noisy per-import warnings and surfacing runtime errors clearly
  - wasm texture loading path now resolves embedded payload bytes for `assets/...` (`load_texture`) instead of defaulting to white fallback rectangles
  - wasm frame timing path no longer relies on `std::time::Instant` in-browser
- validation:
  - pass: `cargo fmt --all --check`
  - pass: `python3 scripts/phase17_web_embedded_smoke.py`
  - pass: desktop guard validation still compiles and runs smoke (`python3 scripts/phase15_desktop_embedded_smoke.py`)
- scope_note:
  - phase 17 remains a preliminary WASM POC focused on stable local web builds and embedded payload execution; broader production-grade wasm ecosystem compatibility is deferred.

## Reporting Contract

All contributors report summary-only payloads to the orchestrator:

- changed files
- validation evidence
- risks
- follow-ups
- ADR refs
- tracker refs
