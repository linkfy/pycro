import time

import pycro


TITLE = "Click Diagnostic: LEFT"
BUTTON = pycro.KEY.MOUSE_LEFT


_start = time.perf_counter()
_last_down = False
_press_count = 0
_release_count = 0
_last_event_s = 0.0


def update(dt: float) -> None:
    global _last_down, _press_count, _release_count, _last_event_s

    down = pycro.is_key_down(BUTTON)
    now = time.perf_counter() - _start

    if down and not _last_down:
        _press_count += 1
        _last_event_s = now
        print(f"[{TITLE}] press #{_press_count} at t={now:.3f}s")

    if (not down) and _last_down:
        _release_count += 1
        _last_event_s = now
        print(f"[{TITLE}] release #{_release_count} at t={now:.3f}s")

    _last_down = down

    mouse_x, mouse_y = pycro.get_mouse_position()

    pycro.clear_background((0.05, 0.06, 0.08, 1.0))
    pycro.draw_text(TITLE, (24.0, 36.0), 26.0, (0.95, 0.95, 0.95, 1.0))
    pycro.draw_text(
        f"down={down}  press={_press_count}  release={_release_count}",
        (24.0, 74.0),
        20.0,
        (0.75, 0.85, 1.0, 1.0),
    )
    pycro.draw_text(
        f"last_event_t={_last_event_s:.3f}s  dt={dt:.4f}s",
        (24.0, 102.0),
        18.0,
        (0.75, 0.85, 1.0, 1.0),
    )
    pycro.draw_text(
        f"mouse=({mouse_x:.1f}, {mouse_y:.1f})",
        (24.0, 128.0),
        18.0,
        (0.75, 0.85, 1.0, 1.0),
    )

    # small crosshair at mouse position
    pycro.put_pixel(mouse_x, mouse_y, (1.0, 0.2, 0.2, 1.0))
    pycro.draw_line(mouse_x - 6.0, mouse_y, mouse_x + 6.0, mouse_y, (1.0, 0.2, 0.2, 1.0))
    pycro.draw_line(mouse_x, mouse_y - 6.0, mouse_x, mouse_y + 6.0, (1.0, 0.2, 0.2, 1.0))

