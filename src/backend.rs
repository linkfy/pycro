//! Backend contract used by the API layer.
//! The first backend owner is Macroquad, but the contract is backend-agnostic.

use macroquad::input::{KeyCode, is_key_down, is_quit_requested};
use macroquad::math::vec2;
use macroquad::miniquad::conf::AppleGfxApi;
use macroquad::prelude::{
    Camera2D, Color as MqColor, DrawTextureParams, Rect, Texture2D, WHITE, clear_background,
    draw_circle, draw_rectangle, draw_text, draw_texture_ex, screen_height, screen_width,
    set_camera,
};
use macroquad::texture::FilterMode;
use macroquad::time::get_frame_time;
use macroquad::window::{Conf, gl_set_drawcall_buffer_capacity, next_frame};
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

/// Packed circle draw payload for batch dispatch.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CircleDraw {
    /// Center point.
    pub position: Vec2,
    /// Radius.
    pub radius: f32,
    /// Fill color.
    pub color: Color,
    /// Per-command render mode override.
    pub render_mode: VectorRenderMode,
}

/// Optional per-command vector rendering mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VectorRenderMode {
    /// Use backend default policy.
    Default,
    /// Force vector rendering for this draw.
    ForceVector,
    /// Force sprite rendering for this draw.
    ForceSprite,
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
    /// draw_text(text, position, font_size, color)
    DrawText {
        /// Text content.
        text: String,
        /// Baseline anchor position.
        position: Vec2,
        /// Font size in pixels.
        font_size: f32,
        /// Text color.
        color: Color,
    },
}

/// Engine backend contract consumed by `api`.
pub trait EngineBackend {
    /// Clears the current frame to a color.
    fn clear_background(&mut self, color: Color);
    /// Draws a circle.
    fn draw_circle(
        &mut self,
        position: Vec2,
        radius: f32,
        color: Color,
        render_mode: VectorRenderMode,
    );
    /// Draws many circles in a single backend call.
    fn draw_circle_batch(&mut self, circles: &[CircleDraw]) {
        for circle in circles {
            self.draw_circle(
                circle.position,
                circle.radius,
                circle.color,
                circle.render_mode,
            );
        }
    }
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
    /// Draws text on screen.
    fn draw_text(&mut self, text: &str, position: Vec2, font_size: f32, color: Color);
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
    let high_dpi = env::var("PYCRO_HIGH_DPI")
        .ok()
        .map(|value| value == "1")
        .unwrap_or(false);
    let sample_count = env::var("PYCRO_SAMPLE_COUNT")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .map(|count| count.clamp(1, 8))
        .unwrap_or(1);
    let mut conf = Conf {
        window_title: "pycro".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi,
        sample_count,
        ..Conf::default()
    };

    #[cfg(target_os = "macos")]
    {
        if std::env::consts::ARCH == "aarch64" {
            let selected = env::var("PYCRO_APPLE_GFX_API")
                .ok()
                .map(|value| value.to_ascii_lowercase());
            conf.platform.apple_gfx_api = match selected.as_deref() {
                Some("metal") => AppleGfxApi::Metal,
                Some("opengl") | Some("gl") => AppleGfxApi::OpenGl,
                _ => AppleGfxApi::OpenGl,
            };
        }
    }

    conf
}

/// First backend implementation behind the contract boundary.
#[derive(Debug)]
pub struct MacroquadBackendContract {
    frame_time: f32,
    dispatch_count: usize,
    count_dispatches: bool,
    #[cfg(test)]
    dispatch_log: Vec<BackendDispatch>,
    textures: HashMap<String, Texture2D>,
    circle_sprite: Option<Texture2D>,
    circle_sprite_size: u16,
    circle_sprite_filter_linear: bool,
    drawcall_max_vertices: Option<usize>,
    drawcall_max_indices: Option<usize>,
    drawcall_capacity_applied: bool,
    use_circle_sprite: bool,
}

