# Platform Capability Matrix

Platform parity is enforced by declared capability support and validation evidence.

| Capability | Desktop | Web | Android | iOS | Notes |
| --- | --- | --- | --- | --- | --- |
| Frame loop ownership | required | required | required | required | Macroquad remains the loop owner on every target |
| Render primitives | planned | planned | planned | planned | Public API kept cross-platform-safe |
| Input | planned | planned | planned | planned | Target-specific quirks must be hidden behind the common API |
| Timing | planned | planned | planned | planned | `update(dt)` semantics remain identical |
| Textures/assets | planned | planned | planned | planned | Asset loading errors must surface consistently |
| Camera | planned | planned | planned | planned | Capability gating required if parity changes |

## Validation Expectations

- Desktop: mandatory runtime smoke
- Web: mandatory build plus scripted smoke
- Android/iOS: mandatory build or package validation and canonical example evidence until automated runtime smoke exists

## Phase-05 Evidence Snapshot (Desktop)

- Input mapping guard:
  - backend key mapping tests cover `Left/Right/Up/Down/Space/Escape` aliases and unknown-key rejection.
- Runtime failure safety guard:
  - runtime tests verify queued draw batch is discarded after update failure, including frames with timing and texture direct-return calls.
- Texture fallback manual scenario:
  - `examples/phase05_input_texture_lab.py` shows explicit HUD state for loaded vs fallback texture path.
  - Manual check expects green marker when loaded texture is active and red marker when fallback slot is active.

## Future Improvement Notes (Backend Selection)

- Introduce explicit backend selection policy per platform/runtime (for example OpenGL vs Metal on macOS) with a stable override mechanism (`env` + config contract).
- Add capability evidence that compares backend behavior on the same scenario set:
  - frame pacing consistency,
  - visual artifact notes at high motion speed,
  - input-to-frame response stability.
- Define a deterministic validation matrix for backend choice:
  - default backend per platform/architecture,
  - fallback backend if the preferred one is unavailable,
  - documented known issues and waivers.
- If backend default policy changes, require an ADR and a tracker/state phase note with before/after evidence.
