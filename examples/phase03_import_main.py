import pycro

import phase03_player

hero: phase03_player.Player | None = None


def setup() -> None:
    global hero
    print("[import_main] main.py imports phase03_player.py and delegates movement/render.")
    hero = phase03_player.create_player("Hero")


def update(dt: float) -> None:
    pycro.clear_background((0.06, 0.07, 0.10, 1.0))

    if hero is None:
        return

    phase03_player.update_player(hero, dt)
    phase03_player.draw_player(hero)

    pycro.draw_text(
        f"frame_time={pycro.frame_time():.3f}",
        (24.0, 30.0),
        24.0,
        (0.95, 0.92, 0.62, 1.0),
    )
