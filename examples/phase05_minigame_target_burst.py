"""Target Burst mini-game.

Controls:
- Arrow keys: move crosshair cursor
- Space: hit action during round, restart after round ends
"""

import pycro

ROUND_SECONDS = 30.0
CURSOR_SPEED = 420.0
PLAYFIELD_MIN_X = 60.0
PLAYFIELD_MAX_X = 1220.0
PLAYFIELD_MIN_Y = 90.0
PLAYFIELD_MAX_Y = 670.0

PANEL_TEXTURES = (
    "examples/assets/kenney_platformer_art_deluxe/Mushroom expansion/Backgrounds/bg_castle.png",
    "examples/assets/kenney_platformer_art_deluxe/Mushroom expansion/Backgrounds/bg_desert.png",
    "examples/assets/kenney_platformer_art_deluxe/Mushroom expansion/Backgrounds/bg_grasslands.png",
)

TARGET_COLORS = (
    (0.98, 0.28, 0.18, 0.95),
    (0.95, 0.62, 0.10, 0.95),
    (0.33, 0.82, 0.36, 0.95),
    (0.24, 0.68, 0.98, 0.95),
)

background_handles: list[pycro.TextureHandle] = []
initialized = False

cursor_x = 640.0
cursor_y = 360.0

score = 0
high_score = 0
remaining_seconds = ROUND_SECONDS
round_over = False

space_was_down = False

# Each target: x, y, vx, vy, radius, color_index
targets: list[list[float]] = []


def _clamp(value: float, low: float, high: float) -> float:
    return max(low, min(high, value))


def _seed_targets() -> None:
    global targets
    targets = [
        [220.0, 160.0, 140.0, 170.0, 22.0, 0.0],
        [470.0, 250.0, -130.0, 155.0, 26.0, 1.0],
        [780.0, 350.0, 165.0, -140.0, 24.0, 2.0],
        [1020.0, 520.0, -150.0, -165.0, 28.0, 3.0],
        [620.0, 560.0, 180.0, 130.0, 20.0, 0.0],
    ]


def _restart_round() -> None:
    global score, remaining_seconds, round_over
    score = 0
    remaining_seconds = ROUND_SECONDS
    round_over = False
    _seed_targets()


def _ensure_initialized() -> None:
    global initialized, background_handles, cursor_x, cursor_y, high_score
    if initialized:
        return
    high_score = 0
    background_handles = [pycro.load_texture(path) for path in PANEL_TEXTURES]
    cursor_x = 640.0
    cursor_y = 360.0
    _restart_round()
    initialized = True


def _move_cursor(dt: float) -> None:
    global cursor_x, cursor_y
    dx = 0.0
    dy = 0.0

    if pycro.is_key_down("Left"):
        dx -= 1.0
    if pycro.is_key_down("Right"):
        dx += 1.0
    if pycro.is_key_down("Up"):
        dy -= 1.0
    if pycro.is_key_down("Down"):
        dy += 1.0

    cursor_x = _clamp(cursor_x + dx * CURSOR_SPEED * dt, PLAYFIELD_MIN_X, PLAYFIELD_MAX_X)
    cursor_y = _clamp(cursor_y + dy * CURSOR_SPEED * dt, PLAYFIELD_MIN_Y, PLAYFIELD_MAX_Y)


def _tick_targets(dt: float) -> None:
    for target in targets:
        x = target[0] + target[2] * dt
        y = target[1] + target[3] * dt
        radius = target[4]

        if x - radius < PLAYFIELD_MIN_X:
            x = PLAYFIELD_MIN_X + radius
            target[2] *= -1.0
        if x + radius > PLAYFIELD_MAX_X:
            x = PLAYFIELD_MAX_X - radius
            target[2] *= -1.0
        if y - radius < PLAYFIELD_MIN_Y:
            y = PLAYFIELD_MIN_Y + radius
            target[3] *= -1.0
        if y + radius > PLAYFIELD_MAX_Y:
            y = PLAYFIELD_MAX_Y - radius
            target[3] *= -1.0

        target[0] = x
        target[1] = y