impl Default for MacroquadBackendContract {
    fn default() -> Self {
        Self {
            frame_time: 0.016,
            dispatch_count: 0,
            count_dispatches: {
                #[cfg(test)]
                {
                    true
                }
                #[cfg(not(test))]
                {
                    env::var("PYCRO_COUNT_DISPATCHES")
                        .map(|value| value == "1")
                        .unwrap_or(false)
                }
            },
            #[cfg(test)]
            dispatch_log: Vec::new(),
            textures: HashMap::new(),
            circle_sprite: None,
            circle_sprite_size: {
                #[cfg(test)]
                {
                    256
                }
                #[cfg(not(test))]
                {
                    env::var("PYCRO_CIRCLE_SPRITE_SIZE")
                        .ok()
                        .and_then(|value| value.parse::<u16>().ok())
                        .map(|size| size.clamp(64, 2048))
                        .unwrap_or(512)
                }
            },
            circle_sprite_filter_linear: {
                #[cfg(test)]
                {
                    true
                }
                #[cfg(not(test))]
                {
                    !matches!(
                        env::var("PYCRO_CIRCLE_SPRITE_FILTER")
                            .ok()
                            .map(|value| value.to_ascii_lowercase())
                            .as_deref(),
                        Some("nearest")
                    )
                }
            },
            drawcall_max_vertices: {
                #[cfg(test)]
                {
                    None
                }
                #[cfg(not(test))]
                {
                    env::var("PYCRO_DRAWCALL_MAX_VERTICES")
                        .ok()
                        .and_then(|value| value.parse::<usize>().ok())
                        .filter(|value| *value >= 1024)
                }
            },
            drawcall_max_indices: {
                #[cfg(test)]
                {
                    None
                }
                #[cfg(not(test))]
                {
                    env::var("PYCRO_DRAWCALL_MAX_INDICES")
                        .ok()
                        .and_then(|value| value.parse::<usize>().ok())
                        .filter(|value| *value >= 1536)
                }
            },
            drawcall_capacity_applied: false,
            use_circle_sprite: {
                #[cfg(test)]
                {
                    false
                }
                #[cfg(not(test))]
                {
                    env::var("PYCRO_CIRCLE_SPRITE")
                        .map(|value| value == "1")
                        .unwrap_or(false)
                }
            },
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
        #[cfg(test)]
        {
            &self.dispatch_log
        }
        #[cfg(not(test))]
        {
            &[]
        }
    }

    /// Returns total dispatched backend operations.
    #[must_use]
    pub const fn dispatch_count(&self) -> usize {
        self.dispatch_count
    }

    fn record_dispatch(&mut self) {
        if self.count_dispatches {
            self.dispatch_count += 1;
        }
    }

    fn ensure_circle_sprite(&mut self) -> Option<Texture2D> {
        if let Some(texture) = self.circle_sprite.as_ref() {
            return Some(texture.clone());
        }

        let size = usize::from(self.circle_sprite_size.max(64));
        let mut bytes = vec![0u8; size * size * 4];
        let center = (size as f32 - 1.0) * 0.5;
        let radius = center.max(1.0);
        let edge_softness = (size as f32 * 0.01).max(1.0);
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let distance = (dx * dx + dy * dy).sqrt();
                let offset = (y * size + x) * 4;
                bytes[offset] = 255;
                bytes[offset + 1] = 255;
                bytes[offset + 2] = 255;
                let alpha = ((radius - distance) / edge_softness).clamp(0.0, 1.0);
                bytes[offset + 3] = (alpha * 255.0) as u8;
            }
        }

