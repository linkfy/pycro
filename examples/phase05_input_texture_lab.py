import pycro

PLAYER_SPEED = 240.0
TOGGLE_COOLDOWN_SECONDS = 0.20
MIN_SCALE = 48.0
MAX_SCALE = 260.0

player_pos = [420.0, 250.0]
sprite_scale = 140.0
toggle_cooldown = 0.0
texture_index = 0

checker_texture: pycro.TextureHandle | None = None
gradient_texture: pycro.TextureHandle | None = None
missing_texture: pycro.TextureHandle | None = None


def setup() -> None:
    global checker_texture, gradient_texture, missing_texture
    print("[phase05_lab] Controls: arrows move sprite, Up/Down change size.")
    print("[phase05_lab] Hold Space to rotate texture source (loaded/fallback).")

    checker_texture = pycro.load_texture(
        "examples/assets/kenney_development_essentials/Checkerboard/checkerboard.png"
    )
    gradient_texture = pycro.load_texture(
        "examples/assets/kenney_development_essentials/Gradient/gradient-radial.png"
    )
    missing_texture = pycro.load_texture("examples/assets/does_not_exist.png")


def _active_texture() -> pycro.TextureHandle | None:
    if checker_texture is None or gradient_texture is None or missing_texture is None:
        return None
    textures = [checker_texture, gradient_texture, missing_texture]
    return textures[texture_index % len(textures)]


def _active_texture_label() -> str:
    labels = ["checkerboard (loaded)", "gradient (loaded)", "missing (fallback)"]
    return labels[texture_index % len(labels)]


def _is_fallback_active() -> bool:
    return texture_index % 3 == 2


def update(dt: float) -> None:
    global player_pos, sprite_scale, toggle_cooldown, texture_index

    if pycro.is_key_down("Left"):
        player_pos[0] -= PLAYER_SPEED * dt
    if pycro.is_key_down("Right"):
        player_pos[0] += PLAYER_SPEED * dt
    if pycro.is_key_down("Up"):
        sprite_scale += PLAYER_SPEED * 0.60 * dt
    if pycro.is_key_down("Down"):
        sprite_scale -= PLAYER_SPEED * 0.60 * dt

    sprite_scale = max(MIN_SCALE, min(MAX_SCALE, sprite_scale))
    toggle_cooldown = max(0.0, toggle_cooldown - dt)

    if pycro.is_key_down("Space") and toggle_cooldown <= 0.0:
        texture_index += 1
        toggle_cooldown = TOGGLE_COOLDOWN_SECONDS

    pycro.clear_background((0.08, 0.09, 0.11, 1.0))

    pycro.draw_circle((90.0, 76.0), 14.0 + (pycro.frame_time() * 220.0), (0.95, 0.80, 0.20, 1.0))
    status_color = (0.90, 0.28, 0.28, 1.0) if _is_fallback_active() else (0.20, 0.92, 0.46, 1.0)
    pycro.draw_circle((124.0, 76.0), 9.0, status_color)

    tex = _active_texture()
    if tex is not None:
        pycro.draw_texture(
            tex,
            (player_pos[0] - (sprite_scale * 0.5), player_pos[1] - (sprite_scale * 0.5)),
            (sprite_scale, sprite_scale),
        )
    else:
        pycro.draw_circle((player_pos[0], player_pos[1]), 34.0, (0.90, 0.22, 0.22, 1.0))

    pycro.draw_text("phase05_input_texture_lab", (18.0, 26.0), 26.0, (0.86, 0.95, 1.0, 1.0))
    pycro.draw_text(
        "texture: " + _active_texture_label(),
        (18.0, 54.0),
        20.0,
        (0.72, 0.90, 0.78, 1.0),
    )
    pycro.draw_text(
        "fallback active" if _is_fallback_active() else "loaded texture active",
        (18.0, 106.0),
        18.0,
        status_color,
    )
    pycro.draw_text(
        "controls: Left/Right move | Up/Down size | Space rotate texture",
        (18.0, 132.0),
        18.0,
        (0.86, 0.86, 0.90, 1.0),
    )
