# Design

## Implementation Approach

- Add cache-hit short-circuit in backend `load_texture(path)`:
  - if path is already present in backend texture map, return handle immediately;
  - otherwise load/decode once and insert into cache.
- Keep `TextureHandle` contract stable (`str` path handle) to avoid API/stub churn.
- Preserve existing platform-specific logic (desktop file read, embedded payload paths, android lazy load behavior).
- Consider optional negative-cache follow-up (missed paths) only if required by profiling evidence.

## Data/Behavior Contract

- Cache key: normalized texture path string as used by current handle contract.
- Cache value: backend-native texture object.
- Cache lifetime: runtime process lifetime (no hot-reload invalidation in v1 unless refinement adds explicit policy).

## Validation Strategy

- Backend unit tests for repeated `load_texture(path)` cache-hit behavior.
- Runtime smoke test that invokes repeated `load_texture` calls and verifies stable behavior.
- Full policy gates (`fmt`, `clippy`, `test`, stubs, mypy, docs, governance).
