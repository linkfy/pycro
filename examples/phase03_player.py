import pycro


class Player:
    def __init__(self, name: str, position: pycro.Vec2) -> None:
        self.name = name
        self.x = position[0]
        self.y = position[1]
        self.speed = 180.0


def create_player(name: str) -> Player:
    return Player(name, (240.0, 180.0))


def update_player(player: Player, dt: float) -> None:
    if pycro.is_key_down(pycro.KEY.RIGHT):
        player.x += player.speed * dt
    if pycro.is_key_down(pycro.KEY.LEFT):
        player.x -= player.speed * dt
    if pycro.is_key_down(pycro.KEY.DOWN):
        player.y += player.speed * dt
    if pycro.is_key_down(pycro.KEY.UP):
        player.y -= player.speed * dt


def draw_player(player: Player) -> None:
    pycro.draw_circle((player.x, player.y), 22.0, (0.25, 0.82, 0.98, 1.0))
    pycro.draw_text(player.name, (player.x - 28.0, player.y - 28.0), 22.0, (1.0, 1.0, 1.0, 1.0))
