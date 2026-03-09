# ADR 0006: Entry-Script Local Import Resolution

## Status

Accepted

## Decision

When loading the configured entry script, runtime inserts that script's directory into RustPython `sys.path` before executing the script.

This guarantees local sidecar imports from the same directory (for example `main.py` importing `player.py`) work deterministically, independent of process working directory.

## Consequences

- `load_script` now configures import resolution as part of lifecycle boot.
- Runtime tests must cover a real `main.py` + `player.py` import case.
- Example scenarios can safely split logic into multiple Python files while keeping `examples/` flat.
