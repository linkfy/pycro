# ADR 0004: Single-Crate Simplification And Module-First Structure

## Status

Accepted

## Decision

For phase-1 execution speed, `pycro_cli`, `pycro_runtime`, and `pycro_api` are unified into one crate: `pycro_cli`. Their boundaries are preserved as modules (`main`, `runtime`, `api`, `backend`) instead of separate crates.

Macroquad ownership remains isolated behind a backend contract (`backend` module) so the rendering/runtime owner can be swapped in the future without changing the Python public contract.

## Consequences

- The workspace has one active crate to reduce cross-crate coordination overhead.
- Public Python API metadata and stub generation remain single-source inside `api`.
- Runtime lifecycle contract remains explicit in `runtime`.
- Backend replacement remains possible via the backend trait contract.

