# ADR 0011: Update-Only Lifecycle Contract

## Status

Accepted

## Context

The runtime lifecycle has historically dispatched two callbacks from the framework side: optional `setup()` once and required `update(dt)` every frame.

This introduced a framework-owned initialization hook that is not required for the core frame-loop contract and complicates lifecycle guarantees across docs, tests, and generated starter templates.

## Decision

The framework lifecycle contract is now update-only:

- The engine loads `main.py`.
- The engine requires `update(dt: float) -> None`.
- The engine dispatches `update(dt)` once per frame.
- The engine no longer auto-dispatches `setup()`.

User scripts may still define helper functions named `setup` (or any other initializer), but invocation is user-owned and not part of the framework contract.

## Consequences

- Runtime load path must stop invoking `setup()` automatically.
- Runtime tests must assert that `setup()` is not auto-called.
- Documentation and generated starter templates must describe `update(dt)` as the only framework lifecycle callback.
- Existing scripts that still contain `setup()` remain valid if their initialization logic is moved to user-owned control flow (for example guarded initialization inside `update`).
