#!/usr/bin/env python3
from __future__ import annotations

import os
import random
import sys

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
BALL_STEP = 500
MIN_BALLS = 1
MAX_BALLS = 56000
KEY_REPEAT_INITIAL_DELAY = 0.18
KEY_REPEAT_INTERVAL = 0.06
MAX_SIM_DT = 1.0 / 20.0
HUD_REFRESH_SECONDS = 0.20

AUTO_ENABLED = os.getenv("BENCHMARK_AUTO", "0") == "1"
AUTO_INITIAL_BALLS = int(os.getenv("BENCHMARK_AUTO_INITIAL_BALLS", "3000"))
AUTO_TARGETS = tuple(
    int(token.strip())
    for token in os.getenv("BENCHMARK_AUTO_TARGETS", "3000,4000").split(",")
    if token.strip()
)
AUTO_STEP_INTERVAL_SECONDS = float(os.getenv("BENCHMARK_AUTO_STEP_INTERVAL", "0.08"))
AUTO_HOLD_SECONDS = float(os.getenv("BENCHMARK_AUTO_HOLD_SECONDS", "2.5"))
AUTO_SESSION_SECONDS = float(os.getenv("BENCHMARK_AUTO_SESSION_SECONDS", "3"))

RUNTIME_NAME = "pygame"
STABLE_FPS_THRESHOLD = float(TARGET_FPS) * FPS_STABLE_RATIO


class AutoDriver:
    def __init__(self) -> None:
        self.stage_index = 0
        self.hold_remaining = 0.0
        self.step_timer = 0.0


class BallState:
    def __init__(self) -> None:
        self.x: list[float] = []
        self.y: list[float] = []
        self.vx: list[float] = []
        self.vy: list[float] = []
        self.radius: list[int] = []
        self.color: list[tuple[int, int, int]] = []

    def count(self) -> int:
        return len(self.x)


def _log_event(event: str, **fields: object) -> None:
    details = " ".join(f"{key}={value}" for key, value in fields.items())
    print(f"[benchmark] runtime={RUNTIME_NAME} event={event} {details}".rstrip())


def _append_ball(state: BallState) -> None:
    radius = random.randint(BALL_MIN_RADIUS, BALL_MAX_RADIUS)
    state.radius.append(radius)
    state.x.append(random.uniform(radius, SCREEN_WIDTH - radius))
    state.y.append(random.uniform(radius, SCREEN_HEIGHT - radius))
    state.vx.append(random.choice((-1.0, 1.0)) * random.uniform(BALL_MIN_SPEED, BALL_MAX_SPEED))
    state.vy.append(random.choice((-1.0, 1.0)) * random.uniform(BALL_MIN_SPEED, BALL_MAX_SPEED))
    state.color.append(
        (
            random.randint(40, 255),
            random.randint(40, 255),
            random.randint(40, 255),
        )
    )


def _remove_last_ball(state: BallState) -> None:
    state.x.pop()
    state.y.pop()
    state.vx.pop()
    state.vy.pop()
    state.radius.pop()
    state.color.pop()


def _set_ball_count(state: BallState, target_count: int) -> None:
    clamped = max(MIN_BALLS, min(MAX_BALLS, target_count))
    while state.count() < clamped:
        _append_ball(state)
    while state.count() > clamped:
        _remove_last_ball(state)


def _update_balls(state: BallState, sim_dt: float) -> None:
    count = state.count()
    for i in range(count):
        x = state.x[i] + (state.vx[i] * sim_dt)
        y = state.y[i] + (state.vy[i] * sim_dt)
        radius = state.radius[i]
        vx = state.vx[i]
        vy = state.vy[i]

        if x <= radius:
            x = float(radius)
            vx = abs(vx)
        elif x >= SCREEN_WIDTH - radius:
            x = float(SCREEN_WIDTH - radius)
            vx = -abs(vx)

        if y <= radius:
            y = float(radius)
            vy = abs(vy)
        elif y >= SCREEN_HEIGHT - radius:
            y = float(SCREEN_HEIGHT - radius)
            vy = -abs(vy)

        state.x[i] = x
        state.y[i] = y
        state.vx[i] = vx
        state.vy[i] = vy


