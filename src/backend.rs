//! Backend contract used by the API layer.
//! The first backend owner is Macroquad, but the contract is backend-agnostic.

use macroquad::input::{KeyCode, is_key_down, is_quit_requested};
use macroquad::math::vec2;
use macroquad::prelude::{
    Camera2D, Color as MqColor, DrawTextureParams, Rect, Texture2D, WHITE, clear_background,
    draw_circle, draw_rectangle, draw_texture_ex, screen_height, screen_width, set_camera,
};
use macroquad::time::get_frame_time;
use macroquad::window::{Conf, next_frame};
use std::collections::HashMap;
use std::env;
use std::path::Path;

/// A two-dimensional vector.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    /// Horizontal coordinate.
    pub x: f32,
    /// Vertical coordinate.
    pub y: f32,
}

/// A normalized RGBA color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    /// Red channel.
    pub r: f32,
    /// Green channel.
    pub g: f32,
    /// Blue channel.
    pub b: f32,
    /// Alpha channel.
    pub a: f32,
}

impl From<Color> for MqColor {
    fn from(value: Color) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }
}

/// An opaque texture handle type owned by the backend implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureHandle(pub String);

/// A single backend dispatch emitted from the Python API surface.
#[derive(Clone, Debug, PartialEq)]
pub enum BackendDispatch {
    /// clear_background(color)
    ClearBackground(Color),
    /// draw_circle(position, radius, color)
    DrawCircle {
        /// Center point.
        position: Vec2,
        /// Radius.
        radius: f32,
        /// Fill color.
        color: Color,
    },
    /// load_texture(path)
    LoadTexture(String),
    /// draw_texture(texture, position, size)
    DrawTexture {
        /// Texture handle.
        texture: TextureHandle,
        /// Position.
        position: Vec2,
        /// Size.
        size: Vec2,
    },
    /// set_camera_target(target)
    SetCameraTarget(Vec2),
}

/// Engine backend contract consumed by `api`.
pub trait EngineBackend {
    /// Clears the current frame to a color.
    fn clear_background(&mut self, color: Color);
    /// Draws a circle.
    fn draw_circle(&mut self, position: Vec2, radius: f32, color: Color);
    /// Returns whether a key is currently down.
    fn is_key_down(&self, key: &str) -> bool;
    /// Returns the current frame delta time.
    fn frame_time(&self) -> f32;
    /// Loads a texture asset.
    fn load_texture(&mut self, path: &str) -> Result<TextureHandle, String>;
    /// Draws a texture.
    fn draw_texture(&mut self, texture: &TextureHandle, position: Vec2, size: Vec2);
    /// Moves the active camera target.
    fn set_camera_target(&mut self, target: Vec2);
}

/// Desktop loop owner config for frame dispatch.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameLoopConfig {
    /// Optional fixed timestep in seconds. If omitted, uses Macroquad frame time.
    pub fixed_dt_seconds: Option<f32>,
    /// Optional frame budget. If omitted, loop runs until the window closes.
    pub frame_count: Option<usize>,
}

impl FrameLoopConfig {
    /// Parses loop options from environment.
    #[must_use]
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(value) = env::var("PYCRO_FRAME_DT")
            && let Ok(parsed) = value.parse::<f32>()
        {
            config.fixed_dt_seconds = Some(parsed.max(0.0));
        }

        if let Ok(value) = env::var("PYCRO_FRAMES")
            && let Ok(parsed) = value.parse::<usize>()
        {
            config.frame_count = Some(parsed.max(1));
        }

        config
    }
}

/// First desktop frame-loop owner. Macroquad owns this loop.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DesktopFrameLoop {
    config: FrameLoopConfig,
}

/// Loop execution report for smoke validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DesktopLoopReport {
    /// Number of frames dispatched.
    pub frames_executed: usize,
}

impl DesktopFrameLoop {
    /// Creates a desktop loop owner with explicit config.
    #[must_use]
    pub const fn new(config: FrameLoopConfig) -> Self {
        Self { config }
    }

