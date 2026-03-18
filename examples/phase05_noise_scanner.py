import pycro

perlin_large: pycro.TextureHandle | None = None
perlin_small: pycro.TextureHandle | None = None
uv_texture: pycro.TextureHandle | None = None

pulse_seconds = 0.0
scanner_y = 220.0


def setup() -> None:
    global perlin_large, perlin_small, uv_texture
    print("[pack_noise_scanner] Up/Down moves scanner marker, Space resets.")
    perlin_large = pycro.load_texture(
        "examples/assets/kenney_development_essentials/Noise/perlin-noise.png"
    )
    perlin_small = pycro.load_texture(
        "examples/assets/kenney_development_essentials/Noise/perlin-noise-small.png"
    )
    uv_texture = pycro.load_texture(
        "examples/assets/kenney_development_essentials/UV texture/uv-texture.png"
    )


def update(dt: float) -> None:
    global pulse_seconds, scanner_y
    pulse_seconds += dt

    pycro.clear_background((0.05, 0.06, 0.08, 1.0))

    if perlin_large is not None:
        pycro.draw_texture(perlin_large, (90.0, 70.0), (280.0, 280.0))
    if perlin_small is not None:
        pycro.draw_texture(perlin_small, (390.0, 70.0), (280.0, 280.0))
    if uv_texture is not None:
        pycro.draw_texture(uv_texture, (690.0, 70.0), (280.0, 280.0))

    scanner_y += (pycro.frame_time() - 0.016) * 90.0
    if pycro.is_key_down(pycro.KEY.UP):
        scanner_y -= 110.0 * dt
    if pycro.is_key_down(pycro.KEY.DOWN):
        scanner_y += 110.0 * dt
    if pycro.is_key_down(pycro.KEY.SPACE):
        scanner_y = 220.0

    pulse_radius = 12.0 + (pulse_seconds % 0.75) * 12.0
    pycro.draw_circle((530.0, scanner_y), pulse_radius, (0.10, 0.92, 0.66, 1.0))
    pycro.draw_text(
        "Kenney Noise + UV Texture Scanner",
        (20.0, 30.0),
        24.0,
        (0.94, 0.96, 0.99, 1.0),
    )
