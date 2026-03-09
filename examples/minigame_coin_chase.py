import pycro

WORLD_WIDTH = 960.0
WORLD_HEIGHT = 540.0
GROUND_Y = 452.0

PLAYER_SIZE = (54.0, 66.0)
PLAYER_SPEED = 265.0
PLAYER_START = (92.0, 368.0)

COIN_SIZE = (34.0, 34.0)
ROUND_SECONDS = 45.0
COLLECTION_RADIUS = 40.0

SPAWN_POINTS = [
    (170.0, 140.0),
    (290.0, 250.0),
    (430.0, 168.0),
    (560.0, 300.0),
    (710.0, 220.0),
    (810.0, 148.0),
    (700.0, 365.0),
    (520.0, 396.0),
    (340.0, 360.0),
    (210.0, 300.0),
]

background_texture: pycro.TextureHandle | None = None
ground_texture: pycro.TextureHandle | None = None
player_texture: pycro.TextureHandle | None = None
coin_texture: pycro.TextureHandle | None = None

player_x = PLAYER_START[0]
player_y = PLAYER_START[1]
active_coin_index = 0
score = 0
elapsed_seconds = 0.0
round_finished = False


def _clamp(value: float, low: float, high: float) -> float:
    if value < low:
        return low
    if value > high:
        return high
    return value


def _reset_round() -> None:
    global player_x, player_y, active_coin_index, score, elapsed_seconds, round_finished
    player_x = PLAYER_START[0]
    player_y = PLAYER_START[1]
    active_coin_index = 0
    score = 0
    elapsed_seconds = 0.0
    round_finished = False


def _center(position: tuple[float, float], size: tuple[float, float]) -> tuple[float, float]:
    return (position[0] + (size[0] * 0.5), position[1] + (size[1] * 0.5))


def setup() -> None:
    global background_texture, ground_texture, player_texture, coin_texture
    print("[minigame_coin_chase] Arrow keys move. Collect coins before time runs out.")
    print("[minigame_coin_chase] Press Space to reset the run at any time.")

    background_texture = pycro.load_texture(
        "examples/assets/kenney_platformer_art_deluxe/Base pack/bg.png"
    )
    ground_texture = pycro.load_texture(
        "examples/assets/kenney_platformer_art_deluxe/Base pack/Tiles/grassMid.png"
    )
    player_texture = pycro.load_texture(
        "examples/assets/kenney_platformer_art_deluxe/Base pack/Player/p1_front.png"
    )
    coin_texture = pycro.load_texture(
        "examples/assets/kenney_platformer_art_deluxe/Base pack/Items/coinGold.png"
    )
    _reset_round()


def update(dt: float) -> None:
    global player_x, player_y, active_coin_index, score, elapsed_seconds, round_finished

    if pycro.is_key_down("Space"):
        _reset_round()

    if not round_finished:
        if pycro.is_key_down("Left"):
            player_x -= PLAYER_SPEED * dt
        if pycro.is_key_down("Right"):
            player_x += PLAYER_SPEED * dt
        if pycro.is_key_down("Up"):
            player_y -= PLAYER_SPEED * dt
        if pycro.is_key_down("Down"):
            player_y += PLAYER_SPEED * dt

        player_x = _clamp(player_x, 16.0, WORLD_WIDTH - PLAYER_SIZE[0] - 16.0)
        player_y = _clamp(player_y, 24.0, WORLD_HEIGHT - PLAYER_SIZE[1] - 24.0)

        elapsed_seconds += dt
        if elapsed_seconds >= ROUND_SECONDS:
            elapsed_seconds = ROUND_SECONDS
            round_finished = True

        coin_pos = SPAWN_POINTS[active_coin_index]
        coin_center = _center(coin_pos, COIN_SIZE)
        player_center = _center((player_x, player_y), PLAYER_SIZE)
        dx = coin_center[0] - player_center[0]
        dy = coin_center[1] - player_center[1]
        if (dx * dx) + (dy * dy) <= COLLECTION_RADIUS * COLLECTION_RADIUS:
            score += 1
            active_coin_index = (active_coin_index + 1) % len(SPAWN_POINTS)

    pycro.clear_background((0.10, 0.17, 0.24, 1.0))

    if background_texture is not None:
        pycro.draw_texture(background_texture, (0.0, 0.0), (WORLD_WIDTH, WORLD_HEIGHT))

    if ground_texture is not None:
        ground_width = 96.0
        x = -32.0
        while x < WORLD_WIDTH + 32.0:
            pycro.draw_texture(ground_texture, (x, GROUND_Y), (ground_width, 96.0))
            x += ground_width

    coin_pos = SPAWN_POINTS[active_coin_index]
    if coin_texture is not None:
        pycro.draw_texture(coin_texture, coin_pos, COIN_SIZE)
    else:
        pycro.draw_circle((coin_pos[0] + 17.0, coin_pos[1] + 17.0), 15.0, (1.0, 0.82, 0.0, 1.0))

    if player_texture is not None:
        pycro.draw_texture(player_texture, (player_x, player_y), PLAYER_SIZE)
    else:
        pycro.draw_circle((player_x + 27.0, player_y + 33.0), 22.0, (0.25, 0.9, 0.9, 1.0))

    time_left = ROUND_SECONDS - elapsed_seconds
    pycro.draw_text(
        f"Score: {score}",
        (20.0, 30.0),
        30.0,
        (0.98, 0.98, 1.0, 1.0),
    )
    pycro.draw_text(
        f"Time: {time_left:05.2f}s",
        (20.0, 64.0),
        28.0,
        (0.95, 0.95, 0.50, 1.0),
    )
    pycro.draw_text(
        "Arrows = move    Space = reset",
        (20.0, 98.0),
        24.0,
        (0.87, 0.91, 0.98, 1.0),
    )

    if round_finished:
        pycro.draw_text(
            "TIME UP! Press Space to try again.",
            (220.0, 200.0),
            42.0,
            (1.0, 0.85, 0.35, 1.0),
        )
