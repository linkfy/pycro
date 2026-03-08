import pycro

loaded_texture: pycro.TextureHandle | None = None
missing_texture: pycro.TextureHandle | None = None


def setup() -> None:
    global loaded_texture, missing_texture
    print("[texture_draw] Space swaps loaded and fallback texture slots.")

    loaded_texture = pycro.load_texture(
        "examples/assets/kenney_development_essentials/Gradient/gradient-radial.png"
    )
    missing_texture = pycro.load_texture("examples/assets/missing.png")


def update(dt: float) -> None:
    _ = dt
    pycro.clear_background((0.07, 0.07, 0.09, 1.0))

    if loaded_texture is None or missing_texture is None:
        pycro.draw_circle((420.0, 260.0), 45.0, (0.95, 0.22, 0.22, 1.0))
        return

    show_loaded_left = not pycro.is_key_down("Space")

    left_tex = loaded_texture if show_loaded_left else missing_texture
    right_tex = missing_texture if show_loaded_left else loaded_texture

    pycro.draw_texture(left_tex, (220.0, 170.0), (180.0, 180.0))
    pycro.draw_texture(right_tex, (460.0, 170.0), (180.0, 180.0))

    # Green marker: loaded texture on the left. Orange marker: loaded texture on the right.
    marker_color = (0.20, 0.90, 0.40, 1.0) if show_loaded_left else (0.95, 0.62, 0.22, 1.0)
    pycro.draw_circle((420.0, 70.0), 18.0, marker_color)
