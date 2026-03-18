GREETING = "phase15 embedded payload lab"


def next_tint(elapsed: float) -> tuple[float, float, float, float]:
    wave = 0.5 + 0.5 * (elapsed % 1.0)
    return (0.35 + 0.45 * wave, 0.82, 0.96, 1.0)
