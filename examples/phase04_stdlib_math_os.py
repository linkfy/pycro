import math
import os

import pycro

CENTER = (640.0, 300.0)
ORBIT_RADIUS = 170.0
BASE_DOT_RADIUS = 18.0

elapsed = 0.0
os_summary = "os import pending"


def setup() -> None:
    global os_summary

    cwd = os.getcwd()
    cwd_leaf = os.path.basename(cwd) or cwd
    home_present = bool(os.getenv("HOME", ""))
    os_summary = f"os.name={os.name} sep='{os.sep}' cwd={cwd_leaf} home={home_present}"

    print("[stdlib_math_os] Imported stdlib modules math + os without workarounds.")
    print(f"[stdlib_math_os] math.pi={math.pi:.6f} os.getcwd()={cwd}")


def update(dt: float) -> None:
    global elapsed
    _ = dt

    if pycro.is_key_down("Space"):
        elapsed = 0.0
    else:
        elapsed += pycro.frame_time()

    angle = elapsed * 1.8
    dot_x = CENTER[0] + (math.cos(angle) * ORBIT_RADIUS)
    dot_y = CENTER[1] + (math.sin(angle) * ORBIT_RADIUS)
    dot_radius = BASE_DOT_RADIUS + (math.sin(angle * 2.0) * 4.0)
    distance = math.hypot(dot_x - CENTER[0], dot_y - CENTER[1])

    pycro.clear_background((0.06, 0.07, 0.10, 1.0))
    pycro.draw_circle(CENTER, ORBIT_RADIUS + 2.0, (0.17, 0.20, 0.29, 1.0))
    pycro.draw_circle((dot_x, dot_y), dot_radius, (0.15, 0.88, 0.68, 1.0))
    pycro.draw_circle(CENTER, 8.0, (0.97, 0.83, 0.32, 1.0))

    pycro.draw_text(
        "stdlib demo: import math + os directly in script",
        (190.0, 52.0),
        26.0,
        (0.95, 0.96, 0.98, 1.0),
    )
    pycro.draw_text(
        os_summary,
        (170.0, 84.0),
        24.0,
        (0.84, 0.88, 0.97, 1.0),
    )
    pycro.draw_text(
        f"orbit_distance={distance:.1f} (Space resets timer)",
        (230.0, 120.0),
        24.0,
        (0.96, 0.91, 0.60, 1.0),
    )
