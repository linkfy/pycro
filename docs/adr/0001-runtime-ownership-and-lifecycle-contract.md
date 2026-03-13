# ADR 0001: Runtime Ownership And Lifecycle Contract

## Status

Superseded by ADR 0011 for lifecycle callback semantics

## Decision

Macroquad owns the frame loop and platform-facing runtime concerns. RustPython owns Python script loading and execution. The engine loads `main.py`, calls optional `setup()`, and calls `update(dt)` once per frame.

## Consequences

- Lifecycle dispatch is a runtime contract, not an implementation detail.
- Platform code must not take control away from the Macroquad-owned frame loop.
- Any future scripting backend must justify why it preserves or replaces the RustPython contract.
