import pycro

BG = (0, 0, 0, 1)
TEXT_COLOR = (1, 1, 0, 1)
DOT_COLOR = (0, 1, 1, 1)

x = 120
y = 120
step = 8


def update(dt: float) -> None:
    global x, y

    pycro.clear_background(BG)

    left_down = pycro.is_key_down(pycro.KEY.LEFT)
    right_down = pycro.is_key_down(pycro.KEY.RIGHT)
    up_down = pycro.is_key_down(pycro.KEY.UP)
    down_down = pycro.is_key_down(pycro.KEY.DOWN)

    if left_down:
        x -= step
    if right_down:
        x += step
    if up_down:
        y -= step
    if down_down:
        y += step

    # Int-based Vec2 + Color values should be coerced to float by runtime.
    pycro.draw_circle((x, y), 18.0, DOT_COLOR)
    pycro.draw_text(
        "Phase 12: Vec2/Color int->float coercion",
        (24, 34),
        26.0,
        TEXT_COLOR,
    )
    pycro.draw_text(f"x={x} y={y}", (24, 66), 22.0, TEXT_COLOR)
    pycro.draw_text(
        f"L={left_down} R={right_down} U={up_down} D={down_down}",
        (24, 94),
        20.0,
        TEXT_COLOR,
    )
