import pycro

TOTAL_SLOT_SECONDS = 10
DIAL_CENTER = (420.0, 220.0)
DIAL_RADIUS = 95.0
TIMELINE_START_X = 180.0
TIMELINE_Y = 430.0
TIMELINE_STEP_X = 55.0
TIMELINE_SLOT_RADIUS = 16.0

elapsed_seconds = 0.0
last_whole_second = -1
second_flash = 0.0


def setup() -> None:
    print("[stopwatch_seconds] Stopwatch advances once per elapsed second.")


def update(dt: float) -> None:
    global elapsed_seconds, last_whole_second, second_flash

    elapsed_seconds += pycro.frame_time()
    whole_second = int(elapsed_seconds)
    if whole_second != last_whole_second:
        last_whole_second = whole_second
        second_flash = 0.22
    else:
        second_flash = max(0.0, second_flash - dt)

    pycro.clear_background((0.04, 0.06, 0.10, 1.0))

    slot_second = whole_second % TOTAL_SLOT_SECONDS
    slot_progress = slot_second / (TOTAL_SLOT_SECONDS - 1)

    # Stopwatch dial: one bright marker jumps each whole second.
    marker_x = DIAL_CENTER[0] - DIAL_RADIUS + (slot_progress * DIAL_RADIUS * 2.0)
    pycro.draw_circle(DIAL_CENTER, DIAL_RADIUS + 10.0, (0.08, 0.12, 0.18, 1.0))
    pycro.draw_circle((marker_x, DIAL_CENTER[1]), 20.0, (0.16, 0.84, 0.95, 1.0))
    pycro.draw_circle(DIAL_CENTER, 7.0 + (second_flash * 35.0), (0.98, 0.86, 0.22, 1.0))

    # Timeline: each dot locks in once its second has elapsed.
    for index in range(TOTAL_SLOT_SECONDS):
        x = TIMELINE_START_X + (index * TIMELINE_STEP_X)
        reached = index <= slot_second
        color = (0.23, 0.88, 0.36, 1.0) if reached else (0.20, 0.22, 0.28, 1.0)
        pycro.draw_circle((x, TIMELINE_Y), TIMELINE_SLOT_RADIUS, color)

    pycro.draw_text(
        f"seconds: {whole_second}",
        (300.0, 70.0),
        34.0,
        (0.95, 0.96, 0.98, 1.0),
    )
