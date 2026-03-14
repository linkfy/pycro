import pycro

checkerboard: pycro.TextureHandle | None = None
checkerboard_transparent: pycro.TextureHandle | None = None

camera_target_x = 320.0


def setup() -> None:
    global checkerboard, checkerboard_transparent
    print("[pack_checkerboard_pan] Arrow keys pan camera target; Space recenters.")
    checkerboard = pycro.load_texture(
        "examples/assets/kenney_development_essentials/Checkerboard/checkerboard.png"
    )
    checkerboard_transparent = pycro.load_texture(
        "examples/assets/kenney_development_essentials/Checkerboard/checkerboard-transparent.png"
    )


def update(dt: float) -> None:
    global camera_target_x

    pycro.clear_background((0.06, 0.07, 0.10, 1.0))

    speed = 140.0
    if pycro.is_key_down(pycro.KEY.LEFT):
        camera_target_x -= speed * dt
    if pycro.is_key_down(pycro.KEY.RIGHT):
        camera_target_x += speed * dt
    if pycro.is_key_down(pycro.KEY.SPACE):
        camera_target_x = 320.0

    pycro.set_camera_target((camera_target_x, 180.0))

    if checkerboard is not None:
        pycro.draw_texture(checkerboard, (180.0, 120.0), (220.0, 220.0))
    if checkerboard_transparent is not None:
        pycro.draw_texture(checkerboard_transparent, (450.0, 120.0), (220.0, 220.0))

    pycro.draw_text(
        "Kenney Checkerboard + Camera Pan",
        (18.0, 28.0),
        24.0,
        (0.92, 0.94, 0.98, 1.0),
    )
    pycro.draw_circle((camera_target_x, 350.0), 10.0, (0.18, 0.78, 0.96, 1.0))
