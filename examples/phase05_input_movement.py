import pycro

PLAYER_RADIUS = 24.0
PLAYER_SPEED = 260.0
START_POSITION = (400.0, 260.0)

player_x = START_POSITION[0]
player_y = START_POSITION[1]


def setup() -> None:
    print("[input_movement] Controls: arrows move the player, Space enables turbo.")
    print("[input_movement] Visual checks: turbo warms background and increases speed.")


def update(dt: float) -> None:
    global player_x, player_y

    turbo = pycro.is_key_down("Space")
    speed = PLAYER_SPEED * (1.75 if turbo else 1.0)

    if pycro.is_key_down("Left"):
        player_x -= speed * dt
    if pycro.is_key_down("Right"):
        player_x += speed * dt
    if pycro.is_key_down("Up"):
        player_y -= speed * dt
    if pycro.is_key_down("Down"):
        player_y += speed * dt

    background = (0.20, 0.08, 0.08, 1.0) if turbo else (0.06, 0.08, 0.14, 1.0)
    pycro.clear_background(background)

    dt_indicator = 10.0 + (pycro.frame_time() * 300.0)
    pycro.draw_circle((90.0, 70.0), dt_indicator, (1.0, 0.78, 0.30, 1.0))
    pycro.draw_circle((player_x, player_y), PLAYER_RADIUS, (0.24, 0.82, 0.95, 1.0))
