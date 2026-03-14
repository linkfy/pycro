import math
import os

import pycro

POINT_COUNT = 44
WAVE_WIDTH = 1120.0
WAVE_LEFT = 80.0
WAVE_CENTER_Y = 330.0
WAVE_SPEED = 1.8

phase = 0.0
frequency = 1.4
amplitude = 96.0
os_summary = "os import pending"


def setup() -> None:
    global os_summary

    cwd = os.getcwd()
    cwd_leaf = os.path.basename(cwd) or cwd
    has_home = bool(os.getenv("HOME", ""))
    os_summary = f"os.name={os.name} cwd={cwd_leaf} home={has_home}"

    print("[stdlib_wave_lab] Imported stdlib modules math + os directly.")
    print("[stdlib_wave_lab] Controls: Left/Right frequency, Up/Down amplitude, Space reset.")


def update(dt: float) -> None:
    global phase, frequency, amplitude

    if pycro.is_key_down(pycro.KEY.LEFT):
        frequency = max(0.4, frequency - (0.9 * dt))
    if pycro.is_key_down(pycro.KEY.RIGHT):
        frequency = min(4.0, frequency + (0.9 * dt))
    if pycro.is_key_down(pycro.KEY.DOWN):
        amplitude = max(24.0, amplitude - (120.0 * dt))
    if pycro.is_key_down(pycro.KEY.UP):
        amplitude = min(220.0, amplitude + (120.0 * dt))

    if pycro.is_key_down(pycro.KEY.SPACE):
        phase = 0.0
    else:
        phase += pycro.frame_time() * WAVE_SPEED

    wave_energy = 0.5 + (0.5 * math.sin(phase * 0.7))
    pycro.clear_background((0.04, 0.07 + (wave_energy * 0.06), 0.12, 1.0))

    for i in range(POINT_COUNT):
        t = i / (POINT_COUNT - 1)
        x = WAVE_LEFT + (t * WAVE_WIDTH)
        angle = (t * (2.0 * math.pi) * frequency) + phase
        y = WAVE_CENTER_Y + (math.sin(angle) * amplitude)
        glow = 0.35 + (0.65 * (0.5 + (0.5 * math.cos(angle))))
        pycro.draw_circle((x, y), 5.0 + (glow * 3.0), (0.18, 0.62 + (0.30 * glow), 1.0, 1.0))

    pycro.draw_text(
        "stdlib wave lab: direct import math + os",
        (260.0, 56.0),
        30.0,
        (0.96, 0.97, 1.0, 1.0),
    )
    pycro.draw_text(
        os_summary,
        (290.0, 90.0),
        24.0,
        (0.80, 0.86, 0.95, 1.0),
    )
    pycro.draw_text(
        f"frequency={frequency:.2f} amplitude={amplitude:.1f} (arrows adjust, Space resets phase)",
        (140.0, 126.0),
        24.0,
        (0.98, 0.89, 0.58, 1.0),
    )
