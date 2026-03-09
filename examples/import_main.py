import pycro

import player

hero: player.Player | None = None


def setup() -> None:
    global hero
    print("[import_main] main.py imports player.py and delegates movement/render.")
    hero = player.create_player("Hero")


def update(dt: float) -> None:
    pycro.clear_background((0.06, 0.07, 0.10, 1.0))

    if hero is None:
        return

    player.update_player(hero, dt)
    player.draw_player(hero)

    pycro.draw_text(
        f"frame_time={pycro.frame_time():.3f}",
        (24.0, 30.0),
        24.0,
        (0.95, 0.92, 0.62, 1.0),
    )
