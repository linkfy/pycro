import pycro

hero_texture: pycro.TextureHandle | None = None
MAIN_CIRCLE_POSITION = (320.0, 220.0)
TEXTURE_POSITION = (860.0, 420.0)
CAMERA_FOCUS_TARGET = (360.0, 260.0)


def setup() -> None:
    global hero_texture
    print("[basic_main] Verifying render/input/camera/texture surface.")
    print("[basic_main] Hold Space to snap camera target.")
    hero_texture = pycro.load_texture(
        "examples/assets/kenney_development_essentials/Checkerboard/checkerboard.png"
    )


def update(dt: float) -> None:
    pycro.clear_background((0.08, 0.09, 0.12, 1.0))

    if pycro.is_key_down(pycro.KEY.SPACE):
        pycro.set_camera_target(CAMERA_FOCUS_TARGET)

    pycro.draw_circle(MAIN_CIRCLE_POSITION, 24.0, (0.2, 0.7, 1.0, 1.0))

    if hero_texture is not None:
        pycro.draw_texture(hero_texture, TEXTURE_POSITION, (96.0, 96.0))

    _ = dt + pycro.frame_time()
