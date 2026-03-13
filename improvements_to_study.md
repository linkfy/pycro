# improvements_to_study

Study and research document for FPS improvements in `pycro`.

## Measurement Protocol (Current Quick Canon)

- Scenario: `examples/compare_pycro_balls_benchmark.py`
- Load: `25000` balls
- Session: `BENCHMARK_AUTO_SESSION_SECONDS=2.5`
- Runs per iteration: `2` consecutive runs
- Base command:

```bash
BENCHMARK_AUTO=1 \
BENCHMARK_AUTO_INITIAL_BALLS=25000 \
BENCHMARK_AUTO_TARGETS=25000 \
BENCHMARK_AUTO_SESSION_SECONDS=2.5 \
PYCRO_FRAMES=600 \
cargo run --release -- examples/compare_pycro_balls_benchmark.py
```

## Short Reference Baseline

- Run A: `wall_fps=23.40`
- Run B: `wall_fps=22.50`
- Initial approximate center: `~22.95`

## Improvement 1 (Positive)

- Applied technique:
  - Internal render-queue redesign: `QueuedDrawBatch` + `CircleRun` to group contiguous circles.
  - Removed per-circle operations such as expanded enum entries in the hot path.
  - Mark/rollback support (`mark/rollback`) for parse aborts without rebuilding the full queue.
- Why it improves:
  - Less structural overhead during enqueue.
  - Lower allocation pressure with thousands of `draw_circle` calls per frame.
- Measured positive result:
  - Run A: `wall_fps=25.04`
  - Run B: `wall_fps=25.37`
  - Approximate center: `~25.20`
  - Delta vs baseline: `+~2.25 FPS`.
- Risk/note:
  - Preserve exact command order to avoid visual semantic changes.
- Validation used:
  - `cargo test submit_render_matches_legacy_draw_path_order_and_payload`
  - `cargo test draw_batch_flush_clears_per_frame`
  - `cargo test submit_circle_batch_queues_expected_draw_circles`

## Improvement 2 (Positive)

- Applied technique:
  - `submit_render` cache moved to `Arc<SubmitRenderCircleCache>` to avoid full vector clone of circles every frame.
  - Fast parse of cached positions (`Vec2`) for the hot path.
  - Preserve `draw_batch` capacity during flush (clear, do not drop capacity).
- Why it improves:
  - Reduces large per-frame copies (thousands of entries).
  - Avoids repeated work rebuilding buffers.
- Measured positive result:
  - Run A: `wall_fps=25.43`
  - Run B: `wall_fps=25.09`
  - Approximate center: `~25.26`
  - Delta vs baseline: `+~2.31 FPS`.
- Risk/note:
  - Validate cache coherence by command identity to avoid false hits.
- Validation used:
  - `cargo test submit_render_matches_legacy_draw_path_order_and_payload`
  - `cargo test draw_batch_flush_clears_per_frame`
  - `cargo test submit_circle_batch_queues_expected_draw_circles`

## Improvement 3 (Slight Positive)

- Applied technique:
  - Cached `setup/update` callables in `RustPythonVm` to avoid globals lookup each frame.
  - Removed string formatting in hot numeric-parse paths (format only on real failure).
- Why it improves:
  - Cuts fixed per-frame bridge cost in VM path.
  - Reduces unnecessary allocations in successful numeric parse.
- Measured positive result:
  - Run A: `wall_fps=25.06`
  - Run B: `wall_fps=24.57`
  - Approximate center: `~24.82`
  - Still above initial baseline (`~22.95`).
- Risk/note:
  - Sensitive to short-run noise; keep multiple repetitions.
- Validation used:
  - `cargo test submit_render_matches_legacy_draw_path_order_and_payload`
  - `cargo test draw_batch_flush_clears_per_frame`
  - `cargo test submit_circle_batch_queues_expected_draw_circles`
  - `cargo test direct_bridge_returns_backend_values_for_frame_time_and_texture_handle`

## Improvement 4 (Slight Positive)

- Applied technique:
  - Compressed `submit_render` layout: circle `runs` + indexed non-circle commands.
  - Internal reorganization to traverse less metadata per frame in the hot path.
  - More direct separation between massive circle blocks and standalone HUD/background commands.
- Why it improves:
  - Reduces Python->Rust bridge iteration overhead when command list shape is stable.
  - Improves data locality with thousands of contiguous circles.
- Measured positive result:
  - Run A: `wall_fps=24.83`
  - Run B: `wall_fps=25.89`
  - Approximate center: `~25.36`
  - Delta vs previous iteration: `+~0.55 FPS` over `~24.82`.
  - Still above initial baseline (`~22.95`).
- Risk/note:
  - Verify compressed layout does not break exact ordering between circles and non-circle commands.
  - Real but small gain; keep repeated runs for stability confirmation.
- Validation used:
  - `cargo test submit_render_matches_legacy_draw_path_order_and_payload`
  - `cargo test draw_batch_flush_clears_per_frame`
  - `cargo test submit_circle_batch_queues_expected_draw_circles`
  - `cargo test direct_bridge_returns_backend_values_for_frame_time_and_texture_handle`

