import pycro

hero_texture: pycro.TextureHandle | None = None


def setup() -> None:
    global hero_texture
    print("[pycro] setup() inicializado")
    hero_texture = pycro.load_texture("assets/hero.png")


def update(dt: float) -> None:
    pycro.clear_background((0.08, 0.09, 0.12, 1.0))

    if pycro.is_key_down("Space"):
        pycro.set_camera_target((0.0, 0.0))

    pycro.draw_circle((160.0, 90.0), 24.0, (0.2, 0.7, 1.0, 1.0))

    if hero_texture is not None:
        pycro.draw_texture(hero_texture, (120.0, 80.0), (64.0, 64.0))

    _ = dt + pycro.frame_time()
