# Playable Manual-Test Scenarios

All examples are executed with:

```bash
cargo run -- examples/<file>.py
```

Quick non-interactive smoke (fixed frame budget):

```bash
PYCRO_FRAMES=3 cargo run -- examples/<file>.py
```

Available scenarios:

- `phase01_basic_main.py`: Baseline integration sample that touches the current API surface.
- `phase05_input_movement.py`: Arrow-key movement plus Space turbo/background change.
- `phase05_timing_frame_pulse.py`: Animation pulse driven by `pycro.frame_time()`.
- `phase05_textures_draw_swap.py`: `load_texture` and `draw_texture` behavior; Space swaps loaded vs fallback slots.
- `phase05_input_texture_lab.py`: phase-05 manual gate scenario combining input movement, texture switching, fallback texture path, and HUD validation.
- `phase05_camera_target_pan.py`: `set_camera_target` world panning with arrow keys; Space turbo.
- `phase05_stopwatch_seconds.py`: second-by-second stopwatch progression with timeline markers.
- `phase03_import_main.py`: main script imports `phase03_player.py` and delegates player class/movement/render logic.
- `phase04_stdlib_math_os.py`: direct stdlib imports (`math`, `os`) animate an orbit and display `os` runtime details.
- `phase04_stdlib_wave_lab.py`: direct stdlib imports (`math`, `os`) drive a controllable sine-wave field.
- `phase05_fps_eased_balls.py`: fixed-step series of balls with lateral easing; Left/Right changes effective FPS (1..120).
- `phase05_runtime_draw_flush_batch.py`: stress-style visual sample with many `draw_circle` calls per frame plus HUD counters.
- `phase05_compare_pycro_balls_benchmark.py`: bouncing-balls benchmark with per-second FPS HUD and Left/Right ball count control.
- `phase05_compare_pygame_balls_benchmark.py`: equivalent standalone pygame benchmark for runtime throughput comparison.
- `phase05_checkerboard_pan.py`: camera pan over Kenney checkerboard textures.
- `phase05_gradient_cycle.py`: Space cycles through Kenney gradient textures.
- `phase05_noise_scanner.py`: Kenney perlin and UV textures with a movable scanner marker.
- `phase05_input_texture_lab.py`: compact input + texture validation loop for phase 05 acceptance.
- `phase05_minigame_runner_dodge.py`: dodge incoming obstacle blocks over platformer deluxe art.
- `phase05_minigame_coin_chase.py`: collect coins before timer ends using platformer deluxe sprites.
- `phase05_minigame_target_burst.py`: burst moving targets with a timer/score HUD.

Shared asset packs:

- `examples/assets/kenney_development_essentials/` (CC0, see `examples/assets/ASSET_PACKS.md`)
- `examples/assets/kenney_platformer_art_deluxe/` (CC0, see `examples/assets/ASSET_PACKS.md`)

Kenney pack assets used by pack-focused scenarios:

- `phase05_checkerboard_pan.py`:
  - `examples/assets/kenney_development_essentials/Checkerboard/checkerboard.png`
  - `examples/assets/kenney_development_essentials/Checkerboard/checkerboard-transparent.png`
- `phase05_gradient_cycle.py`:
  - `examples/assets/kenney_development_essentials/Gradient/gradient-radial.png`
  - `examples/assets/kenney_development_essentials/Gradient/gradient-horizontal.png`
  - `examples/assets/kenney_development_essentials/Gradient/gradient-vertical.png`
  - `examples/assets/kenney_development_essentials/Gradient/gradient-angular.png`
- `phase05_noise_scanner.py`:
  - `examples/assets/kenney_development_essentials/Noise/perlin-noise.png`
  - `examples/assets/kenney_development_essentials/Noise/perlin-noise-small.png`
  - `examples/assets/kenney_development_essentials/UV texture/uv-texture.png`
- `phase05_input_texture_lab.py`:
  - `examples/assets/kenney_development_essentials/Checkerboard/checkerboard.png`
  - `examples/assets/kenney_development_essentials/Gradient/gradient-radial.png`

## Manual Test Checklist

1. Run each scenario and confirm the window opens without Python/runtime errors.
2. `phase05_input_movement.py`: Hold arrow keys and verify the cyan player circle moves; hold Space and confirm speed/background change.
3. `phase05_timing_frame_pulse.py`: Verify the center circle continuously pulses and the small top-left marker changes size per frame.
4. `phase05_textures_draw_swap.py`: Verify two panels render; one should use the Kenney radial gradient texture, the missing path should render fallback white. Hold Space and verify panel roles swap.
5. `phase05_camera_target_pan.py`: Hold arrow keys and verify the world shifts as camera target moves; hold Space for faster pan.
6. `phase05_input_texture_lab.py`: Left/Right should move the sprite, Up/Down should change sprite size, and holding Space should rotate texture source between two loaded textures and one fallback slot without runtime errors.
7. `phase05_stopwatch_seconds.py`: Verify the on-screen `seconds: N` text increments once per second, plus one new timeline dot each second.
8. `phase03_import_main.py`: Verify `phase03_player.py` import succeeds, arrow keys move the player, and the name label is rendered from the imported module.
9. `phase04_stdlib_math_os.py`: Verify script imports `math` and `os` without helper modules, a dot orbits the center, and on-screen text shows `os.name`, `os.sep`, plus cwd/home info; hold Space to reset the orbit timer.
10. `phase04_stdlib_wave_lab.py`: Verify direct `math`/`os` imports work, wave points animate, Left/Right changes frequency, Up/Down changes amplitude, and Space resets phase.
11. `phase05_fps_eased_balls.py`: Verify multiple balls move left/right and visibly ease near both side bounds; hold Left/Right to change on-screen `effective_fps` value within 1..120 and confirm `current_frame_fps` is rendered.
12. `phase05_runtime_draw_flush_batch.py`: Verify the grid and sweep markers animate every frame and HUD reports changing `frame` with stable high `draw_calls`.
13. `phase05_compare_pycro_balls_benchmark.py`: Hold Left/Right to decrease/increase balls by 50 with controlled repeat; confirm HUD updates `balls`, per-second `fps`, `best stable balls`, and `target_fps`.
14. `phase05_checkerboard_pan.py`: Hold Left/Right and verify camera pan marker shifts; press Space to recenter.
15. `phase05_gradient_cycle.py`: Tap Space and verify the rendered panel cycles through 4 Kenney gradients.
16. `phase05_noise_scanner.py`: Verify the three texture panels render (perlin large, perlin small, uv texture); Up/Down moves scanner marker; Space recenters.
17. `phase05_minigame_runner_dodge.py`: Dodge incoming blocks; after collision press Space to restart.
18. `phase05_minigame_coin_chase.py`: Collect coins with arrow keys and verify score/time HUD updates.
19. `phase05_minigame_target_burst.py`: Move cursor with arrows, press Space to hit targets and confirm score/timer changes.
20. Press Escape to close interactive runs.

## Key Mapping Notes

Current backend key names: `Left`, `Right`, `Up`, `Down`, `Space`, `Escape`.

## Compare Benchmark Notes

The isolated compare benchmark process lives in `examples/compare/README.md` and tracks equivalent pycro vs pygame throughput measurements.

- pycro command: `cargo run -- examples/phase05_compare_pycro_balls_benchmark.py`
- pygame command: `python3 examples/phase05_compare_pygame_balls_benchmark.py`
- both in parallel: `./examples/run_compare_benchmarks.sh`