    /// Runs the frame callback using Macroquad's real window/event loop.
    pub async fn run(
        &self,
        mut on_frame: impl FnMut(f32) -> Result<(), String>,
    ) -> Result<DesktopLoopReport, String> {
        let mut frames_executed = 0usize;
        loop {
            let dt = self.config.fixed_dt_seconds.unwrap_or_else(get_frame_time);
            on_frame(dt)?;
            frames_executed += 1;

            next_frame().await;

            let reached_budget = self
                .config
                .frame_count
                .is_some_and(|budget| frames_executed >= budget);
            if reached_budget || is_quit_requested() {
                break;
            }
        }

        Ok(DesktopLoopReport { frames_executed })
    }
}

/// Window configuration used by the real Macroquad loop owner.
#[must_use]
pub fn window_conf() -> Conf {
    Conf {
        window_title: "pycro".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Conf::default()
    }
}

/// First backend implementation behind the contract boundary.
#[derive(Debug)]
pub struct MacroquadBackendContract {
    frame_time: f32,
    dispatch_log: Vec<BackendDispatch>,
    textures: HashMap<String, Texture2D>,
}

impl Default for MacroquadBackendContract {
    fn default() -> Self {
        Self {
            frame_time: 0.016,
            dispatch_log: Vec::new(),
            textures: HashMap::new(),
        }
    }
}

impl MacroquadBackendContract {
    /// Updates per-frame timing consumed by API timing helpers.
    pub fn set_frame_time(&mut self, dt: f32) {
        self.frame_time = dt;
    }

    /// Returns collected dispatch operations.
    #[must_use]
    pub fn dispatch_log(&self) -> &[BackendDispatch] {
        &self.dispatch_log
    }
}

fn key_code_from_name(key: &str) -> Option<KeyCode> {
    match key {
        "Space" | "space" => Some(KeyCode::Space),
        "Left" | "left" => Some(KeyCode::Left),
        "Right" | "right" => Some(KeyCode::Right),
        "Up" | "up" => Some(KeyCode::Up),
        "Down" | "down" => Some(KeyCode::Down),
        "Escape" | "escape" => Some(KeyCode::Escape),
        _ => None,
    }
}

impl EngineBackend for MacroquadBackendContract {
    fn clear_background(&mut self, color: Color) {
        self.dispatch_log
            .push(BackendDispatch::ClearBackground(color));
        clear_background(color.into());
    }

    fn draw_circle(&mut self, position: Vec2, radius: f32, color: Color) {
        self.dispatch_log.push(BackendDispatch::DrawCircle {
            position,
            radius,
            color,
        });
        draw_circle(position.x, position.y, radius, color.into());
    }

    fn is_key_down(&self, key: &str) -> bool {
        key_code_from_name(key).is_some_and(is_key_down)
    }

    fn frame_time(&self) -> f32 {
        self.frame_time
    }

    fn load_texture(&mut self, path: &str) -> Result<TextureHandle, String> {
        self.dispatch_log
            .push(BackendDispatch::LoadTexture(path.to_owned()));

        let handle = TextureHandle(path.to_owned());
        let resolved = Path::new(path);
        if let Ok(bytes) = std::fs::read(resolved) {
            let texture = Texture2D::from_file_with_format(&bytes, None);
            self.textures.insert(handle.0.clone(), texture);
        }

        Ok(handle)
    }

    fn draw_texture(&mut self, texture: &TextureHandle, position: Vec2, size: Vec2) {
        self.dispatch_log.push(BackendDispatch::DrawTexture {
            texture: texture.clone(),
            position,
            size,
        });

        if let Some(native_texture) = self.textures.get(&texture.0) {
            draw_texture_ex(
                native_texture,
                position.x,
                position.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(size.x, size.y)),
                    ..DrawTextureParams::default()
                },
            );
            return;
        }

        // Fallback marker when texture file cannot be loaded.
        draw_rectangle(position.x, position.y, size.x, size.y, WHITE);
    }

    fn set_camera_target(&mut self, target: Vec2) {
        self.dispatch_log
            .push(BackendDispatch::SetCameraTarget(target));

        let width = screen_width();
        let height = screen_height();
        let display_rect = Rect::new(
            target.x - (width * 0.5),
            target.y - (height * 0.5),
            width,
            height,
        );
        set_camera(&Camera2D::from_display_rect(display_rect));
    }
}
