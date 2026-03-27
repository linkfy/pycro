import math
import pycro

_t = 0.0

def update(dt: float) -> None:
    global _t
    _t += dt

    width, height = pycro.get_window_size()
    pulse = 0.5 + 0.5 * math.sin(_t * 1.5)
    rect_w = width * (0.2 + 0.1 * pulse)
    rect_h = height * (0.15 + 0.08 * pulse)
    rect_x = (width - rect_w) * 0.5
    rect_y = (height - rect_h) * 0.5

    pycro.clear_background((0.06, 0.08, 0.12, 1.0))
    pycro.draw_rectangle(rect_x, rect_y, rect_w, rect_h, (0.22, 0.74, 0.98, 0.92))
    pycro.draw_text(
        f"window={int(width)}x{int(height)} rect={int(rect_w)}x{int(rect_h)}",
        (24.0, 36.0),
        28.0,
        (0.95, 0.97, 1.0, 1.0),
    )
