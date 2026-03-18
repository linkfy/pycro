# ADR 0017: Phase Orchestrator Ownership and Write-Constrained Worker Handoff

## Status

accepted

## Context

The governance contract already required delegation and summary-only worker reporting, but it did not explicitly enforce orchestrator ownership for every active phase.
The contract also lacked a canonical operating mode for write-constrained workers.
In practice, workers may be read-only in some environments and still provide high-value analysis or patch intent; without a formal fallback, this caused repeated operator clarification and inconsistent execution.

## Decision

1. Every phase marked `planned` or `in_progress` must have `architecture-orchestrator` as recorded owner.
2. Delegated workers may own implementation slices, but not phase lifecycle ownership.
3. If a worker cannot write, the worker must provide summary/input handoff payloads:
   - `target_files`
   - `proposed_edits`
   - `integration_notes`
4. The orchestrator performs final repository edits and integration for write-constrained worker handoffs.
5. Tracker/state must record this fallback mode when it is used.

## Consequences

Positive:

- phase ownership remains unambiguous,
- write-constrained environments do not block delegated execution,
- orchestration remains mandatory without reverting to god-agent behavior.

Tradeoffs:

- orchestrator integration burden increases when workers cannot write,
- governance validation becomes stricter and requires tighter tracker/state synchronization.
