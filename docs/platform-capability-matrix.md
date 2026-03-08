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

