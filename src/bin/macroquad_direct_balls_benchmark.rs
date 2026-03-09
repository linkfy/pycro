//! Direct Macroquad benchmark for 25k bouncing balls with wall-clock FPS sampling.

use std::env;
use std::time::Instant;

use macroquad::prelude::*;

const SCREEN_WIDTH: i32 = 1280;
const SCREEN_HEIGHT: i32 = 720;
const TARGET_FPS: f64 = 60.0;
const FPS_STABLE_RATIO: f64 = 0.95;
const DEFAULT_BALLS: usize = 25_000;
const DEFAULT_SESSION_SECONDS: f64 = 3.0;
const MAX_SIM_DT: f32 = 1.0 / 20.0;
const BALL_MIN_RADIUS: f32 = 8.0;
const BALL_MAX_RADIUS: f32 = 18.0;
const BALL_MIN_SPEED: f32 = 120.0;
const BALL_MAX_SPEED: f32 = 320.0;
const RUNTIME_NAME: &str = "macroquad_direct";

struct Lcg {
    state: u32,
}

impl Lcg {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    fn next_unit(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        self.state as f32 / u32::MAX as f32
    }

    fn range(&mut self, min: f32, max: f32) -> f32 {
        min + ((max - min) * self.next_unit())
    }

    fn sign(&mut self) -> f32 {
        if self.next_unit() < 0.5 { -1.0 } else { 1.0 }
    }
}

#[derive(Clone, Copy)]
struct Ball {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    radius: f32,
    color: Color,
}

fn parse_env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(default)
}

fn parse_env_f64(name: &str, default: f64) -> f64 {
    env::var(name)
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(default)
}

fn make_ball(rng: &mut Lcg) -> Ball {
    let radius = rng.range(BALL_MIN_RADIUS, BALL_MAX_RADIUS);
    let x = rng.range(radius, (SCREEN_WIDTH as f32) - radius);
    let y = rng.range(radius, (SCREEN_HEIGHT as f32) - radius);
    let vx = rng.sign() * rng.range(BALL_MIN_SPEED, BALL_MAX_SPEED);
    let vy = rng.sign() * rng.range(BALL_MIN_SPEED, BALL_MAX_SPEED);
    let color = Color::new(
        rng.range(40.0 / 255.0, 1.0),
        rng.range(40.0 / 255.0, 1.0),
        rng.range(40.0 / 255.0, 1.0),
        1.0,
    );
    Ball {
        x,
        y,
        vx,
        vy,
        radius,
        color,
    }
}

fn update_ball(ball: &mut Ball, sim_dt: f32) {
    ball.x += ball.vx * sim_dt;
    ball.y += ball.vy * sim_dt;

    if ball.x <= ball.radius {
        ball.x = ball.radius;
        ball.vx = ball.vx.abs();
    } else if ball.x >= (SCREEN_WIDTH as f32) - ball.radius {
        ball.x = (SCREEN_WIDTH as f32) - ball.radius;
        ball.vx = -ball.vx.abs();
    }

    if ball.y <= ball.radius {
        ball.y = ball.radius;
        ball.vy = ball.vy.abs();
    } else if ball.y >= (SCREEN_HEIGHT as f32) - ball.radius {
        ball.y = (SCREEN_HEIGHT as f32) - ball.radius;
        ball.vy = -ball.vy.abs();
    }
}

fn log_event(event: &str, fields: &[(&str, String)]) {
    print!("[benchmark] runtime={RUNTIME_NAME} event={event}");
    for (key, value) in fields {
        print!(" {key}={value}");
    }
    println!();
}

