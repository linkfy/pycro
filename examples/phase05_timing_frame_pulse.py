import pycro

pulse = 0.0
pulse_dir = 1.0


def setup() -> None:
    print("[timing_frame_pulse] Pulse animation is driven by pycro.frame_time().")
    print("[timing_frame_pulse] Visual checks: center pulse breathes, corner dot follows dt.")


def update(dt: float) -> None:
    global pulse, pulse_dir

    _ = dt
    pulse += pycro.frame_time() * pulse_dir
    if pulse > 1.0:
        pulse = 1.0
        pulse_dir = -1.0
    elif pulse < 0.0:
        pulse = 0.0
        pulse_dir = 1.0

    pycro.clear_background((0.03, 0.05, 0.10, 1.0))

    radius = 20.0 + (pulse * 70.0)
    color = (0.20 + (0.60 * pulse), 0.45, 1.00 - (0.40 * pulse), 1.0)
    pycro.draw_circle((420.0, 260.0), radius, color)

    # Visual heartbeat marker tied directly to current frame time.
    dt_dot_radius = 8.0 + (pycro.frame_time() * 220.0)
    pycro.draw_circle((80.0, 80.0), dt_dot_radius, (0.98, 0.78, 0.22, 1.0))