        let texture = Texture2D::from_rgba8(size as u16, size as u16, &bytes);
        texture.set_filter(if self.circle_sprite_filter_linear {
            FilterMode::Linear
        } else {
            FilterMode::Nearest
        });
        self.circle_sprite = Some(texture.clone());
        Some(texture)
    }

    fn apply_drawcall_capacity_once(&mut self) {
        if self.drawcall_capacity_applied {
            return;
        }
        let Some(max_vertices) = self.drawcall_max_vertices else {
            return;
        };
        let Some(max_indices) = self.drawcall_max_indices else {
            return;
        };
        gl_set_drawcall_buffer_capacity(max_vertices, max_indices);
        self.drawcall_capacity_applied = true;
    }

    fn should_render_circle_as_sprite(&self, render_mode: VectorRenderMode) -> bool {
        match render_mode {
            VectorRenderMode::Default => self.use_circle_sprite,
            VectorRenderMode::ForceVector => false,
            VectorRenderMode::ForceSprite => true,
        }
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
        self.apply_drawcall_capacity_once();
        self.record_dispatch();
        #[cfg(test)]
        self.dispatch_log
            .push(BackendDispatch::ClearBackground(color));
        clear_background(color.into());
    }

    fn draw_circle(
        &mut self,
        position: Vec2,
        radius: f32,
        color: Color,
        render_mode: VectorRenderMode,
    ) {
        self.record_dispatch();
        #[cfg(test)]
        self.dispatch_log.push(BackendDispatch::DrawCircle {
            position,
            radius,
            color,
        });
        if self.should_render_circle_as_sprite(render_mode)
            && let Some(sprite) = self.ensure_circle_sprite()
        {
            let diameter_f = (radius * 2.0).max(2.0);
            let half = diameter_f * 0.5;
            draw_texture_ex(
                &sprite,
                position.x - half,
                position.y - half,
                color.into(),
                DrawTextureParams {
                    dest_size: Some(vec2(diameter_f, diameter_f)),
                    ..DrawTextureParams::default()
                },
            );
            return;
        }
        draw_circle(position.x, position.y, radius, color.into());
    }

    fn draw_circle_batch(&mut self, circles: &[CircleDraw]) {
        if self.count_dispatches {
            self.dispatch_count += circles.len();
        }
        #[cfg(test)]
        for circle in circles {
            self.dispatch_log.push(BackendDispatch::DrawCircle {
                position: circle.position,
                radius: circle.radius,
                color: circle.color,
            });
        }
        if self.use_circle_sprite && let Some(sprite) = self.ensure_circle_sprite() {
            for circle in circles {
                if self.should_render_circle_as_sprite(circle.render_mode) {
                    let diameter_f = (circle.radius * 2.0).max(2.0);
                    let half = diameter_f * 0.5;
                    draw_texture_ex(
                        &sprite,
                        circle.position.x - half,
                        circle.position.y - half,
                        circle.color.into(),
                        DrawTextureParams {
                            dest_size: Some(vec2(diameter_f, diameter_f)),
                            ..DrawTextureParams::default()
                        },
                    );
                } else {
                    draw_circle(
                        circle.position.x,
                        circle.position.y,
                        circle.radius,
                        circle.color.into(),
                    );
                }
            }
            return;
        }

        for circle in circles {
            draw_circle(
                circle.position.x,
                circle.position.y,
                circle.radius,
                circle.color.into(),
            );
        }
    }

    fn is_key_down(&self, key: &str) -> bool {
        key_code_from_name(key).is_some_and(is_key_down)
    }

    fn frame_time(&self) -> f32 {
        self.frame_time
    }

    fn load_texture(&mut self, path: &str) -> Result<TextureHandle, String> {
        self.record_dispatch();
        #[cfg(test)]
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
        self.record_dispatch();
        #[cfg(test)]
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
        self.record_dispatch();
        #[cfg(test)]
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

    fn draw_text(&mut self, text: &str, position: Vec2, font_size: f32, color: Color) {
        self.record_dispatch();
        #[cfg(test)]
        self.dispatch_log.push(BackendDispatch::DrawText {
            text: text.to_owned(),
            position,
            font_size,
            color,
        });
        draw_text(text, position.x, position.y, font_size, color.into());
    }
}
