import pycro

BG_IDLE = (0.05, 0.06, 0.09, 1.0)
BG_ACTIVE = (0.10, 0.06, 0.08, 1.0)
TEXT = (0.93, 0.95, 0.98, 1.0)
OK = (0.20, 0.82, 0.36, 1.0)
BAD = (0.86, 0.24, 0.26, 1.0)
CURSOR = (0.24, 0.82, 0.95, 1.0)

CURSOR_RADIUS = 14.0
CURSOR_SPEED = 260.0
cursor_x = 400.0
cursor_y = 250.0


def update(dt: float) -> None:
    global cursor_x, cursor_y

    left_down = pycro.is_key_down(pycro.KEY.LEFT)
    right_down = pycro.is_key_down(pycro.KEY.RIGHT)
    up_down = pycro.is_key_down(pycro.KEY.UP)
    down_down = pycro.is_key_down(pycro.KEY.DOWN)
    space_down = pycro.is_key_down(pycro.KEY.SPACE)
    escape_down = pycro.is_key_down(pycro.KEY.ESCAPE)

    if left_down:
        cursor_x -= CURSOR_SPEED * dt
    if right_down:
        cursor_x += CURSOR_SPEED * dt
    if up_down:
        cursor_y -= CURSOR_SPEED * dt
    if down_down:
        cursor_y += CURSOR_SPEED * dt

    pycro.clear_background(BG_ACTIVE if space_down else BG_IDLE)
    pycro.draw_circle((cursor_x, cursor_y), CURSOR_RADIUS, CURSOR)

    pycro.draw_text("Windows Input Diagnostic", (20.0, 32.0), 30.0, TEXT)
    pycro.draw_text("Focus the window, then hold each key.", (20.0, 60.0), 22.0, TEXT)
    pycro.draw_text("Expected: pressed key shows True immediately.", (20.0, 86.0), 20.0, TEXT)

    pycro.draw_text(f"LEFT:   {left_down}", (20.0, 132.0), 24.0, OK if left_down else BAD)
    pycro.draw_text(f"RIGHT:  {right_down}", (20.0, 160.0), 24.0, OK if right_down else BAD)
    pycro.draw_text(f"UP:     {up_down}", (20.0, 188.0), 24.0, OK if up_down else BAD)
    pycro.draw_text(f"DOWN:   {down_down}", (20.0, 216.0), 24.0, OK if down_down else BAD)
    pycro.draw_text(f"SPACE:  {space_down}", (20.0, 244.0), 24.0, OK if space_down else BAD)
    pycro.draw_text(f"ESCAPE: {escape_down}", (20.0, 272.0), 24.0, OK if escape_down else BAD)

    pycro.draw_text(
        "Arrow keys should move the cyan dot while held.",
        (20.0, 316.0),
        20.0,
        TEXT,
    )
