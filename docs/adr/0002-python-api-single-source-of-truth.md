# ADR 0002: Python API Single Source Of Truth

## Status

Accepted

## Decision

The public Python API is authored once in Rust metadata inside the `api` module of `pycro_cli`. That metadata drives both runtime registration planning and generation of `python/pycro/__init__.pyi`.

## Consequences

- Manual editing of the canonical stub file is not allowed.
- Every public API addition must declare signature, summary, and platform support.
- CI can fail deterministically on stub drift.
