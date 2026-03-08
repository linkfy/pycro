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

- `basic_main.py`: Baseline integration sample that touches the current API surface.
- `input_movement.py`: Arrow-key movement plus Space turbo/background change.
- `timing_frame_pulse.py`: Animation pulse driven by `pycro.frame_time()`.
- `textures_draw_swap.py`: `load_texture` and `draw_texture` behavior; Space swaps loaded vs fallback slots.
- `camera_target_pan.py`: `set_camera_target` world panning with arrow keys; Space turbo.
- `stopwatch_seconds.py`: second-by-second stopwatch progression with timeline markers.
- `pack_checkerboard_pan.py`: camera pan over Kenney checkerboard textures.
- `pack_gradient_cycle.py`: Space cycles through Kenney gradient textures.
- `pack_noise_scanner.py`: Kenney perlin and UV textures with a movable scanner marker.

Shared asset packs:

- `examples/assets/kenney_development_essentials/` (CC0, see `examples/assets/ASSET_PACKS.md`)

Kenney pack assets used by pack-focused scenarios:

- `pack_checkerboard_pan.py`:
  - `examples/assets/kenney_development_essentials/Checkerboard/checkerboard.png`
  - `examples/assets/kenney_development_essentials/Checkerboard/checkerboard-transparent.png`
- `pack_gradient_cycle.py`:
  - `examples/assets/kenney_development_essentials/Gradient/gradient-radial.png`
  - `examples/assets/kenney_development_essentials/Gradient/gradient-horizontal.png`
  - `examples/assets/kenney_development_essentials/Gradient/gradient-vertical.png`
  - `examples/assets/kenney_development_essentials/Gradient/gradient-angular.png`
- `pack_noise_scanner.py`:
  - `examples/assets/kenney_development_essentials/Noise/perlin-noise.png`
  - `examples/assets/kenney_development_essentials/Noise/perlin-noise-small.png`
  - `examples/assets/kenney_development_essentials/UV texture/uv-texture.png`

## Manual Test Checklist

1. Run each scenario and confirm the window opens without Python/runtime errors.
2. `input_movement.py`: Hold arrow keys and verify the cyan player circle moves; hold Space and confirm speed/background change.
3. `timing_frame_pulse.py`: Verify the center circle continuously pulses and the small top-left marker changes size per frame.
4. `textures_draw_swap.py`: Verify two panels render; one should use the Kenney radial gradient texture, the missing path should render fallback white. Hold Space and verify panel roles swap.
5. `camera_target_pan.py`: Hold arrow keys and verify the world shifts as camera target moves; hold Space for faster pan.
6. `stopwatch_seconds.py`: Verify the on-screen `seconds: N` text increments once per second, plus one new timeline dot each second.
7. `pack_checkerboard_pan.py`: Hold Left/Right and verify camera pan marker shifts; press Space to recenter.
8. `pack_gradient_cycle.py`: Tap Space and verify the rendered panel cycles through 4 Kenney gradients.
9. `pack_noise_scanner.py`: Verify the three texture panels render (perlin large, perlin small, uv texture); Up/Down moves scanner marker; Space recenters.
10. Press Escape to close interactive runs.

## Key Mapping Notes

Current backend key names: `Left`, `Right`, `Up`, `Down`, `Space`, `Escape`.
