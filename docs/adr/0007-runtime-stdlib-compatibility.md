# ADR 0007: Runtime Stdlib Compatibility For Gameplay Imports

## Status

Accepted

## Decision

`RustPythonVm` keeps `Interpreter::without_stdlib(...)` for now, and the runtime injects a compatibility stdlib surface before entry-script execution.

Phase-4 contract:

- `import math` and `import os` must work from gameplay scripts without user-side workarounds.
- Runtime installs compatibility modules into `sys.modules` during script load.
- Local sidecar modules in the entry-script directory still take precedence when names collide (for example `math.py` beside `main.py`).

Compatibility surface added:

- `math`: `pi`, `sqrt`, `sin`, `cos`, `hypot`
- `os`: `name`, `sep`, `pathsep`, `linesep`, `getcwd`, `getenv`, `path.basename`

## Consequences

- Phase-4 objective is met for `math`/`os` imports with deterministic runtime behavior and tests.
- This is intentionally a focused compatibility layer, not full RustPython stdlib bootstrap.
- Future full-stdlib enablement can replace this layer behind the same gameplay contract once broader module parity is required.
