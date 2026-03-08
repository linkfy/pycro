import pycro

target_x = 400.0
target_y = 250.0
CAMERA_SPEED = 320.0

WORLD_POINTS = (
    ((120.0, 120.0), (0.95, 0.30, 0.30, 1.0)),
    ((800.0, 120.0), (0.30, 0.95, 0.42, 1.0)),
    ((120.0, 620.0), (0.35, 0.62, 0.98, 1.0)),
    ((800.0, 620.0), (0.96, 0.88, 0.34, 1.0)),
    ((460.0, 360.0), (0.93, 0.34, 0.93, 1.0)),
)


def setup() -> None:
    print("[camera_target_pan] Controls: arrows pan camera target, Space boosts speed.")
    print("[camera_target_pan] Visual checks: world markers move as camera target shifts.")


def update(dt: float) -> None:
    global target_x, target_y

    speed = CAMERA_SPEED * (2.0 if pycro.is_key_down("Space") else 1.0)

    if pycro.is_key_down("Left"):
        target_x -= speed * dt
    if pycro.is_key_down("Right"):
        target_x += speed * dt
    if pycro.is_key_down("Up"):
        target_y -= speed * dt
    if pycro.is_key_down("Down"):
        target_y += speed * dt

    pycro.set_camera_target((target_x, target_y))
    pycro.clear_background((0.03, 0.03, 0.05, 1.0))

    for position, color in WORLD_POINTS:
        pycro.draw_circle(position, 28.0, color)

    # Camera target marker in world space.
    pycro.draw_circle((target_x, target_y), 12.0, (1.0, 1.0, 1.0, 1.0))
