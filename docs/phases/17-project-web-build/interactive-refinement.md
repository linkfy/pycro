# Interactive Refinement

Use this document to record requirement and scope adjustments discovered during execution.

## Update Rule

When refinement changes phase scope or task ordering:

1. Update this file first.
2. Update `implementation.md` task board.
3. Sync `docs/task-tracker.txt`.
4. Sync `state/repo-state.json`.

No change is considered active until all four artifacts are synchronized.

## Renumbering Note

- 2026-03-17: Web build phase shifted from 16 to 17 after inserting `16-spec-driven-agent-workflow-hardening`.
- 2026-03-17: Phase 17 kickoff activated; requirements/design validated and implementation opened on `codex/17-project-web-build`.
- 2026-03-17: Phase 17 slice 1 implemented web output orchestration (`dist/web`: wasm + gl.js + index.html) and added `scripts/phase17_web_embedded_smoke.py`; smoke currently blocked by wasm dependency configuration (`getrandom` web backend) in this toolchain.
- 2026-03-17: Step-1/2 refinement complete: resolved wasm blockers (`getrandom` web backend + non-wasm `mimalloc`) and validated embedded payload web smoke PASS without relying on loose project `.py` files beside the web output.
- 2026-03-17: Browser black-screen incident root cause identified as wasm imports (`__wbindgen_placeholder__`) unresolved by default `gl.js`; loader compatibility shims were added in vendored `gl.js` for non-`env` module imports and known `__wbindgen_*` symbols.
- 2026-03-18: Web runtime compatibility shims were hardened for `crypto/time/object` wasm-bindgen imports after browser errors (`getTimezoneOffset`, `getRandomValues`) during real project validation.
- 2026-03-18: POC stability issue discovered: textures rendered white in web despite assets present; backend `load_texture` on wasm was updated to load from embedded payload bytes (`assets/...`) to restore expected rendering.
- 2026-03-18: Phase documentation expanded with explicit WASM POC scope, build/run commands, non-web parity checks, and troubleshooting guidance for common script/runtime pitfalls observed in the field.
