import pycro

from phase15_embedded_payload_support import GREETING, next_tint

BG_COLOR = (0.05, 0.06, 0.08, 1.0)
TEXTURE_PATH = "examples/assets/pattern.png"

texture = None
elapsed = 0.0
frames = 0


def update(dt: float) -> None:
    global texture, elapsed, frames
    elapsed += dt
    frames += 1

    if texture is None:
        texture = pycro.load_texture(TEXTURE_PATH)

    pycro.clear_background(BG_COLOR)
    pycro.draw_texture(texture, (24.0, 96.0), (128.0, 128.0))
    pycro.draw_text(GREETING, (24.0, 52.0), 28.0, next_tint(elapsed))
    pycro.draw_text(f"frames: {frames}", (24.0, 250.0), 22.0, (0.9, 0.93, 0.98, 1.0))
    pycro.draw_text(f"dt: {dt:.4f}", (24.0, 280.0), 22.0, (0.75, 0.87, 0.96, 1.0))
