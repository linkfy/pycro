import pycro

GRID_COLS = 12
GRID_ROWS = 6
START_X = 90.0
START_Y = 110.0
CELL_W = 92.0
CELL_H = 76.0
BASE_RADIUS = 16.0

frame_index = 0
phase = 0.0


def setup() -> None:
    print("[runtime_draw_flush_batch] validating multi-draw flush per frame")
    print("[runtime_draw_flush_batch] expected: animated grid + moving sweep marker")


def _frame_dt(dt: float) -> float:
    sampled = pycro.frame_time()
    if sampled > 0.0:
        return sampled
    return dt


def update(dt: float) -> None:
    global frame_index, phase

    frame_index += 1
    phase = (phase + (_frame_dt(dt) * 0.85)) % 1.0

    pycro.clear_background((0.04, 0.05, 0.08, 1.0))

    draw_calls = 0
    for row in range(GRID_ROWS):
        for col in range(GRID_COLS):
            anim = float((frame_index + (row * 3) + (col * 5)) % 14)
            brightness = 0.22 + (anim * 0.05)
            jitter = ((float((frame_index + row + col) % 7) - 3.0) * 1.8)
            x = START_X + (float(col) * CELL_W) + jitter
            y = START_Y + (float(row) * CELL_H) + (((anim % 4.0) - 1.5) * 2.3)
            radius = BASE_RADIUS + ((anim % 5.0) * 0.9)
            pycro.draw_circle(
                (x, y),
                radius,
                (0.10 + (brightness * 0.35), 0.30 + (brightness * 0.46), 0.95, 0.94),
            )
            draw_calls += 1

    sweep_x = 80.0 + (phase * 1020.0)
    for idx in range(4):
        y = 88.0 + (float(idx) * 150.0)
        pycro.draw_circle((sweep_x, y), 10.0 + (float(idx) * 2.5), (1.0, 0.87, 0.28, 0.92))
        draw_calls += 1

    pycro.draw_text(
        "runtime_draw_flush_batch: many draw calls each frame",
        (84.0, 38.0),
        28.0,
        (0.95, 0.97, 1.0, 1.0),
    )
    pycro.draw_text(
        f"frame={frame_index} draw_calls={draw_calls} phase={phase:.2f}",
        (84.0, 72.0),
        24.0,
        (0.98, 0.84, 0.40, 1.0),
    )

    print(
        f"[runtime_draw_flush_batch] frame={frame_index} draw_calls={draw_calls} phase={phase:.2f}"
    )
