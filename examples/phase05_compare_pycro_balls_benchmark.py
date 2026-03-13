import os

import pycro

LOGICAL_WIDTH = 1280.0
LOGICAL_HEIGHT = 720.0
TARGET_FPS = 60
FPS_STABLE_RATIO = 0.95
INITIAL_BALLS = 25000
MIN_BALLS = 1
MAX_BALLS = 56000
BALL_STEP = 500
KEY_REPEAT_INITIAL_DELAY = 0.18
KEY_REPEAT_INTERVAL = 0.06
MAX_SIM_DT = 1.0 / 20.0
HUD_REFRESH_SECONDS = 0.20

AUTO_ENABLED = os.getenv("BENCHMARK_AUTO", "0") == "1"
AUTO_INITIAL_BALLS = int(os.getenv("BENCHMARK_AUTO_INITIAL_BALLS", "25000"))
AUTO_TARGETS = tuple(
    int(token.strip())
    for token in os.getenv("BENCHMARK_AUTO_TARGETS", "25000").split(",")
    if token.strip()
)
AUTO_STEP_INTERVAL_SECONDS = float(os.getenv("BENCHMARK_AUTO_STEP_INTERVAL", "0.08"))
AUTO_HOLD_SECONDS = float(os.getenv("BENCHMARK_AUTO_HOLD_SECONDS", "2.5"))
AUTO_SESSION_SECONDS = float(os.getenv("BENCHMARK_AUTO_SESSION_SECONDS", "2.5"))

RUNTIME_NAME = "pycro"
STABLE_FPS_THRESHOLD = float(TARGET_FPS) * FPS_STABLE_RATIO

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

_wall_sample_time = 0.0
_sim_sample_time = 0.0
_sample_frames = 0
_display_fps = 0
_best_stable_balls = 0
_sample_index = 0
_elapsed_wall_seconds = 0.0
_stable_sample_seconds = 0

_nearest_count_for_target: dict[int, int] = {}
_nearest_summary_cache = ""

_auto_stage_index = 0
_auto_hold_remaining = 0.0
_auto_step_timer = 0.0
_summary_emitted = False

_hud_refresh_in = 0.0
_hud_lines: list[str] = []
_session_initial_balls = INITIAL_BALLS
_clear_render_command: list[object] = ["clear_background", (0.05, 0.06, 0.08, 1.0)]
_circle_render_commands: list[list[object]] = []
_hud_render_commands: list[list[object]] = [
    ["draw_circle", (240.0, 98.0), 205.0, (0.0, 0.0, 0.0, 0.82)],
    ["draw_circle", (340.0, 160.0), 150.0, (0.0, 0.0, 0.0, 0.82)],
    ["draw_text", "", (20.0, 32.0), 28.0, (0.98, 0.98, 0.98, 1.0)],
    ["draw_text", "", (20.0, 62.0), 24.0, (0.82, 0.90, 1.00, 1.0)],
    ["draw_text", "", (20.0, 90.0), 24.0, (0.95, 0.84, 0.50, 1.0)],
    ["draw_text", "", (20.0, 118.0), 24.0, (0.85, 0.92, 0.62, 1.0)],
    ["draw_text", "", (20.0, 146.0), 22.0, (0.95, 0.84, 0.50, 1.0)],
    ["draw_text", "", (20.0, 172.0), 22.0, (0.75, 0.87, 0.75, 1.0)],
    ["draw_text", "", (20.0, 198.0), 22.0, (0.75, 0.87, 0.75, 1.0)],
]
_render_commands: list[list[object]] = []


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


def _refresh_nearest_summary() -> None:
    global _nearest_summary_cache
    current = _ball_count()
    parts: list[str] = []
    for target in AUTO_TARGETS:
        reached = _nearest_count_for_target.get(target, current)
        delta = abs(reached - target)
        parts.append(f"{target}:{reached}(delta={delta})")
    _nearest_summary_cache = ",".join(parts)


def _set_ball_count(count: int) -> None:
    target = max(MIN_BALLS, min(MAX_BALLS, count))
    while _ball_count() < target:
        _append_random_ball()
    while _ball_count() > target:
        _remove_last_ball()
    _sync_circle_render_commands_for_ball_count()
    _rebuild_render_commands()


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


