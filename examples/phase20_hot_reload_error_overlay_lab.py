import pycro

BG_OK = (0.08, 0.11, 0.14, 1.0)
TEXT = (0.92, 0.95, 0.98, 1.0)

# Manual phase-20 validation toggle:
# 1) Run this file with `pycro`.
# 2) Change `TRIGGER_RUNTIME_ERROR` to True and save.
# 3) Confirm in-window error overlay appears.
# 4) Change it back to False and save.
# 5) Confirm runtime recovers without restarting process.
TRIGGER_RUNTIME_ERROR = False


def update(dt: float) -> None:
    pycro.clear_background(BG_OK)
    pycro.draw_text("phase20 hot reload + runtime overlay lab", (24.0, 48.0), 32.0, TEXT)
    pycro.draw_text("Edit this file while app is running.", (24.0, 88.0), 26.0, TEXT)
    pycro.draw_text(
        "Set TRIGGER_RUNTIME_ERROR = True, save, then set back to False.",
        (24.0, 124.0),
        24.0,
        TEXT,
    )

    if TRIGGER_RUNTIME_ERROR:
        raise NameError("name 'Player' is not defined")
