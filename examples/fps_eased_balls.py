import pycro

LEFT_BOUND = 90.0
RIGHT_BOUND = 1190.0
EASE_ZONE = 180.0
TOP_LANE_Y = 140.0
BOTTOM_LANE_Y = 530.0
MIN_FPS = 1
MAX_FPS = 120
FPS_REPEAT_SECONDS = 0.09
MIN_SPEED_SCALE = 0.2
MAX_SPEED_SCALE = 6.0
SPEED_STEP = 0.1

balls: list[dict[str, float]] = []
effective_fps = 60
speed_scale = 1.0
sim_accumulator = 0.0
fps_repeat_timer = 0.0
sim_fps_sample_seconds = 0.0
sim_fps_sample_steps = 0
render_fps_sample_seconds = 0.0
render_fps_sample_frames = 0
displayed_sim_fps = 60
displayed_render_fps = 60


def _smoothstep(value: float) -> float:
    if value <= 0.0:
        return 0.0
    if value >= 1.0:
        return 1.0
    return value * value * (3.0 - (2.0 * value))


def _init_balls() -> None:
    global balls
    balls = []
    for index in range(7):
        balls.append(
            {
                "x": LEFT_BOUND + (index * 110.0),
                "y": TOP_LANE_Y + (index * 65.0),
                "speed": 160.0 + (index * 24.0),
                "direction": 1.0,
                "radius": 11.0 + (index * 1.5),
            }
        )


def setup() -> None:
    _init_balls()
    print("[fps_eased_balls] Left/Right adjusts effective FPS (1..120).")
    print("[fps_eased_balls] Up/Down adjusts ball speed scale.")
    print("[fps_eased_balls] Balls ease in/out near side bounds using fixed-step simulation.")


def _step_simulation(step_dt: float) -> None:
    for ball in balls:
        x = ball["x"]
        direction = ball["direction"]
        speed = ball["speed"]

        nearest_bound_distance = x - LEFT_BOUND
        right_distance = RIGHT_BOUND - x
        if right_distance < nearest_bound_distance:
            nearest_bound_distance = right_distance

        ease_ratio = nearest_bound_distance / EASE_ZONE
        eased_speed_factor = 0.20 + (0.80 * _smoothstep(ease_ratio))
        lane_ratio = (ball["y"] - TOP_LANE_Y) / (BOTTOM_LANE_Y - TOP_LANE_Y)
        lane_ratio = max(0.0, min(1.0, lane_ratio))
        vertical_speed_factor = 0.70 + (1.90 * lane_ratio)
        x += (speed * speed_scale * vertical_speed_factor * eased_speed_factor * direction) * step_dt

        if x >= RIGHT_BOUND:
            x = RIGHT_BOUND
            direction = -1.0
        elif x <= LEFT_BOUND:
            x = LEFT_BOUND
            direction = 1.0

        ball["x"] = x
        ball["direction"] = direction


def _update_effective_fps(frame_dt: float) -> None:
    global effective_fps, speed_scale, fps_repeat_timer

    fps_repeat_timer -= frame_dt
    if fps_repeat_timer > 0.0:
        return

    moved = False
    if pycro.is_key_down("Left"):
        effective_fps = max(MIN_FPS, effective_fps - 1)
        moved = True
    if pycro.is_key_down("Right"):
        effective_fps = min(MAX_FPS, effective_fps + 1)
        moved = True
    if pycro.is_key_down("Up"):
        speed_scale = min(MAX_SPEED_SCALE, speed_scale + SPEED_STEP)
        moved = True
    if pycro.is_key_down("Down"):
        speed_scale = max(MIN_SPEED_SCALE, speed_scale - SPEED_STEP)
        moved = True

    if moved:
        fps_repeat_timer = FPS_REPEAT_SECONDS


def update(dt: float) -> None:
    global sim_accumulator, sim_fps_sample_seconds, sim_fps_sample_steps, displayed_sim_fps
    global render_fps_sample_seconds, render_fps_sample_frames, displayed_render_fps

    _ = dt
    frame_dt = pycro.frame_time()
    render_fps_sample_seconds += frame_dt
    render_fps_sample_frames += 1
    _update_effective_fps(frame_dt)

    step_dt = 1.0 / float(effective_fps)
    sim_accumulator += frame_dt

    # Keep deterministic progression: fixed-step sim clock driven by frame_time.
    step_budget = 120
    while sim_accumulator >= step_dt and step_budget > 0:
        _step_simulation(step_dt)
        sim_accumulator -= step_dt
        sim_fps_sample_steps += 1
        step_budget -= 1

    if step_budget == 0 and sim_accumulator > step_dt:
        sim_accumulator = step_dt

    sim_fps_sample_seconds += frame_dt
    if sim_fps_sample_seconds >= 1.0:
        displayed_sim_fps = int(
            round(float(sim_fps_sample_steps) / max(sim_fps_sample_seconds, 0.0001))
        )
        sim_fps_sample_seconds = 0.0
        sim_fps_sample_steps = 0
    if render_fps_sample_seconds >= 1.0:
        displayed_render_fps = int(
            round(float(render_fps_sample_frames) / max(render_fps_sample_seconds, 0.0001))
        )
        render_fps_sample_seconds = 0.0
        render_fps_sample_frames = 0

    pycro.clear_background((0.04, 0.07, 0.12, 1.0))

    pycro.draw_text(
        "fps eased balls: lower lanes run faster + lateral easing",
        (130.0, 48.0),
        30.0,
        (0.95, 0.97, 1.0, 1.0),
    )
    pycro.draw_text(
        f"effective_fps={effective_fps} speed_scale={speed_scale:.1f}",
        (300.0, 84.0),
        26.0,
        (0.98, 0.86, 0.52, 1.0),
    )
    pycro.draw_text(
        f"sim_fps={displayed_sim_fps}  render_fps={displayed_render_fps}",
        (420.0, 114.0),
        22.0,
        (0.78, 0.87, 0.98, 1.0),
    )

    pycro.draw_circle((LEFT_BOUND, 565.0), 5.0, (0.55, 0.63, 0.78, 1.0))
    pycro.draw_circle((RIGHT_BOUND, 565.0), 5.0, (0.55, 0.63, 0.78, 1.0))

    for index, ball in enumerate(balls):
        tint = 0.25 + (index * 0.09)
        pycro.draw_circle(
            (ball["x"], ball["y"]),
            ball["radius"],
            (0.14 + (tint * 0.35), 0.48 + (tint * 0.34), 0.96, 1.0),
        )