def _record_nearest_targets(
    targets: tuple[int, ...], nearest_for_target: dict[int, int], current: int
) -> bool:
    changed = False
    for target in targets:
        previous = nearest_for_target.get(target)
        if previous is None or abs(current - target) < abs(previous - target):
            nearest_for_target[target] = current
            changed = True
    return changed


def _nearest_summary(targets: tuple[int, ...], nearest_for_target: dict[int, int], current: int) -> str:
    parts: list[str] = []
    for target in targets:
        reached = nearest_for_target.get(target, current)
        delta = abs(reached - target)
        parts.append(f"{target}:{reached}(delta={delta})")
    return ",".join(parts)


def _auto_drive(state: BallState, sim_dt: float, driver: AutoDriver) -> None:
    if not AUTO_ENABLED or not AUTO_TARGETS:
        return
    if driver.stage_index >= len(AUTO_TARGETS):
        return

    target = AUTO_TARGETS[driver.stage_index]
    current = state.count()
    if current == target:
        if driver.hold_remaining <= 0.0:
            driver.hold_remaining = AUTO_HOLD_SECONDS
            _log_event("auto_target_reached", target=target, balls=current)
        driver.hold_remaining -= sim_dt
        if driver.hold_remaining <= 0.0:
            driver.stage_index += 1
        return

    driver.step_timer -= sim_dt
    if driver.step_timer > 0.0:
        return

    driver.step_timer = AUTO_STEP_INTERVAL_SECONDS
    if current < target:
        _set_ball_count(state, current + BALL_STEP)
    else:
        _set_ball_count(state, current - BALL_STEP)


def _build_hud_lines(
    fps_display: int,
    balls_count: int,
    best_balls: int,
    stable_sample_seconds: int,
    nearest_targets: str,
) -> list[str]:
    status = "stable" if fps_display >= int(round(STABLE_FPS_THRESHOLD)) else "unstable"
    return [
        "pygame balls benchmark (Left/Right: -/+ 50 balls, Esc: exit)",
        f"fps={fps_display} status={status}",
        f"balls={balls_count}",
        f"target_fps={TARGET_FPS}",
        f"best_stable_balls={best_balls}",
        f"stable_seconds={stable_sample_seconds}",
        f"nearest_targets={nearest_targets}",
    ]


def _render_hud_surfaces(font: pygame.font.Font, lines: list[str]) -> list[tuple[pygame.Surface, tuple[int, int]]]:
    rendered: list[tuple[pygame.Surface, tuple[int, int]]] = []
    for i, line in enumerate(lines):
        color = HUD_ACCENT if i == 0 else TEXT_COLOR
        rendered.append((font.render(line, True, color), (20, 20 + i * 28)))
    return rendered