fn window_conf() -> Conf {
    Conf {
        window_title: "macroquad direct balls benchmark".to_owned(),
        window_width: SCREEN_WIDTH,
        window_height: SCREEN_HEIGHT,
        high_dpi: false,
        sample_count: 1,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let ball_count = parse_env_usize("BENCHMARK_BALLS", DEFAULT_BALLS).max(1);
    let session_seconds =
        parse_env_f64("BENCHMARK_AUTO_SESSION_SECONDS", DEFAULT_SESSION_SECONDS).max(0.1);
    let stable_threshold = TARGET_FPS * FPS_STABLE_RATIO;

    let mut rng = Lcg::new(0xA5A5_A5A5);
    let mut balls = Vec::with_capacity(ball_count);
    for _ in 0..ball_count {
        balls.push(make_ball(&mut rng));
    }

    log_event(
        "session_start",
        &[
            ("balls", ball_count.to_string()),
            ("target_fps", format!("{TARGET_FPS:.0}")),
            ("session_seconds", format!("{session_seconds:.2}")),
            ("sim_dt_cap", format!("{MAX_SIM_DT:.4}")),
        ],
    );

    let wall_start = Instant::now();
    let mut previous_wall = Instant::now();

    let mut sample_index: u64 = 0;
    let mut sample_frames: u64 = 0;
    let mut sample_wall_seconds = 0.0_f64;
    let mut sample_sim_seconds = 0.0_f64;

    let mut total_frames: u64 = 0;
    let mut total_wall_seconds = 0.0_f64;
    let mut total_sim_seconds = 0.0_f64;
    let mut stable_sample_seconds: u64 = 0;
    let mut best_stable_balls = 0_usize;

    loop {
        let sim_dt = get_frame_time().min(MAX_SIM_DT);
        for ball in &mut balls {
            update_ball(ball, sim_dt);
        }

        clear_background(Color::new(10.0 / 255.0, 16.0 / 255.0, 28.0 / 255.0, 1.0));
        for ball in &balls {
            draw_circle(ball.x, ball.y, ball.radius, ball.color);
        }

        next_frame().await;

        let now = Instant::now();
        let wall_dt = now.duration_since(previous_wall).as_secs_f64();
        previous_wall = now;

        sample_frames += 1;
        sample_wall_seconds += wall_dt;
        sample_sim_seconds += sim_dt as f64;
        total_frames += 1;
        total_wall_seconds += wall_dt;
        total_sim_seconds += sim_dt as f64;

        if sample_wall_seconds >= 1.0 {
            sample_index += 1;
            let wall_fps = sample_frames as f64 / sample_wall_seconds.max(1e-6);
            let sim_fps = sample_frames as f64 / sample_sim_seconds.max(1e-6);
            let stable = wall_fps >= stable_threshold;
            if stable {
                stable_sample_seconds += 1;
                best_stable_balls = ball_count;
            }

            log_event(
                "sample",
                &[
                    ("second", sample_index.to_string()),
                    ("balls", ball_count.to_string()),
                    ("wall_fps", format!("{wall_fps:.2}")),
                    ("sim_fps", format!("{sim_fps:.2}")),
                    ("threshold", format!("{stable_threshold:.2}")),
                    (
                        "status",
                        if stable { "stable" } else { "unstable" }.to_owned(),
                    ),
                    ("best_stable_balls", best_stable_balls.to_string()),
                    ("sim_dt_cap", format!("{MAX_SIM_DT:.4}")),
                ],
            );

            sample_frames = 0;
            sample_wall_seconds = 0.0;
            sample_sim_seconds = 0.0;
        }

        if wall_start.elapsed().as_secs_f64() >= session_seconds {
            break;
        }
    }

    let wall_fps = total_frames as f64 / total_wall_seconds.max(1e-6);
    let sim_fps = total_frames as f64 / total_sim_seconds.max(1e-6);
    log_event(
        "summary",
        &[
            ("reason", "auto_session_timeout".to_owned()),
            ("balls", ball_count.to_string()),
            ("elapsed", format!("{total_wall_seconds:.2}")),
            ("samples", sample_index.to_string()),
            ("stable_sample_seconds", stable_sample_seconds.to_string()),
            ("best_stable_balls", best_stable_balls.to_string()),
            ("wall_fps", format!("{wall_fps:.2}")),
            ("sim_fps", format!("{sim_fps:.2}")),
            ("sim_dt_cap", format!("{MAX_SIM_DT:.4}")),
        ],
    );
}