## Improvement 5 (Positive)

- Applied technique:
  - New backend API: `draw_circle_batch`.
  - Runtime flush dispatches full `CircleRun` batches instead of one Rust call per circle.
  - Reuses the already-grouped structure in `QueuedDrawBatch`.
- Why it improves:
  - Reduces call overhead and branching in end-of-frame flush.
  - Better matches benchmark pattern (thousands of contiguous circles compressed into runs).
- Measured positive result:
  - Run A: `wall_fps=25.86`
  - Run B: `wall_fps=26.27`
  - Approximate center: `~26.07`
  - Delta vs previous iteration (`run11=24.83`, `run12=25.89`, center `~25.36`): `+~0.71 FPS`.
  - Still above initial baseline (`~22.95`).
- Risk/note:
  - Preserve exact ordering semantics within each `CircleRun`.
  - If dispatch count is evidence, batch mode must preserve equivalent accounting.
- Validation used:
  - `cargo test submit_render_matches_legacy_draw_path_order_and_payload`
  - `cargo test draw_batch_flush_clears_per_frame`
  - `cargo test submit_circle_batch_queues_expected_draw_circles`
  - `cargo test direct_bridge_returns_backend_values_for_frame_time_and_texture_handle`
  - Re-run canonical short benchmark at `25000` balls

## Improvement 6 (Clearly Positive)

- Applied technique:
  - `Cargo.toml` tuning in `release` profile.
  - Step 1: `lto=thin` + `codegen-units=1`.
  - Step 2: on top of that, `lto=fat` + `panic=abort`.
- Why it improves:
  - `codegen-units=1` + LTO helps optimize hot paths across module boundaries.
  - `lto=fat` increases inlining and overhead elimination in final binary.
  - `panic=abort` slightly reduces release binary overhead.
- Measured positive result:
  - Step 1 (`lto=thin` + `codegen-units=1`):
    - Run A: `wall_fps=26.97`
    - Run B: `wall_fps=27.39`
    - Approximate center: `~27.18`
    - Delta vs previous iteration (`~26.07`): `+~1.12 FPS`.
  - Step 2 (`lto=fat` + `panic=abort`):
    - Run A: `wall_fps=29.22`
    - Run B: `wall_fps=29.45`
    - Approximate center: `~29.34`
    - Delta vs step 1 (`~27.18`): `+~2.16 FPS`.
    - Total delta vs pre-tuning iteration (`~26.07`): `+~3.28 FPS`.
  - Still well above initial baseline (`~22.95`).
- Risk/note:
  - Increases release build times.
  - `panic=abort` changes release failure behavior; acceptable for benchmark, but monitor operational implications if kept as permanent default.
  - Validate linking/tooling differences across platforms.
- Validation used:
  - `cargo test`
  - `cargo test submit_render_matches_legacy_draw_path_order_and_payload`
  - `cargo test draw_batch_flush_clears_per_frame`
  - `cargo test submit_circle_batch_queues_expected_draw_circles`
  - `cargo test direct_bridge_returns_backend_values_for_frame_time_and_texture_handle`
  - Re-run canonical short benchmark at `25000` balls after each release-profile change

## Improvement 7 (Clearly Positive)

- Applied technique:
  - Use `mimalloc` as process global allocator.
  - Replace default allocator under canonical short protocol (`2.5s`, `2` runs).
- Why it improves:
  - Benchmark creates heavy allocation/memory pressure in Python->Rust bridge and temporary render structures.
  - Better allocator fit reduces alloc/free overhead and improves hot-path locality.
- Measured positive result:
  - Run A: `wall_fps=31.93`
  - Run B: `wall_fps=31.86`
  - Approximate center: `~31.90`
  - Delta vs previous stable reference (`run17=29.22`, `run18=29.45`, center `~29.34`): `+~2.56 FPS`.
  - Still well above initial baseline (`~22.95`).
- Risk/note:
  - Introduces global allocator dependency; monitor compatibility and platform behavior.
  - May change memory profile and startup/shutdown times while improving throughput.
  - Keep cross-validation on real desktop and review debugging/tooling differences.
- Validation used:
  - `cargo test`
  - `cargo test submit_render_matches_legacy_draw_path_order_and_payload`
  - `cargo test draw_batch_flush_clears_per_frame`
  - `cargo test submit_circle_batch_queues_expected_draw_circles`
  - `cargo test direct_bridge_returns_backend_values_for_frame_time_and_texture_handle`
  - Re-run canonical short benchmark at `25000` balls with protocol `2.5s / 2 runs`

## Improvement 8 (Marginal Positive)

- Applied technique:
  - Specific fast path for circle positions represented as `PyList[PyFloat, PyFloat]`.
  - `Vec2` parse shortcut to avoid generic sequence path when benchmark already provides mutable two-float lists.
- Why it improves:
  - Benchmark updates positions in-place on reusable Python lists frame to frame.
  - Skipping part of generic parsing slightly reduces Python->Rust bridge overhead on the hottest repeated path.