def _log_event(event: str, **fields: object) -> None:
    details = " ".join(f"{key}={value}" for key, value in fields.items())
    print(f"[benchmark] runtime={RUNTIME_NAME} event={event} {details}".rstrip())


def _record_nearest_targets() -> None:
    current = _ball_count()
    changed = False
    for target in AUTO_TARGETS:
        previous = _nearest_count_for_target.get(target)
        if previous is None or abs(current - target) < abs(previous - target):
            _nearest_count_for_target[target] = current
            changed = True
    if changed:
        _refresh_nearest_summary()


def _auto_drive(sim_dt: float) -> None:
    global _auto_stage_index, _auto_hold_remaining, _auto_step_timer
    if _summary_emitted:
        return
    if not AUTO_ENABLED or not AUTO_TARGETS:
        return
    if _auto_stage_index >= len(AUTO_TARGETS):
        return

    target = AUTO_TARGETS[_auto_stage_index]
    current = _ball_count()
    if current == target:
        if _auto_hold_remaining <= 0.0:
            _auto_hold_remaining = AUTO_HOLD_SECONDS
            _log_event("auto_target_reached", target=target, balls=current)
        _auto_hold_remaining -= sim_dt
        if _auto_hold_remaining <= 0.0:
            _auto_stage_index += 1
        return

    _auto_step_timer -= sim_dt
    if _auto_step_timer > 0.0:
        return

    _auto_step_timer = AUTO_STEP_INTERVAL_SECONDS
    if current < target:
        _set_ball_count(current + BALL_STEP)
    else:
        _set_ball_count(current - BALL_STEP)


def _refresh_hud_lines() -> None:
    status = "stable" if _display_fps >= int(round(STABLE_FPS_THRESHOLD)) else "unstable"
    _hud_lines.clear()
    _hud_lines.append(f"balls: {_ball_count()}")
    _hud_lines.append(f"fps: {_display_fps}")
    _hud_lines.append(f"status: {status}")
    _hud_lines.append(f"best stable balls: {_best_stable_balls}")
    _hud_lines.append(f"stable seconds: {_stable_sample_seconds}")
    _hud_lines.append(f"targets nearest: {_nearest_summary_cache}")
    _sync_hud_render_commands()


def _emit_summary(reason: str) -> None:
    global _summary_emitted
    if _summary_emitted:
        return
    _summary_emitted = True
    wall_fps = float(_sample_frames) / max(_wall_sample_time, 1e-6)
    sim_fps = float(_sample_frames) / max(_sim_sample_time, 1e-6)
    _log_event(
        "summary",
        reason=reason,
        best_stable_balls=_best_stable_balls,
        stable_sample_seconds=_stable_sample_seconds,
        nearest_targets=_nearest_summary_cache,
        elapsed=f"{_elapsed_wall_seconds:.2f}",
        samples=_sample_index,
        wall_fps=f"{wall_fps:.2f}",
        sim_fps=f"{sim_fps:.2f}",
        sim_dt_cap=f"{MAX_SIM_DT:.4f}",
    )


def _sync_circle_render_commands_for_ball_count() -> None:
    target = _ball_count()
    while len(_circle_render_commands) < target:
        i = len(_circle_render_commands)
        _circle_render_commands.append(
            ["draw_circle", [_ball_x[i], _ball_y[i]], _ball_radius[i], _ball_color[i]]
        )
    while len(_circle_render_commands) > target:
        _circle_render_commands.pop()


def _sync_hud_render_commands() -> None:
    for i, line in enumerate(_hud_lines):
        _hud_render_commands[i + 2][1] = line


def _rebuild_render_commands() -> None:
    _render_commands.clear()
    _render_commands.append(_clear_render_command)
    _render_commands.extend(_circle_render_commands)
    _render_commands.extend(_hud_render_commands)