def _handle_space_action(space_pressed: bool) -> None:
    global score, high_score, remaining_seconds, round_over

    if round_over:
        _restart_round()
        return

    best_hit_index = -1
    best_hit_dist_sq = 1e20
    for i, target in enumerate(targets):
        dx = cursor_x - target[0]
        dy = cursor_y - target[1]
        dist_sq = dx * dx + dy * dy
        radius = target[4]
        if dist_sq <= radius * radius and dist_sq < best_hit_dist_sq:
            best_hit_dist_sq = dist_sq
            best_hit_index = i

    if best_hit_index >= 0:
        score += 1
        if score > high_score:
            high_score = score
        hit = targets[best_hit_index]
        hit[0] = PLAYFIELD_MIN_X + 80.0 + ((score * 137.0) % (PLAYFIELD_MAX_X - PLAYFIELD_MIN_X - 160.0))
        hit[1] = PLAYFIELD_MIN_Y + 70.0 + ((score * 89.0) % (PLAYFIELD_MAX_Y - PLAYFIELD_MIN_Y - 140.0))
        hit[2] = -hit[2]
        hit[3] = -hit[3]
        hit[5] = float((int(hit[5]) + 1) % len(TARGET_COLORS))
        remaining_seconds = min(ROUND_SECONDS, remaining_seconds + 0.55)
    else:
        remaining_seconds = max(0.0, remaining_seconds - 0.75)

    if remaining_seconds <= 0.0:
        round_over = True


def _draw_backdrop() -> None:
    pycro.clear_background((0.05, 0.07, 0.10, 1.0))

    panel_w = 396.0
    panel_h = 224.0
    start_x = 64.0
    y_top = 100.0
    y_bottom = 404.0

    for row in (y_top, y_bottom):
        for col in range(3):
            tex = background_handles[col % len(background_handles)]
            x = start_x + col * (panel_w + 24.0)
            pycro.draw_texture(tex, (x, row), (panel_w, panel_h))


def _draw_targets() -> None:
    for target in targets:
        color = TARGET_COLORS[int(target[5]) % len(TARGET_COLORS)]
        pycro.draw_circle((target[0], target[1]), target[4], color)
        pycro.draw_circle((target[0], target[1]), target[4] * 0.42, (1.0, 1.0, 1.0, 0.75))


def _draw_cursor() -> None:
    pycro.draw_circle((cursor_x, cursor_y), 18.0, (1.0, 1.0, 1.0, 0.14))
    pycro.draw_circle((cursor_x, cursor_y), 12.0, (1.0, 1.0, 1.0, 0.20))
    pycro.draw_circle((cursor_x, cursor_y), 3.5, (1.0, 0.92, 0.30, 1.0))


def _draw_hud() -> None:
    pycro.draw_text(f"Score {score}", (28.0, 42.0), 34.0, (1.0, 1.0, 1.0, 1.0))
    pycro.draw_text(
        f"Time {remaining_seconds:04.1f}",
        (28.0, 76.0),
        30.0,
        (0.90, 0.96, 1.0, 1.0),
    )
    pycro.draw_text(
        f"Best {high_score}",
        (28.0, 108.0),
        24.0,
        (0.74, 0.90, 1.0, 1.0),
    )
    pycro.draw_text(
        "Arrows move cursor  Space: hit / restart",
        (28.0, 702.0),
        24.0,
        (0.95, 0.95, 0.95, 1.0),
    )

    if round_over:
        pycro.draw_text("ROUND OVER", (486.0, 334.0), 52.0, (1.0, 0.88, 0.30, 1.0))
        pycro.draw_text(
            "Press Space to restart",
            (476.0, 374.0),
            32.0,
            (1.0, 1.0, 1.0, 1.0),
        )


def update(dt: float) -> None:
    global remaining_seconds, round_over, space_was_down
    _ensure_initialized()

    _move_cursor(dt)

    if not round_over:
        _tick_targets(dt)
        remaining_seconds = max(0.0, remaining_seconds - dt)
        if remaining_seconds <= 0.0:
            round_over = True

    space_down = pycro.is_key_down("Space")
    space_pressed = space_down and not space_was_down
    if space_pressed:
        _handle_space_action(space_pressed)
    space_was_down = space_down

    _draw_backdrop()
    _draw_targets()
    _draw_cursor()
    _draw_hud()
