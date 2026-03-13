import pycro

SCREEN_W = 960.0
SCREEN_H = 540.0
PLAYER_W = 54.0
PLAYER_H = 54.0
OBSTACLE_W = 72.0
OBSTACLE_H = 72.0
PLAYER_SPEED = 280.0
OBSTACLE_SPEED = 260.0
FIXED_DT = 1.0 / 60.0
MAX_STEPS = 5

BG_PATH = "examples/assets/kenney_platformer_art_deluxe/Base pack/bg.png"
SAMPLE_PATH = "examples/assets/kenney_platformer_art_deluxe/Request pack/sample.png"
SHEET_PATH = "examples/assets/kenney_platformer_art_deluxe/Request pack/sheet.png"

bg_tex = ""
sample_tex = ""
sheet_tex = ""
player_x = 120.0
player_y = SCREEN_H * 0.5
obstacles: list[tuple[float, float]] = []
spawn_clock = 0.0
pattern_index = 0
survival_time = 0.0
accumulator = 0.0
failed = False

# Deterministic lane and cadence pattern.
SPAWN_PATTERN_Y = [90.0, 170.0, 250.0, 330.0, 410.0, 250.0, 170.0, 330.0]
SPAWN_PATTERN_DT = [0.85, 0.62, 0.72, 0.55, 0.80, 0.58, 0.68, 0.60]


def setup() -> None:
    global bg_tex, sample_tex, sheet_tex
    bg_tex = pycro.load_texture(BG_PATH)
    sample_tex = pycro.load_texture(SAMPLE_PATH)
    sheet_tex = pycro.load_texture(SHEET_PATH)
    reset_game()


def reset_game() -> None:
    global player_x, player_y, obstacles, spawn_clock, pattern_index, survival_time, accumulator, failed
    player_x = 120.0
    player_y = SCREEN_H * 0.5
    obstacles = []
    spawn_clock = 0.0
    pattern_index = 0
    survival_time = 0.0
    accumulator = 0.0
    failed = False


def update(dt: float) -> None:
    global accumulator
    _ = dt

    pycro.clear_background((0.05, 0.06, 0.1, 1.0))
    pycro.draw_texture(bg_tex, (0.0, 0.0), (SCREEN_W, SCREEN_H))
    pycro.draw_texture(sample_tex, (0.0, SCREEN_H - 180.0), (SCREEN_W, 180.0))

    if failed:
        draw_world()
        pycro.draw_text("Game Over", (360.0, 230.0), 44.0, (1.0, 0.3, 0.3, 1.0))
        pycro.draw_text("Press Space to restart", (320.0, 275.0), 30.0, (1.0, 1.0, 1.0, 1.0))
        if pycro.is_key_down("Space"):
            reset_game()
        return

    frame = pycro.frame_time()
    if frame < 0.0:
        frame = 0.0
    if frame > 0.05:
        frame = 0.05
    accumulator += frame

    steps = 0
    while accumulator >= FIXED_DT and steps < MAX_STEPS:
        simulate(FIXED_DT)
        accumulator -= FIXED_DT
        steps += 1

    draw_world()
    pycro.draw_text(f"Time {survival_time:.2f}", (20.0, 40.0), 32.0, (1.0, 1.0, 1.0, 1.0))
    pycro.draw_text("Arrows move | Dodge incoming blocks", (20.0, 76.0), 24.0, (0.9, 0.95, 1.0, 1.0))


def simulate(step: float) -> None:
    global player_x, player_y, spawn_clock, pattern_index, survival_time, failed, obstacles

    dx = 0.0
    dy = 0.0
    if pycro.is_key_down("Left"):
        dx -= PLAYER_SPEED * step
    if pycro.is_key_down("Right"):
        dx += PLAYER_SPEED * step
    if pycro.is_key_down("Up"):
        dy -= PLAYER_SPEED * step
    if pycro.is_key_down("Down"):
        dy += PLAYER_SPEED * step
    player_x = clamp(player_x + dx, 20.0, SCREEN_W - PLAYER_W - 20.0)
    player_y = clamp(player_y + dy, 20.0, SCREEN_H - PLAYER_H - 20.0)

    spawn_clock += step
    next_spawn = SPAWN_PATTERN_DT[pattern_index % len(SPAWN_PATTERN_DT)]
    if spawn_clock >= next_spawn:
        spawn_clock -= next_spawn
        y = SPAWN_PATTERN_Y[pattern_index % len(SPAWN_PATTERN_Y)]
        obstacles.append((SCREEN_W + 40.0, y))
        pattern_index += 1

    moved: list[tuple[float, float]] = []
    for ox, oy in obstacles:
        nx = ox - (OBSTACLE_SPEED * step)
        if nx > -OBSTACLE_W:
            moved.append((nx, oy))
    obstacles = moved

    for ox, oy in obstacles:
        if hit(player_x, player_y, PLAYER_W, PLAYER_H, ox, oy, OBSTACLE_W, OBSTACLE_H):
            failed = True
            return

    survival_time += step


def draw_world() -> None:
    pycro.draw_texture(sheet_tex, (player_x, player_y), (PLAYER_W, PLAYER_H))
    for ox, oy in obstacles:
        pycro.draw_texture(sample_tex, (ox, oy), (OBSTACLE_W, OBSTACLE_H))


def hit(ax: float, ay: float, aw: float, ah: float, bx: float, by: float, bw: float, bh: float) -> bool:
    return ax < bx + bw and ax + aw > bx and ay < by + bh and ay + ah > by


def clamp(value: float, lo: float, hi: float) -> float:
    if value < lo:
        return lo
    if value > hi:
        return hi
    return value