def setup() -> None:
    global _display_fps, _best_stable_balls, _stable_sample_seconds, _hud_refresh_in
    global _session_initial_balls
    _session_initial_balls = (
        max(MIN_BALLS, min(MAX_BALLS, AUTO_INITIAL_BALLS)) if AUTO_ENABLED else INITIAL_BALLS
    )
    _set_ball_count(_session_initial_balls)
    _display_fps = TARGET_FPS
    _best_stable_balls = _session_initial_balls
    _stable_sample_seconds = 0
    _record_nearest_targets()
    _refresh_hud_lines()
    _rebuild_render_commands()
    _hud_refresh_in = HUD_REFRESH_SECONDS
    _log_event(
        "session_start",
        target_fps=TARGET_FPS,
        stable_threshold=f"{STABLE_FPS_THRESHOLD:.2f}",
        initial_balls=_session_initial_balls,
        auto_mode=int(AUTO_ENABLED),
        auto_targets=",".join(str(v) for v in AUTO_TARGETS) if AUTO_TARGETS else "none",
        metric="wall_fps",
        sim_dt_cap=f"{MAX_SIM_DT:.4f}",
    )


def update(dt: float) -> None:
    global _wall_sample_time, _sim_sample_time, _sample_frames
    global _display_fps, _best_stable_balls, _sample_index
    global _elapsed_wall_seconds, _stable_sample_seconds, _hud_refresh_in
    _ = dt

    if pycro.is_key_down("Escape"):
        _emit_summary("escape")
        raise SystemExit(0)

    wall_dt = pycro.frame_time()
    sim_dt = min(wall_dt, MAX_SIM_DT)
    _elapsed_wall_seconds += wall_dt

    if _consume_left_repeat(pycro.is_key_down("Left"), sim_dt):
        _set_ball_count(_ball_count() - BALL_STEP)
        _refresh_hud_lines()
    if _consume_right_repeat(pycro.is_key_down("Right"), sim_dt):
        _set_ball_count(_ball_count() + BALL_STEP)
        _refresh_hud_lines()
    _auto_drive(sim_dt)
    _record_nearest_targets()

    count = _ball_count()
    for i in range(count):
        x = _ball_x[i] + (_ball_vx[i] * sim_dt)
        y = _ball_y[i] + (_ball_vy[i] * sim_dt)
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
        command_position: list[float] = _circle_render_commands[i][1]  # type: ignore[assignment]
        command_position[0] = x
        command_position[1] = y

    _wall_sample_time += wall_dt
    _sim_sample_time += sim_dt
    _sample_frames += 1
    if _wall_sample_time >= 1.0 and not _summary_emitted:
        wall_fps = float(_sample_frames) / max(_wall_sample_time, 1e-6)
        sim_fps = float(_sample_frames) / max(_sim_sample_time, 1e-6)
        _display_fps = int(round(wall_fps))
        stability = "stable" if wall_fps >= STABLE_FPS_THRESHOLD else "unstable"
        if stability == "stable":
            _stable_sample_seconds += 1
            _best_stable_balls = max(_best_stable_balls, count)
        _sample_index += 1
        _log_event(
            "sample",
            second=_sample_index,
            balls=count,
            wall_fps=f"{wall_fps:.2f}",
            sim_fps=f"{sim_fps:.2f}",
            threshold=f"{STABLE_FPS_THRESHOLD:.2f}",
            status=stability,
            best_stable_balls=_best_stable_balls,
            stable_sample_seconds=_stable_sample_seconds,
            nearest_targets=_nearest_summary_cache,
            sim_dt_cap=f"{MAX_SIM_DT:.4f}",
        )
        _wall_sample_time = 0.0
        _sim_sample_time = 0.0
        _sample_frames = 0
        _refresh_hud_lines()

    if AUTO_ENABLED and AUTO_SESSION_SECONDS > 0.0 and _elapsed_wall_seconds >= AUTO_SESSION_SECONDS:
        _emit_summary("auto_session_timeout")

    _hud_refresh_in -= wall_dt
    if _hud_refresh_in <= 0.0:
        _refresh_hud_lines()
        _hud_refresh_in = HUD_REFRESH_SECONDS

    pycro.submit_render(_render_commands)  # type: ignore[arg-type]
