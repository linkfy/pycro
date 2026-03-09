import pycro

gradients: list[pycro.TextureHandle] = []
active_index = 0
previous_space = False

PATHS = [
    "examples/assets/kenney_development_essentials/Gradient/gradient-radial.png",
    "examples/assets/kenney_development_essentials/Gradient/gradient-horizontal.png",
    "examples/assets/kenney_development_essentials/Gradient/gradient-vertical.png",
    "examples/assets/kenney_development_essentials/Gradient/gradient-angular.png",
]


def setup() -> None:
    global gradients
    print("[pack_gradient_cycle] Press Space to cycle gradient textures.")
    gradients = [pycro.load_texture(path) for path in PATHS]


def update(dt: float) -> None:
    global active_index, previous_space
    _ = dt

    pycro.clear_background((0.08, 0.08, 0.11, 1.0))

    if not gradients:
        pycro.draw_circle((420.0, 240.0), 45.0, (0.95, 0.22, 0.22, 1.0))
        return

    current_space = pycro.is_key_down("Space")
    if current_space and not previous_space:
        active_index = (active_index + 1) % len(gradients)
    previous_space = current_space

    current_texture = gradients[active_index]

    pycro.draw_texture(current_texture, (160.0, 80.0), (520.0, 320.0))
    pycro.draw_text(
        f"Gradient {active_index + 1}/{len(gradients)}",
        (26.0, 36.0),
        30.0,
        (0.96, 0.96, 0.98, 1.0),
    )
    pycro.draw_text(
        "Space: next texture",
        (26.0, 68.0),
        22.0,
        (0.74, 0.86, 0.98, 1.0),
    )