def _draw_hud(
    surface: pygame.Surface,
    hud_backdrop: pygame.Surface,
    rendered_lines: list[tuple[pygame.Surface, tuple[int, int]]],
) -> None:
    surface.blit(hud_backdrop, (10, 10))
    for text_surface, pos in rendered_lines:
        surface.blit(text_surface, pos)


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

    state = BallState()
    session_initial_balls = (
        max(MIN_BALLS, min(MAX_BALLS, AUTO_INITIAL_BALLS)) if AUTO_ENABLED else INITIAL_BALLS
    )
    _set_ball_count(state, session_initial_balls)

    wall_sample_time = 0.0
    sim_sample_time = 0.0
    sample_frames = 0
    fps_display = TARGET_FPS
    best_balls_no_drop = state.count()
    sample_index = 0
    stable_sample_seconds = 0
    elapsed_wall_seconds = 0.0

    nearest_for_target: dict[int, int] = {}
    _record_nearest_targets(AUTO_TARGETS, nearest_for_target, state.count())
    nearest_summary = _nearest_summary(AUTO_TARGETS, nearest_for_target, state.count())

    auto_driver = AutoDriver()

    hud_refresh_in = 0.0
    hud_lines = _build_hud_lines(
        fps_display,
        state.count(),
        best_balls_no_drop,
        stable_sample_seconds,
        nearest_summary,
    )
    rendered_hud_lines = _render_hud_surfaces(font, hud_lines)

    hud_backdrop = pygame.Surface((700, 196), pygame.SRCALPHA)
    hud_backdrop.fill((0, 0, 0, 185))

    _log_event(
        "session_start",
        target_fps=TARGET_FPS,
        stable_threshold=f"{STABLE_FPS_THRESHOLD:.2f}",
        initial_balls=session_initial_balls,
        auto_mode=int(AUTO_ENABLED),
        auto_targets=",".join(str(v) for v in AUTO_TARGETS) if AUTO_TARGETS else "none",
        metric="wall_fps",
        sim_dt_cap=f"{MAX_SIM_DT:.4f}",
    )

    running = True
    while running:
        wall_dt = clock.tick(TARGET_FPS) / 1000.0
        sim_dt = min(wall_dt, MAX_SIM_DT)
        elapsed_wall_seconds += wall_dt

        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_ESCAPE:
                    running = False
                elif event.key == pygame.K_LEFT:
                    _set_ball_count(state, state.count() - BALL_STEP)
                    hud_refresh_in = 0.0
                elif event.key == pygame.K_RIGHT:
                    _set_ball_count(state, state.count() + BALL_STEP)
                    hud_refresh_in = 0.0

        _auto_drive(state, sim_dt, auto_driver)
        if _record_nearest_targets(AUTO_TARGETS, nearest_for_target, state.count()):
            nearest_summary = _nearest_summary(AUTO_TARGETS, nearest_for_target, state.count())
            hud_refresh_in = 0.0

        _update_balls(state, sim_dt)

        wall_sample_time += wall_dt
        sim_sample_time += sim_dt
        sample_frames += 1
        if wall_sample_time >= 1.0:
            wall_fps = float(sample_frames) / max(wall_sample_time, 1e-6)
            sim_fps = float(sample_frames) / max(sim_sample_time, 1e-6)
            fps_display = int(round(wall_fps))
            stability = "stable" if wall_fps >= STABLE_FPS_THRESHOLD else "unstable"
            if stability == "stable":
                stable_sample_seconds += 1
                best_balls_no_drop = max(best_balls_no_drop, state.count())
            sample_index += 1
            _log_event(
                "sample",
                second=sample_index,
                balls=state.count(),
                wall_fps=f"{wall_fps:.2f}",
                sim_fps=f"{sim_fps:.2f}",
                threshold=f"{STABLE_FPS_THRESHOLD:.2f}",
                status=stability,
                best_stable_balls=best_balls_no_drop,
                stable_sample_seconds=stable_sample_seconds,
                nearest_targets=nearest_summary,
                sim_dt_cap=f"{MAX_SIM_DT:.4f}",
            )
            wall_sample_time = 0.0
            sim_sample_time = 0.0
            sample_frames = 0
            hud_refresh_in = 0.0

        hud_refresh_in -= wall_dt
        if hud_refresh_in <= 0.0:
            hud_lines = _build_hud_lines(
                fps_display,
                state.count(),
                best_balls_no_drop,
                stable_sample_seconds,
                nearest_summary,
            )
            rendered_hud_lines = _render_hud_surfaces(font, hud_lines)
            hud_refresh_in = HUD_REFRESH_SECONDS

        screen.fill(BACKGROUND)
        for i in range(state.count()):
            pygame.draw.circle(screen, state.color[i], (int(state.x[i]), int(state.y[i])), state.radius[i])

        _draw_hud(screen, hud_backdrop, rendered_hud_lines)
        pygame.display.flip()

        if AUTO_ENABLED and AUTO_SESSION_SECONDS > 0.0 and elapsed_wall_seconds >= AUTO_SESSION_SECONDS:
            running = False

    wall_fps = float(sample_frames) / max(wall_sample_time, 1e-6)
    sim_fps = float(sample_frames) / max(sim_sample_time, 1e-6)
    _log_event(
        "summary",
        reason=("auto_session_timeout" if AUTO_ENABLED and AUTO_SESSION_SECONDS > 0.0 else "exit"),
        best_stable_balls=best_balls_no_drop,
        stable_sample_seconds=stable_sample_seconds,
        nearest_targets=nearest_summary,
        elapsed=f"{elapsed_wall_seconds:.2f}",
        samples=sample_index,
        wall_fps=f"{wall_fps:.2f}",
        sim_fps=f"{sim_fps:.2f}",
        sim_dt_cap=f"{MAX_SIM_DT:.4f}",
    )

    pygame.quit()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
