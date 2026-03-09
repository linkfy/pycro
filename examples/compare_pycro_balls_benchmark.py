import pycro

LOGICAL_WIDTH = 1280.0
LOGICAL_HEIGHT = 720.0
TARGET_FPS = 60
FPS_STABLE_RATIO = 0.95
INITIAL_BALLS = 500
MIN_BALLS = 1
MAX_BALLS = 5000
BALL_STEP = 50
KEY_REPEAT_INITIAL_DELAY = 0.18
KEY_REPEAT_INTERVAL = 0.06
MAX_SIM_DT = 1.0 / 20.0

_rng_state = 0xA5A5A5A5
_ball_x: list[float] = []
_ball_y: list[float] = []
_ball_vx: list[float] = []
_ball_vy: list[float] = []
_ball_radius: list[float] = []
_ball_color: list[pycro.Color] = []

_left_held = False
_left_repeat_in = 0.0
_right_held = False
_right_repeat_in = 0.0

_fps_sample_time = 0.0
_fps_sample_frames = 0
_display_fps = 0
_best_stable_balls = 0


def _ball_count() -> int:
    return len(_ball_x)


def _rand01() -> float:
    global _rng_state
    _rng_state = (1664525 * _rng_state + 1013904223) & 0xFFFFFFFF
    return float(_rng_state) / 4294967295.0


def _uniform(min_value: float, max_value: float) -> float:
    return min_value + ((max_value - min_value) * _rand01())


def _random_sign() -> float:
    if _rand01() < 0.5:
        return -1.0
    return 1.0


def _append_random_ball() -> None:
    radius = _uniform(4.0, 10.0)
    _ball_x.append(_uniform(radius, LOGICAL_WIDTH - radius))
    _ball_y.append(_uniform(radius, LOGICAL_HEIGHT - radius))
    _ball_vx.append(_random_sign() * _uniform(120.0, 360.0))
    _ball_vy.append(_random_sign() * _uniform(120.0, 360.0))
    _ball_radius.append(radius)
    _ball_color.append(
        (
            _uniform(0.15, 1.0),
            _uniform(0.15, 1.0),
            _uniform(0.15, 1.0),
            1.0,
        )
    )


def _remove_last_ball() -> None:
    _ball_x.pop()
    _ball_y.pop()
    _ball_vx.pop()
    _ball_vy.pop()
    _ball_radius.pop()
    _ball_color.pop()


def _set_ball_count(count: int) -> None:
    target = max(MIN_BALLS, min(MAX_BALLS, count))
    while _ball_count() < target:
        _append_random_ball()
    while _ball_count() > target:
        _remove_last_ball()


def _consume_left_repeat(is_down: bool, dt: float) -> bool:
    global _left_held, _left_repeat_in
    if not is_down:
        _left_held = False
        _left_repeat_in = 0.0
        return False
    if not _left_held:
        _left_held = True
        _left_repeat_in = KEY_REPEAT_INITIAL_DELAY
        return True
    _left_repeat_in -= dt
    if _left_repeat_in <= 0.0:
        _left_repeat_in += KEY_REPEAT_INTERVAL
        return True
    return False


def _consume_right_repeat(is_down: bool, dt: float) -> bool:
    global _right_held, _right_repeat_in
    if not is_down:
        _right_held = False
        _right_repeat_in = 0.0
        return False
    if not _right_held:
        _right_held = True
        _right_repeat_in = KEY_REPEAT_INITIAL_DELAY
        return True
    _right_repeat_in -= dt
    if _right_repeat_in <= 0.0:
        _right_repeat_in += KEY_REPEAT_INTERVAL
        return True
    return False


def setup() -> None:
    global _display_fps, _best_stable_balls
    _set_ball_count(INITIAL_BALLS)
    _display_fps = TARGET_FPS
    _best_stable_balls = INITIAL_BALLS
    print("[compare_pycro_balls_benchmark] Left/Right: adjust balls by 50 (key repeat).")


def update(dt: float) -> None:
    global _fps_sample_time, _fps_sample_frames, _display_fps, _best_stable_balls
    _ = dt

    if pycro.is_key_down("Escape"):
        raise SystemExit(0)

    frame_dt = min(pycro.frame_time(), MAX_SIM_DT)

    if _consume_left_repeat(pycro.is_key_down("Left"), frame_dt):
        _set_ball_count(_ball_count() - BALL_STEP)
    if _consume_right_repeat(pycro.is_key_down("Right"), frame_dt):
        _set_ball_count(_ball_count() + BALL_STEP)

    for i in range(_ball_count()):
        x = _ball_x[i] + (_ball_vx[i] * frame_dt)
        y = _ball_y[i] + (_ball_vy[i] * frame_dt)
        radius = _ball_radius[i]
        vx = _ball_vx[i]
        vy = _ball_vy[i]

        if x + radius >= LOGICAL_WIDTH:
            x = LOGICAL_WIDTH - radius
            vx = -abs(vx)
        elif x - radius <= 0.0:
            x = radius
            vx = abs(vx)

        if y + radius >= LOGICAL_HEIGHT:
            y = LOGICAL_HEIGHT - radius
            vy = -abs(vy)
        elif y - radius <= 0.0:
            y = radius
            vy = abs(vy)

        _ball_x[i] = x
        _ball_y[i] = y
        _ball_vx[i] = vx
        _ball_vy[i] = vy

    _fps_sample_time += frame_dt
    _fps_sample_frames += 1
    if _fps_sample_time >= 1.0:
        second_fps = float(_fps_sample_frames) / max(_fps_sample_time, 1e-6)
        _display_fps = int(round(second_fps))
        if second_fps >= float(TARGET_FPS) * FPS_STABLE_RATIO:
            _best_stable_balls = max(_best_stable_balls, _ball_count())
        _fps_sample_time = 0.0
        _fps_sample_frames = 0

    pycro.clear_background((0.05, 0.06, 0.08, 1.0))

    for i in range(_ball_count()):
        pycro.draw_circle((_ball_x[i], _ball_y[i]), _ball_radius[i], _ball_color[i])

    pycro.draw_text(
        f"balls: {_ball_count()}",
        (20.0, 32.0),
        28.0,
        (0.98, 0.98, 0.98, 1.0),
    )
    pycro.draw_text(
        f"fps: {_display_fps}",
        (20.0, 62.0),
        24.0,
        (0.82, 0.90, 1.00, 1.0),
    )
    pycro.draw_text(
        f"best stable balls: {_best_stable_balls}",
        (20.0, 90.0),
        24.0,
        (0.95, 0.84, 0.50, 1.0),
    )
    pycro.draw_text(
        f"target_fps: {TARGET_FPS}",
        (20.0, 118.0),
        24.0,
        (0.75, 0.87, 0.75, 1.0),
    )
