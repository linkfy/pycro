# ADR 0016: Embedded project payload build strategy for multi-target packaging

status: accepted
date: 2026-03-17

## Context

Phase 15 was initially trending toward a desktop-only `dist/` layout that copied runtime and loose project files next to each other.

That direction does not scale well to the future target roadmap:

- web does not naturally consume loose project `.py` files next to the runtime;
- Android and iOS cannot rely on arbitrary host filesystem layouts at runtime;
- a desktop-only loose-file contract would force later phases to invent a second packaging model for embedded/mobile targets.

The product goal is to make future game development easier even if builders compile from a local `pycro` source checkout. The important outcome is that target packaging is straightforward and consistent, not that `pycro` itself ships as a prebuilt closed builder.

## Decision

Adopt a shared embedded project payload strategy for build phases 15-18:

- `pycro project build` remains the CLI entry surface.
- The external project contract from phase 14 remains valid as the authoring input (`main.py`, sidecar `.py`, optional `assets/`, optional `pycro-project.toml`).
- Downstream build phases must convert that authoring input into a canonical packaged payload suitable for embedding or target-native bundling.
- Phase 15 desktop becomes the first concrete implementation of that strategy:
  - compile a desktop artifact from `pycro` sources,
  - embed the project Python payload into the produced artifact instead of shipping loose `.py` files beside it.
- Future phases must reuse the same payload strategy:
  - web: packaged payload consumed by web runtime/bootstrap,
  - Android: payload embedded into app resources/assets,
  - iOS: payload embedded into app bundle resources.

## Consequences

- Desktop no longer defines the canonical packaging model as a loose `dist/` directory.
- The build roadmap is now aligned around one cross-target packaging abstraction instead of one per platform family.
- Source-assisted builder workflows are explicitly acceptable when they reduce friction for downstream game developers.
- Phase 15 implementation must be replanned before code continues.