- Measured positive result:
  - Run A: `wall_fps=31.69`
  - Run B: `wall_fps=32.17`
  - Approximate center: `~31.93`
  - Delta vs previous reference (`run21=31.93`, `run22=31.86`, center `~31.90`): `+~0.03 FPS`.
  - Gain exists but is near short-run noise.
- Risk/note:
  - Low functional risk if fast path correctly falls back to generic path.
  - Main risk is overfitting to current benchmark shape; benefit may vanish for tuples/other sequences.
  - Keep generic path intact and treat shortcut as opportunistic optimization.
- Validation used:
  - `cargo test`
  - `cargo test submit_render_matches_legacy_draw_path_order_and_payload`
  - `cargo test draw_batch_flush_clears_per_frame`
  - `cargo test submit_circle_batch_queues_expected_draw_circles`
  - `cargo test direct_bridge_returns_backend_values_for_frame_time_and_texture_handle`
  - Re-run canonical short benchmark at `25000` balls with protocol `2.5s / 2 runs`

## Improvement 9 (Strong Positive)

- Applied technique:
  - Enable `PYCRO_CIRCLE_SPRITE=1`.
  - Backend `draw_circle_batch` renders circles using a circular sprite texture instead of repeated tessellation via `draw_circle`.
  - Change is applied in batch path, not in Python benchmark logic.
- Why it improves:
  - Reduces rasterization/geometry cost per circle in backend.
  - Better fit for benchmark dominant case: thousands of circles with same base visual pattern.
  - Works especially well with grouped `CircleRun` flush.
- Measured positive result:
  - Run A: `wall_fps=37.88`
  - Run B: `wall_fps=37.30`
  - Approximate center: `~37.59`
  - Delta vs recent non-sprite reference (`run37=32.15`, `run38=31.71`, center `~31.93`): `+~5.66 FPS`.
  - Still well above initial baseline (`~22.95`).
- Risk/note:
  - Real visual risk: sprite can differ from `draw_circle` in edge quality, smoothing, alpha, gamma, or scaling appearance.
  - Review artifacts under fast motion, overlap, and extreme radii.
  - If adopted as default, validate cross-platform consistency (output may depend on graphics backend).
- Validation used:
  - `cargo test`
  - `cargo test submit_render_matches_legacy_draw_path_order_and_payload`
  - `cargo test draw_batch_flush_clears_per_frame`
  - `cargo test submit_circle_batch_queues_expected_draw_circles`
  - `cargo test direct_bridge_returns_backend_values_for_frame_time_and_texture_handle`
  - Re-run canonical short benchmark at `25000` balls with `PYCRO_CIRCLE_SPRITE=1`
  - Visual verification required to confirm sprite-vs-tessellation quality remains acceptable

## Mandatory Template For Future Positive Improvements

When an improvement increases FPS (even minimally), add a new entry with:

1. Applied technique.
2. Why it improves (mechanism).
3. Exact evidence (2 runs, `wall_fps` summary).
4. Delta against current baseline.
5. Technical risks and executed validations.

## Improvement 10 (Positive: visual quality + stable performance)

- Applied technique:
  - Diameter-based texture cache approach was discarded because it caused severe FPS regression in practice (multiplied textures and draw calls).
  - Switched to a single high-resolution circular texture for all radii, with linear scaling.
  - Edge was tuned with smoothing (soft AA) to preserve clean appearance under scaling.
- Why it improves:
  - Quality: large circles no longer appear as pixelated because source texture has more detail and linear filtering avoids hard stepping.
  - Performance: reusing one texture preserves batching better (fewer resource switches and less draw-call fragmentation) compared to per-diameter scheme.
- Positive result observed by user on screen:
  - Observed FPS: `~39 FPS`.
  - Visual improvement confirmed: large circles with smoother edges and less pixelation.
- Risks/adjustments:
  - If base sprite is too small, pixelation returns at high radii; if too large, memory usage increases.
  - Recommended tuning via `PYCRO_CIRCLE_SPRITE_SIZE` based on target visual/perf profile.
  - Keep visual verification at extreme sizes to avoid halos or excessive blur.

## Improvement 11 (Minimal Positive)

- Applied technique:
  - In `src/backend.rs`, `draw_circle_batch` removed the `all(...)` pre-scan.
  - Switched to a single pass that decides per circle whether to use sprite or vector path.
- Why it improves:
  - Avoids an extra full pass over the batch before rendering.
  - Reduces fixed overhead in the hot path while preserving sprite/vector selection semantics.
- Exact measured evidence (canonical protocol `25000 / 2.5s / 2 runs`, `PYCRO_FLUSH_STDIO_ON_UPDATE=1`, backend `opengl`, `sprite=1`):
  - Before:
    - Run A: `wall_fps=35.61`
    - Run B: `wall_fps=35.77`
    - Approximate center: `~35.69`
  - After:
    - Run A: `wall_fps=35.71`
    - Run B: `wall_fps=36.08`
    - Approximate center: `~35.90`
- Delta:
  - Average delta: `+~0.21 FPS`.
- Risk/note:
  - Small gain, close to short-run noise; revalidate with more repetitions for statistical stability.
  - Verify per-circle decision does not introduce subtle visual divergence vs previous homogeneous batch behavior.
