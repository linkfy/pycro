# Phase 3 Performance Investigation Plan (25000 balls)

## Goal
Push `pycro` toward or above the `pygame-ce` reference at `25000` balls, using short, repeatable benchmark cycles.

## Canonical protocol (phase 3)
- Session length: `3` seconds per run.
- Load under test: `25000` balls.
- Runs per sample: `3` per runtime/path.
- Report metric: `wall_fps` median (with min/max).
- Mandatory metadata: Python version, pygame/pygame-ce version, SDL version, rust/cargo profile, host machine.

## Baseline evidence (direct 3s runs)
Collected on branch `codex/perf-25000-investigation`.

- `pycro` (`cargo run --release -- examples/phase05_compare_pycro_balls_benchmark.py`):
  - `wall_fps=23.29` (`elapsed=3.02`, `samples=2`, `balls=25000`)
- `pygame-ce` (`.venv314/bin/python examples/phase05_compare_pygame_balls_benchmark.py`):
  - `wall_fps=33.37` (`elapsed=3.00`, `samples=2`, `balls=25000`)
- `macroquad_direct` (`cargo run --release --bin macroquad_direct_balls_benchmark`):
  - `wall_fps=81.16` (`elapsed=3.02`, `samples=3`, `balls=25000`)

Conclusion: Macroquad backend has enough headroom; optimization focus stays on Python/runtime bridge and per-frame dispatch/parse overhead.

## Parallel subagent execution plan
Launch in parallel with disjoint ownership:

1. `runtime-backend-slice` (owner: runtime/backend worker)
- Scope: `src/runtime.rs`, `src/main.rs`, `src/backend.rs`
- Focus: reduce bridge/dispatch overhead, reduce per-frame parsing/allocations.
- Output: one small measurable optimization per commit.

2. `benchmark-harness-slice` (owner: benchmark worker)
- Scope: `examples/phase05_compare_pycro_balls_benchmark.py`, `examples/run_compare_benchmarks.sh`, `examples/compare/README.md`
- Focus: ensure strictly comparable before/after runs and fast reruns.
- Output: deterministic runner improvements only (no simulation rule changes).

3. `measurement-auditor-slice` (owner: measurement worker)
- Scope: result capture + summaries in docs/state only.
- Focus: produce before/after table with medians and regression flags for `3000` and `6000` guardrails.
- Output: evidence-only updates; reject noisy or mixed-log data.

## Iteration loop per optimization candidate
1. Capture `before` median (`3x`, `25000`, `3s`).
2. Apply one optimization patch.
3. Capture `after` median (`3x`, `25000`, `3s`).
4. Keep patch only if median delta is positive.
5. Run guardrails at `3000` and `6000`; reject patch if regression is worse than `5%`.

## Exit criteria
- Primary: `pycro` reaches or exceeds `pygame-ce` reference median at `25000`.
- Secondary: no guardrail regression beyond `5%` at `3000` and `6000`.
- Once the target is consistently met, run one longer confirmation pass before closeout.

## Ready-to-run command set
- `pycro`:
  - `BENCHMARK_AUTO=1 BENCHMARK_AUTO_INITIAL_BALLS=25000 BENCHMARK_AUTO_TARGETS=25000 BENCHMARK_AUTO_SESSION_SECONDS=3 PYCRO_FRAMES=800 cargo run --release -- examples/phase05_compare_pycro_balls_benchmark.py`
- `pygame-ce`:
  - `BENCHMARK_AUTO=1 BENCHMARK_AUTO_INITIAL_BALLS=25000 BENCHMARK_AUTO_TARGETS=25000 BENCHMARK_AUTO_SESSION_SECONDS=3 ./.venv314/bin/python examples/phase05_compare_pygame_balls_benchmark.py`
- `macroquad_direct`:
  - `BENCHMARK_AUTO=1 BENCHMARK_AUTO_INITIAL_BALLS=25000 BENCHMARK_AUTO_TARGETS=25000 BENCHMARK_AUTO_SESSION_SECONDS=3 cargo run --release --bin macroquad_direct_balls_benchmark`
