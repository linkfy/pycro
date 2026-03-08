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

Shared asset packs:

- `examples/assets/kenney_development_essentials/` (CC0, see `examples/assets/ASSET_PACKS.md`)

## Manual Test Checklist

1. Run each scenario and confirm the window opens without Python/runtime errors.
2. `input_movement.py`: Hold arrow keys and verify the cyan player circle moves; hold Space and confirm speed/background change.
3. `timing_frame_pulse.py`: Verify the center circle continuously pulses and the small top-left marker changes size per frame.
4. `textures_draw_swap.py`: Verify two panels render; one should use `examples/assets/pattern.png`, the missing path should render fallback white. Hold Space and verify panel roles swap.
5. `camera_target_pan.py`: Hold arrow keys and verify the world shifts as camera target moves; hold Space for faster pan.
6. `stopwatch_seconds.py`: Verify the on-screen `seconds: N` text increments once per second, plus one new timeline dot each second.
7. Press Escape to close interactive runs.

## Key Mapping Notes

Current backend key names: `Left`, `Right`, `Up`, `Down`, `Space`, `Escape`.
