# ADR 0014: `pycro generate_stubs` project-local default output

status: accepted
date: 2026-03-16

## Context

Phase 13 introduced `pycro generate_stubs` in the main CLI. Its first implementation defaulted to writing `python/pycro/__init__.pyi`, which matches repository maintenance workflows but does not match everyday project usage where users run `pycro` from a game project directory with `main.py`.

Project scaffolding created by `pycro init` already uses `pycro.pyi` in the project root.

## Decision

`pycro generate_stubs` now defaults to writing `pycro.pyi` in the current working directory (project-local path).

- `pycro generate_stubs` -> writes `./pycro.pyi`
- `pycro generate_stubs --check` -> checks `./pycro.pyi`
- explicit custom path remains supported (`pycro generate_stubs <path>`)

The internal helper workflow `cargo run --bin generate_stubs -- ...` remains available for repository-level canonical stub validation.

## Consequences

- Better default UX for project authors: stubs are generated next to `main.py` by default.
- Existing CI/governance checks for repository canonical stubs remain unchanged.
- Documentation must clearly distinguish project-local CLI default from repository maintenance commands.
