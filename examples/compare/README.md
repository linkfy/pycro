# pycro vs pygame Compare Benchmark

This folder documents the isolated benchmark stream `phase-compare-benchmark`.

Benchmark scripts:

- `examples/compare_pycro_balls_benchmark.py`
- `examples/compare_pygame_balls_benchmark.py`

Both scripts implement the same interaction contract:

- Simulate bouncing balls on a `1280x720` logical screen
- Start with `500` balls
- Left/Right decreases/increases balls by 50 (repeat enabled)
- New balls are assigned random colors
- HUD reports `balls`, current `fps` (updated per second), `target_fps`, and `best stable balls`

Stability rule used by both runtimes:

- `no_drop` means: sampled FPS per second is at least `95%` of `target_fps`

## Run Protocol

1. Start with default ball count.
2. Increase balls with Right until FPS begins to fluctuate below the no-drop threshold.
3. Decrease/increase around that point to find the highest stable count.
4. Repeat 3 runs per runtime and record the best stable count.

## Commands

pycro benchmark:

```bash
cargo run -- examples/compare_pycro_balls_benchmark.py
```

quick pycro smoke:

```bash
PYCRO_FRAMES=3 cargo run -- examples/compare_pycro_balls_benchmark.py
```

pygame benchmark:

```bash
python3 examples/compare_pygame_balls_benchmark.py
```

run both at the same time:

```bash
./examples/run_compare_benchmarks.sh
```

if pygame is missing:

```bash
python3 -m pip install pygame
```

## Result Template

Use this table in PR notes or tracker evidence:

| Runtime | Target FPS | Best Stable Balls (run 1) | run 2 | run 3 | Final best |
| --- | --- | --- | --- | --- | --- |
| pycro | 60 |  |  |  |  |
| pygame | 60 |  |  |  |  |
