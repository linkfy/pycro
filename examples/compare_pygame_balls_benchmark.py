#!/usr/bin/env python3
from __future__ import annotations

import random
import sys
from dataclasses import dataclass

try:
    import pygame
except ImportError:
    print(
        "pygame no está instalado. Instálalo con: python3 -m pip install pygame",
        file=sys.stderr,
    )
    raise SystemExit(1)

SCREEN_WIDTH = 1280
SCREEN_HEIGHT = 720
TARGET_FPS = 60
FPS_STABLE_RATIO = 0.95
BACKGROUND = (10, 16, 28)
TEXT_COLOR = (240, 245, 255)
HUD_ACCENT = (252, 210, 115)
BALL_MIN_RADIUS = 8
BALL_MAX_RADIUS = 18
BALL_MIN_SPEED = 120.0
BALL_MAX_SPEED = 320.0
INITIAL_BALLS = 500
BALL_STEP = 50
KEY_REPEAT_INITIAL_DELAY = 0.18
KEY_REPEAT_INTERVAL = 0.06


@dataclass
class Ball:
    x: float
    y: float
    vx: float
    vy: float
    radius: int
    color: tuple[int, int, int]


def make_ball() -> Ball:
    radius = random.randint(BALL_MIN_RADIUS, BALL_MAX_RADIUS)
    x = random.uniform(radius, SCREEN_WIDTH - radius)
    y = random.uniform(radius, SCREEN_HEIGHT - radius)
    vx = random.choice((-1.0, 1.0)) * random.uniform(BALL_MIN_SPEED, BALL_MAX_SPEED)
    vy = random.choice((-1.0, 1.0)) * random.uniform(BALL_MIN_SPEED, BALL_MAX_SPEED)
    color = (
        random.randint(40, 255),
        random.randint(40, 255),
        random.randint(40, 255),
    )
    return Ball(x=x, y=y, vx=vx, vy=vy, radius=radius, color=color)


def update_ball(ball: Ball, dt: float) -> None:
    ball.x += ball.vx * dt
    ball.y += ball.vy * dt

    if ball.x <= ball.radius:
        ball.x = float(ball.radius)
        ball.vx *= -1.0
    elif ball.x >= SCREEN_WIDTH - ball.radius:
        ball.x = float(SCREEN_WIDTH - ball.radius)
        ball.vx *= -1.0

    if ball.y <= ball.radius:
        ball.y = float(ball.radius)
        ball.vy *= -1.0
    elif ball.y >= SCREEN_HEIGHT - ball.radius:
        ball.y = float(SCREEN_HEIGHT - ball.radius)
        ball.vy *= -1.0


def draw_hud(
    surface: pygame.Surface,
    font: pygame.font.Font,
    fps_display: int,
    balls_count: int,
    best_balls: int,
) -> None:
    lines = [
        "pygame balls benchmark (Left/Right: -/+ 50 bolas, Esc: salir)",
        f"fps={fps_display}",
        f"balls={balls_count}",
        f"target_fps={TARGET_FPS}",
        f"best_balls_no_drop={best_balls}",
    ]

    for i, line in enumerate(lines):
        color = HUD_ACCENT if i == 0 else TEXT_COLOR
        text = font.render(line, True, color)
        surface.blit(text, (20, 20 + i * 28))


def main() -> int:
    pygame.init()
    pygame.display.set_caption("pygame balls benchmark")
    screen = pygame.display.set_mode((SCREEN_WIDTH, SCREEN_HEIGHT))
    clock = pygame.time.Clock()
    font = pygame.font.Font(pygame.font.get_default_font(), 30)
    pygame.key.set_repeat(
        int(KEY_REPEAT_INITIAL_DELAY * 1000.0),
        int(KEY_REPEAT_INTERVAL * 1000.0),
    )

    balls = [make_ball() for _ in range(INITIAL_BALLS)]
    fps_timer = 0.0
    fps_frames = 0
    fps_display = TARGET_FPS
    best_balls_no_drop = len(balls)

    running = True
    while running:
        dt = clock.tick(TARGET_FPS) / 1000.0

        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_ESCAPE:
                    running = False
                elif event.key == pygame.K_LEFT:
                    remove_count = min(BALL_STEP, len(balls))
                    if remove_count > 0:
                        del balls[-remove_count:]
                elif event.key == pygame.K_RIGHT:
                    balls.extend(make_ball() for _ in range(BALL_STEP))

        fps_timer += dt
        fps_frames += 1
        if fps_timer >= 1.0:
            fps_display = int(round(fps_frames / max(fps_timer, 1e-6)))
            if fps_display >= int(TARGET_FPS * FPS_STABLE_RATIO):
                best_balls_no_drop = max(best_balls_no_drop, len(balls))
            fps_timer = 0.0
            fps_frames = 0

        for ball in balls:
            update_ball(ball, dt)

        screen.fill(BACKGROUND)
        for ball in balls:
            pygame.draw.circle(screen, ball.color, (int(ball.x), int(ball.y)), ball.radius)

        draw_hud(screen, font, fps_display, len(balls), best_balls_no_drop)
        pygame.display.flip()

    pygame.quit()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
