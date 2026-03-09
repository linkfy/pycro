# pycro vs pygame Compare Benchmark

This folder documents the isolated benchmark stream `phase-compare-benchmark`.

Benchmark scripts:

- `examples/compare_pycro_balls_benchmark.py`
- `examples/compare_pygame_balls_benchmark.py`

Both scripts implement the same interaction contract:

- Simulate bouncing balls on a `1280x720` logical screen
- Interactive mode starts with `500` balls
- Left/Right decreases/increases balls by 50 (repeat enabled)
- New balls are assigned random colors
- HUD reports `balls`, current `fps` (updated per second), `stable/unstable`, `target_fps`, `best stable balls`, and nearest reached count for configured targets

Stability rule used by both runtimes:

- `no_drop` means: sampled FPS per second is at least `95%` of `target_fps`
- canonical metric is `wall_fps` (real elapsed wall time)
- `sim_fps` is diagnostic only (derived from capped simulation dt)

Short auto protocol (mandatory for quick checks):

- auto sessions always start at `3000` balls by default
- auto session duration is `3` seconds (`BENCHMARK_AUTO_SESSION_SECONDS=3`)
- goal is fast detection of FPS improvements using canonical `wall_fps`
- if `3000` is stable (`wall_fps >= 57`), escalate checks to `6000` balls

## Benchmark Environment Recommendations (Mandatory For Fair Comparisons)

- Record runtime stack versions in every benchmark note:
  - Python version (`python --version`)
  - pygame variant and version (`pygame` vs `pygame-ce`)
  - SDL version (`pygame.get_sdl_version()`)
- Prefer direct single-runtime runs when validating source metrics to avoid log-mixing ambiguity:
  - Run `pycro` and `pygame` separately first.
  - Use the matrix `.sh` only after direct runs match expectations.
- Keep pycro in release mode for canonical comparisons.
- If using headless mode (`SDL_VIDEODRIVER=dummy`), mark results as headless and avoid comparing them directly with windowed runs.
- Canonical reference stack for high-load pygame baselines:
  - Python `3.12+`
  - `pygame-ce 2.5.x`
  - SDL `2.30+`

Homogeneous log format (both runtimes):

- `[benchmark] runtime=<pycro|pygame> event=session_start ...`
- `[benchmark] runtime=<pycro|pygame> event=sample second=<n> balls=<count> wall_fps=<value> sim_fps=<value> threshold=<value> status=<stable|unstable> best_stable_balls=<count> nearest_targets=... sim_dt_cap=<value>`
- `[benchmark] runtime=<pycro|pygame> event=summary reason=<exit|escape|auto_session_timeout> ... wall_fps=<value> sim_fps=<value> sim_dt_cap=<value>`

## Run Protocol

1. Interactive: start with default ball count. Auto: starts fixed at `3000`.
2. Increase balls with Right until FPS begins to fluctuate below the no-drop threshold.
3. Decrease/increase around that point to find the highest stable count.
4. Repeat 3 runs per runtime and record the best stable count, plus nearest reached for 3000 and 4000.

## Commands

pycro benchmark:

```bash
cargo run --release -- examples/compare_pycro_balls_benchmark.py
```

quick pycro smoke:

```bash
PYCRO_FRAMES=3 cargo run --release -- examples/compare_pycro_balls_benchmark.py
```

pycro auto session (non-interactive, logs per second + summary):

```bash
BENCHMARK_AUTO=1 BENCHMARK_AUTO_INITIAL_BALLS=3000 BENCHMARK_AUTO_TARGETS=3000 BENCHMARK_AUTO_SESSION_SECONDS=3 PYCRO_FRAMES=900 cargo run --release -- examples/compare_pycro_balls_benchmark.py
```

pygame benchmark:

```bash
python3 examples/compare_pygame_balls_benchmark.py
```

pygame version check (recommended before every session):

```bash
python3 -c "import sys, pygame; print(sys.version); print(pygame.version.ver); print(pygame.get_sdl_version())"
```

pygame auto session (headless-capable):

```bash
SDL_VIDEODRIVER=dummy BENCHMARK_AUTO=1 BENCHMARK_AUTO_INITIAL_BALLS=3000 BENCHMARK_AUTO_TARGETS=3000 BENCHMARK_AUTO_SESSION_SECONDS=3 python3 examples/compare_pygame_balls_benchmark.py
```

run both at the same time:

```bash
./examples/run_compare_benchmarks.sh
```

non-interactive matrix (default 3 runs x [3000] per runtime):

```bash
BENCHMARK_MATRIX=1 BENCHMARK_RUNS=3 BENCHMARK_TARGET_MATRIX=3000 BENCHMARK_AUTO_INITIAL_BALLS=3000 BENCHMARK_AUTO_SESSION_SECONDS=3 ./examples/run_compare_benchmarks.sh
```

`run_compare_benchmarks.sh` uses release mode for pycro by default. Set `PYCRO_CARGO_PROFILE=""` only for debug diagnostics.

Phase 3 note: all exploratory before/after comparisons run with `3s` sessions to accelerate iteration, then promising patches are re-checked with longer sessions before closeout.
Operational plan and latest baseline snapshot are tracked in `docs/perf-phase-3-plan.md`.

if pygame is missing:

```bash
python3 -m pip install pygame
```

## Result Template

Use this table in PR notes or tracker evidence (`wall_fps` canonical):

| Runtime | Phase | Target balls | run 1 (best_stable_balls / wall_fps) | run 2 | run 3 | Final best stable |
| --- | --- | --- | --- | --- | --- | --- |
| pycro | short-protocol | 3000 |  |  |  |  |
| pycro | escalation | 6000 |  |  |  |  |
| pygame | short-protocol | 3000 |  |  |  |  |
| pygame | escalation | 6000 |  |  |  |  |

Nearest-target tracking template:

| Runtime | nearest to 3000 | delta | nearest to 6000 | delta |
| --- | --- | --- | --- | --- |
| pycro |  |  |  |  |
| pygame |  |  |  |  |
