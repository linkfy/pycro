//! Backend contract used by the API layer.
//! The first backend owner is Macroquad, but the contract is backend-agnostic.

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
use crate::embedded_project_payload;
#[cfg(not(target_os = "android"))]
use macroquad::input::is_quit_requested;
use macroquad::input::{KeyCode, MouseButton};
#[cfg(not(test))]
use macroquad::input::{is_key_down, is_mouse_button_down};
use macroquad::math::vec2;
#[cfg(target_os = "macos")]
use macroquad::miniquad::conf::AppleGfxApi;
#[cfg(not(test))]
use macroquad::models::{Mesh, Vertex, draw_mesh};
use macroquad::prelude::{
    Camera2D, Color as MqColor, DrawTextureParams, Rect, Texture2D, WHITE, clear_background,
    draw_circle, draw_line as mq_draw_line, draw_rectangle, draw_text, draw_texture_ex,
    screen_height, screen_width, set_camera,
};
use macroquad::texture::FilterMode;
use macroquad::time::get_frame_time;
use macroquad::window::{Conf, gl_set_drawcall_buffer_capacity, next_frame};
use std::collections::HashMap;
use std::env;
#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
use std::path::Path;
#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "android"),
    not(target_os = "ios")
))]
use std::sync::Once;
use std::sync::atomic::{AtomicBool, Ordering};

static INTERRUPT_REQUESTED: AtomicBool = AtomicBool::new(false);
#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "android"),
    not(target_os = "ios")
))]
static INTERRUPT_HANDLER_ONCE: Once = Once::new();

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

/// Packed textured quad payload.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextureDraw {
    /// Top-left position.
    pub position: Vec2,
    /// Draw size.
    pub size: Vec2,
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
    /// draw_rectangle(position, size, color)
    DrawRectangle {
        /// Top-left position.
        position: Vec2,
        /// Rectangle size.
        size: Vec2,
        /// Fill color.
        color: Color,
    },
    /// put_pixel(position, color)
    PutPixel {
        /// Pixel position.
        position: Vec2,
        /// Pixel color.
        color: Color,
    },
    /// draw_line(start, end, color)
    DrawLine {
        /// Line start position.
        start: Vec2,
        /// Line end position.
        end: Vec2,
        /// Line color.
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
    /// Flushes deferred rendering work for the current frame.
    fn finish_frame(&mut self) {}
    /// Moves the active camera target.
    fn set_camera_target(&mut self, target: Vec2);
    /// Draws text on screen.
    fn draw_text(&mut self, text: &str, position: Vec2, font_size: f32, color: Color);
    /// Returns current window size in pixels.
    fn get_window_size(&self) -> Vec2;
    /// Returns current mouse position in pixels.
    fn get_mouse_position(&self) -> Vec2;
    /// Draws a filled rectangle.
    fn draw_rectangle(&mut self, position: Vec2, size: Vec2, color: Color);
    /// Draws a single pixel in screen space.
    fn put_pixel(&mut self, position: Vec2, color: Color);
    /// Draws a 1px line in screen space.
    fn draw_line(&mut self, start: Vec2, end: Vec2, color: Color);
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
        install_interrupt_handler();
        loop {
            if interrupt_requested() {
                return Err("runtime interrupted by keyboard interrupt (Ctrl+C)".to_owned());
            }
            let dt = self.config.fixed_dt_seconds.unwrap_or_else(get_frame_time);
            on_frame(dt)?;
            frames_executed += 1;

            next_frame().await;

            let reached_budget = self
                .config
                .frame_count
                .is_some_and(|budget| frames_executed >= budget);
            if reached_budget || should_terminate_frame_loop() {
                break;
            }
        }

        Ok(DesktopLoopReport { frames_executed })
    }
}

#[cfg(target_os = "android")]
fn should_terminate_frame_loop() -> bool {
    false
}

#[cfg(not(target_os = "android"))]
fn should_terminate_frame_loop() -> bool {
    is_quit_requested() || interrupt_requested()
}

#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "android"),
    not(target_os = "ios")
))]
fn install_interrupt_handler() {
    INTERRUPT_HANDLER_ONCE.call_once(|| {
        if let Err(error) = ctrlc::set_handler(|| {
            INTERRUPT_REQUESTED.store(true, Ordering::Release);
        }) {
            eprintln!("failed to install Ctrl+C handler: {error}");
        }
    });
}

#[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
fn install_interrupt_handler() {}

fn interrupt_requested() -> bool {
    INTERRUPT_REQUESTED.load(Ordering::Acquire)
}

fn wrap_overlay_message(message: &str, max_chars: usize, max_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for raw_line in message.lines() {
        if raw_line.is_empty() {
            lines.push(String::new());
            if lines.len() >= max_lines {
                break;
            }
            continue;
        }
        let mut current = String::new();
        for word in raw_line.split_whitespace() {
            let candidate_len = if current.is_empty() {
                word.len()
            } else {
                current.len() + 1 + word.len()
            };
            if candidate_len > max_chars && !current.is_empty() {
                lines.push(std::mem::take(&mut current));
                if lines.len() >= max_lines {
                    break;
                }
                current = word.to_owned();
            } else if current.is_empty() {
                current.push_str(word);
            } else {
                current.push(' ');
                current.push_str(word);
            }
        }
        if lines.len() >= max_lines {
            break;
        }
        lines.push(current);
        if lines.len() >= max_lines {
            break;
        }
    }
    if lines.len() >= max_lines
        && let Some(last) = lines.last_mut()
        && !last.ends_with("...")
    {
        last.push_str("...");
    }
    lines
}

/// Draws an in-window runtime error overlay while keeping terminal logs unchanged.
pub fn draw_runtime_error_overlay(message: &str) {
    let width = screen_width();
    let height = screen_height();
    clear_background(MqColor {
        r: 0.08,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    });

    let margin = 24.0;
    let panel_x = margin;
    let panel_y = margin;
    let panel_w = (width - (2.0 * margin)).max(100.0);
    let panel_h = (height - (2.0 * margin)).max(100.0);
    draw_rectangle(
        panel_x,
        panel_y,
        panel_w,
        panel_h,
        MqColor {
            r: 0.16,
            g: 0.03,
            b: 0.03,
            a: 0.95,
        },
    );
    draw_text(
        "pycro runtime error",
        panel_x + 16.0,
        panel_y + 36.0,
        32.0,
        MqColor {
            r: 1.0,
            g: 0.85,
            b: 0.85,
            a: 1.0,
        },
    );

    let mut y = panel_y + 68.0;
    for line in wrap_overlay_message(message, 96, 22) {
        if y > panel_y + panel_h - 12.0 {
            break;
        }
        draw_text(
            line.as_str(),
            panel_x + 16.0,
            y,
            22.0,
            MqColor {
                r: 0.98,
                g: 0.96,
                b: 0.96,
                a: 1.0,
            },
        );
        y += 24.0;
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
    let conf = Conf {
        window_title: "pycro".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi,
        sample_count,
        ..Conf::default()
    };

    #[cfg(target_os = "macos")]
    {
        let mut conf = conf;
        if std::env::consts::ARCH == "aarch64" {
            let selected = env::var("PYCRO_APPLE_GFX_API")
                .ok()
                .map(|value| value.to_ascii_lowercase());
            conf.platform.apple_gfx_api = match selected.as_deref() {
                Some("metal") => AppleGfxApi::Metal,
                Some("opengl") | Some("gl") => AppleGfxApi::OpenGl,
                _ => AppleGfxApi::Metal,
            };
        }
        conf
    }

    #[cfg(not(target_os = "macos"))]
    {
        conf
    }
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
    pending_texture: Option<TextureHandle>,
    pending_texture_draws: Vec<TextureDraw>,
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
            pending_texture: None,
            pending_texture_draws: Vec::new(),
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

    fn max_sprites_per_texture_geometry_batch(&self) -> usize {
        let max_vertices = self.drawcall_max_vertices.unwrap_or(10_000);
        let max_indices = self.drawcall_max_indices.unwrap_or(5_000);
        let by_vertices = max_vertices / 4;
        let by_indices = max_indices / 6;
        by_vertices
            .min(by_indices)
            .max(1)
            .min(usize::from(u16::MAX) / 4)
    }

    fn queue_texture_draw(&mut self, texture: &TextureHandle, position: Vec2, size: Vec2) {
        if self
            .pending_texture
            .as_ref()
            .is_some_and(|pending| pending == texture)
        {
            self.pending_texture_draws
                .push(TextureDraw { position, size });
            return;
        }
        self.flush_pending_texture_batch();
        self.pending_texture = Some(texture.clone());
        self.pending_texture_draws
            .push(TextureDraw { position, size });
    }

    fn draw_texture_geometry_chunk(&self, native_texture: &Texture2D, draws: &[TextureDraw]) {
        #[cfg(not(test))]
        {
            let mut vertices = Vec::with_capacity(draws.len() * 4);
            let mut indices = Vec::with_capacity(draws.len() * 6);
            for (draw_index, draw) in draws.iter().enumerate() {
                let base = u16::try_from(draw_index * 4).expect("texture batch index must fit u16");
                let x = draw.position.x;
                let y = draw.position.y;
                let w = draw.size.x;
                let h = draw.size.y;
                vertices.push(Vertex::new(x, y, 0.0, 0.0, 0.0, WHITE));
                vertices.push(Vertex::new(x + w, y, 0.0, 1.0, 0.0, WHITE));
                vertices.push(Vertex::new(x + w, y + h, 0.0, 1.0, 1.0, WHITE));
                vertices.push(Vertex::new(x, y + h, 0.0, 0.0, 1.0, WHITE));
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }
            let mesh = Mesh {
                vertices,
                indices,
                texture: Some(native_texture.clone()),
            };
            draw_mesh(&mesh);
        }
        #[cfg(test)]
        {
            let _ = (native_texture, draws);
        }
    }

    fn draw_texture_batch_native(&self, native_texture: &Texture2D, draws: &[TextureDraw]) {
        let chunk_size = self.max_sprites_per_texture_geometry_batch();
        for chunk in draws.chunks(chunk_size) {
            if chunk.len() == 1 {
                let draw = chunk[0];
                draw_texture_ex(
                    native_texture,
                    draw.position.x,
                    draw.position.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(draw.size.x, draw.size.y)),
                        ..DrawTextureParams::default()
                    },
                );
            } else {
                self.draw_texture_geometry_chunk(native_texture, chunk);
            }
        }
    }

    fn flush_pending_texture_batch(&mut self) {
        let Some(texture) = self.pending_texture.take() else {
            return;
        };
        let draws = std::mem::take(&mut self.pending_texture_draws);
        if draws.is_empty() {
            return;
        }

        #[cfg(target_os = "android")]
        if !self.textures.contains_key(&texture.0) {
            let normalized = texture.0.trim_start_matches("./");
            if let Some(payload) = embedded_project_payload() {
                if let Some(file) = payload
                    .files
                    .iter()
                    .find(|file| file.relative_path == normalized)
                {
                    let native_texture = Texture2D::from_file_with_format(file.bytes, None);
                    self.textures.insert(texture.0.clone(), native_texture);
                }
            }
        }

        if let Some(native_texture) = self.textures.get(&texture.0) {
            self.draw_texture_batch_native(native_texture, &draws);
            return;
        }

        // Fallback marker when texture file cannot be loaded.
        for draw in draws {
            draw_rectangle(
                draw.position.x,
                draw.position.y,
                draw.size.x,
                draw.size.y,
                WHITE,
            );
        }
    }
}

fn key_code_from_name(key: &str) -> Option<KeyCode> {
    if key.len() == 1 {
        let letter = key
            .as_bytes()
            .first()
            .copied()
            .map(char::from)
            .map(|value| value.to_ascii_uppercase());
        return match letter {
            Some('A') => Some(KeyCode::A),
            Some('B') => Some(KeyCode::B),
            Some('C') => Some(KeyCode::C),
            Some('D') => Some(KeyCode::D),
            Some('E') => Some(KeyCode::E),
            Some('F') => Some(KeyCode::F),
            Some('G') => Some(KeyCode::G),
            Some('H') => Some(KeyCode::H),
            Some('I') => Some(KeyCode::I),
            Some('J') => Some(KeyCode::J),
            Some('K') => Some(KeyCode::K),
            Some('L') => Some(KeyCode::L),
            Some('M') => Some(KeyCode::M),
            Some('N') => Some(KeyCode::N),
            Some('O') => Some(KeyCode::O),
            Some('P') => Some(KeyCode::P),
            Some('Q') => Some(KeyCode::Q),
            Some('R') => Some(KeyCode::R),
            Some('S') => Some(KeyCode::S),
            Some('T') => Some(KeyCode::T),
            Some('U') => Some(KeyCode::U),
            Some('V') => Some(KeyCode::V),
            Some('W') => Some(KeyCode::W),
            Some('X') => Some(KeyCode::X),
            Some('Y') => Some(KeyCode::Y),
            Some('Z') => Some(KeyCode::Z),
            _ => None,
        };
    }

    let normalized = key.to_ascii_lowercase().replace('_', "");

    match normalized.as_str() {
        "space" => Some(KeyCode::Space),
        "apostrophe" => Some(KeyCode::Apostrophe),
        "comma" => Some(KeyCode::Comma),
        "minus" => Some(KeyCode::Minus),
        "period" => Some(KeyCode::Period),
        "slash" => Some(KeyCode::Slash),
        "key0" | "0" => Some(KeyCode::Key0),
        "key1" | "1" => Some(KeyCode::Key1),
        "key2" | "2" => Some(KeyCode::Key2),
        "key3" | "3" => Some(KeyCode::Key3),
        "key4" | "4" => Some(KeyCode::Key4),
        "key5" | "5" => Some(KeyCode::Key5),
        "key6" | "6" => Some(KeyCode::Key6),
        "key7" | "7" => Some(KeyCode::Key7),
        "key8" | "8" => Some(KeyCode::Key8),
        "key9" | "9" => Some(KeyCode::Key9),
        "semicolon" => Some(KeyCode::Semicolon),
        "equal" => Some(KeyCode::Equal),
        "leftbracket" => Some(KeyCode::LeftBracket),
        "backslash" => Some(KeyCode::Backslash),
        "rightbracket" => Some(KeyCode::RightBracket),
        "graveaccent" => Some(KeyCode::GraveAccent),
        "world1" => Some(KeyCode::World1),
        "world2" => Some(KeyCode::World2),
        "escape" => Some(KeyCode::Escape),
        "enter" => Some(KeyCode::Enter),
        "tab" => Some(KeyCode::Tab),
        "backspace" => Some(KeyCode::Backspace),
        "insert" => Some(KeyCode::Insert),
        "delete" => Some(KeyCode::Delete),
        "right" => Some(KeyCode::Right),
        "left" => Some(KeyCode::Left),
        "down" => Some(KeyCode::Down),
        "up" => Some(KeyCode::Up),
        "pageup" => Some(KeyCode::PageUp),
        "pagedown" => Some(KeyCode::PageDown),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "capslock" => Some(KeyCode::CapsLock),
        "scrolllock" => Some(KeyCode::ScrollLock),
        "numlock" => Some(KeyCode::NumLock),
        "printscreen" => Some(KeyCode::PrintScreen),
        "pause" => Some(KeyCode::Pause),
        "f1" => Some(KeyCode::F1),
        "f2" => Some(KeyCode::F2),
        "f3" => Some(KeyCode::F3),
        "f4" => Some(KeyCode::F4),
        "f5" => Some(KeyCode::F5),
        "f6" => Some(KeyCode::F6),
        "f7" => Some(KeyCode::F7),
        "f8" => Some(KeyCode::F8),
        "f9" => Some(KeyCode::F9),
        "f10" => Some(KeyCode::F10),
        "f11" => Some(KeyCode::F11),
        "f12" => Some(KeyCode::F12),
        "f13" => Some(KeyCode::F13),
        "f14" => Some(KeyCode::F14),
        "f15" => Some(KeyCode::F15),
        "f16" => Some(KeyCode::F16),
        "f17" => Some(KeyCode::F17),
        "f18" => Some(KeyCode::F18),
        "f19" => Some(KeyCode::F19),
        "f20" => Some(KeyCode::F20),
        "f21" => Some(KeyCode::F21),
        "f22" => Some(KeyCode::F22),
        "f23" => Some(KeyCode::F23),
        "f24" => Some(KeyCode::F24),
        "f25" => Some(KeyCode::F25),
        "kp0" => Some(KeyCode::Kp0),
        "kp1" => Some(KeyCode::Kp1),
        "kp2" => Some(KeyCode::Kp2),
        "kp3" => Some(KeyCode::Kp3),
        "kp4" => Some(KeyCode::Kp4),
        "kp5" => Some(KeyCode::Kp5),
        "kp6" => Some(KeyCode::Kp6),
        "kp7" => Some(KeyCode::Kp7),
        "kp8" => Some(KeyCode::Kp8),
        "kp9" => Some(KeyCode::Kp9),
        "kpdecimal" => Some(KeyCode::KpDecimal),
        "kpdivide" => Some(KeyCode::KpDivide),
        "kpmultiply" => Some(KeyCode::KpMultiply),
        "kpsubtract" => Some(KeyCode::KpSubtract),
        "kpadd" => Some(KeyCode::KpAdd),
        "kpenter" => Some(KeyCode::KpEnter),
        "kpequal" => Some(KeyCode::KpEqual),
        "leftshift" => Some(KeyCode::LeftShift),
        "leftcontrol" => Some(KeyCode::LeftControl),
        "leftalt" => Some(KeyCode::LeftAlt),
        "leftsuper" => Some(KeyCode::LeftSuper),
        "rightshift" => Some(KeyCode::RightShift),
        "rightcontrol" => Some(KeyCode::RightControl),
        "rightalt" => Some(KeyCode::RightAlt),
        "rightsuper" => Some(KeyCode::RightSuper),
        "menu" => Some(KeyCode::Menu),
        "back" => Some(KeyCode::Back),
        _ => None,
    }
}

fn mouse_button_from_name(key: &str) -> Option<MouseButton> {
    match key {
        "MOUSE_LEFT" | "mouse_left" | "MouseLeft" => Some(MouseButton::Left),
        "MOUSE_RIGHT" | "mouse_right" | "MouseRight" => Some(MouseButton::Right),
        "MOUSE_MIDDLE" | "mouse_middle" | "MouseMiddle" => Some(MouseButton::Middle),
        _ => None,
    }
}

impl EngineBackend for MacroquadBackendContract {
    fn clear_background(&mut self, color: Color) {
        self.flush_pending_texture_batch();
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
        self.flush_pending_texture_batch();
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
        self.flush_pending_texture_batch();
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
        if self.use_circle_sprite
            && let Some(sprite) = self.ensure_circle_sprite()
        {
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
        #[cfg(test)]
        {
            mouse_button_from_name(key).is_some() || key_code_from_name(key).is_some()
        }
        #[cfg(not(test))]
        {
            if let Some(button) = mouse_button_from_name(key) {
                return is_mouse_button_down(button);
            }
            key_code_from_name(key).is_some_and(is_key_down)
        }
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
        #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
        {
            let resolved = Path::new(path);
            match std::fs::read(resolved) {
                Ok(bytes) => {
                    let texture = Texture2D::from_file_with_format(&bytes, None);
                    self.textures.insert(handle.0.clone(), texture);
                }
                Err(error) => {
                    return Err(format!("load_texture: could not read \"{path}\": {error}"));
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let normalized = path.trim_start_matches("./");
            match embedded_project_payload() {
                None => {
                    return Err(format!(
                        "load_texture: no embedded project payload: \"{path}\" cannot be loaded in WASM without a project build"
                    ));
                }
                Some(payload) => {
                    match payload
                        .files
                        .iter()
                        .find(|file| file.relative_path == normalized)
                    {
                        None => {
                            return Err(format!(
                                "load_texture: file not found in embedded payload: \"{path}\""
                            ));
                        }
                        Some(file) => {
                            let texture = Texture2D::from_file_with_format(file.bytes, None);
                            self.textures.insert(handle.0.clone(), texture);
                        }
                    }
                }
            }
        }
        #[cfg(target_os = "android")]
        {
            // Android: defer texture decoding until first draw to avoid startup crashes
            // when user scripts load textures during module import.
        }

        Ok(handle)
    }

    fn draw_texture(&mut self, texture: &TextureHandle, position: Vec2, size: Vec2) {
        self.apply_drawcall_capacity_once();
        self.record_dispatch();
        #[cfg(test)]
        self.dispatch_log.push(BackendDispatch::DrawTexture {
            texture: texture.clone(),
            position,
            size,
        });
        self.queue_texture_draw(texture, position, size);
    }

    fn finish_frame(&mut self) {
        self.flush_pending_texture_batch();
    }

    fn set_camera_target(&mut self, target: Vec2) {
        self.flush_pending_texture_batch();
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
        self.flush_pending_texture_batch();
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

    fn get_window_size(&self) -> Vec2 {
        #[cfg(test)]
        {
            Vec2 {
                x: 1280.0,
                y: 720.0,
            }
        }
        #[cfg(not(test))]
        {
            Vec2 {
                x: screen_width(),
                y: screen_height(),
            }
        }
    }

    fn get_mouse_position(&self) -> Vec2 {
        #[cfg(test)]
        {
            Vec2 { x: 0.0, y: 0.0 }
        }
        #[cfg(not(test))]
        {
            let (x, y) = macroquad::input::mouse_position();
            Vec2 { x, y }
        }
    }

    fn draw_rectangle(&mut self, position: Vec2, size: Vec2, color: Color) {
        self.flush_pending_texture_batch();
        self.record_dispatch();
        #[cfg(test)]
        self.dispatch_log.push(BackendDispatch::DrawRectangle {
            position,
            size,
            color,
        });
        draw_rectangle(position.x, position.y, size.x, size.y, color.into());
    }

    fn put_pixel(&mut self, position: Vec2, color: Color) {
        self.flush_pending_texture_batch();
        self.record_dispatch();
        #[cfg(test)]
        self.dispatch_log
            .push(BackendDispatch::PutPixel { position, color });
        draw_rectangle(position.x, position.y, 1.0, 1.0, color.into());
    }

    fn draw_line(&mut self, start: Vec2, end: Vec2, color: Color) {
        self.flush_pending_texture_batch();
        self.record_dispatch();
        #[cfg(test)]
        self.dispatch_log
            .push(BackendDispatch::DrawLine { start, end, color });
        mq_draw_line(start.x, start.y, end.x, end.y, 1.0, color.into());
    }
}

#[cfg(test)]
mod tests {
    use super::{key_code_from_name, mouse_button_from_name};
    use crate::api::KEY_ENUM_MEMBERS;

    #[test]
    fn key_code_from_name_supports_expected_aliases() {
        for key in ["Left", "left", "Right", "right", "Up", "up", "Down", "down"] {
            assert!(
                key_code_from_name(key).is_some(),
                "expected known key mapping for {key}"
            );
        }

        for key in ["Space", "space", "Escape", "escape"] {
            assert!(
                key_code_from_name(key).is_some(),
                "expected known control mapping for {key}"
            );
        }

        for key in ["Enter", "Tab", "Backspace", "Insert", "Delete"] {
            assert!(
                key_code_from_name(key).is_some(),
                "expected known extended key mapping for {key}"
            );
        }

        for key in ["A", "a", "B", "b", "Z", "z"] {
            assert!(
                key_code_from_name(key).is_some(),
                "expected known letter mapping for {key}"
            );
        }
    }

    #[test]
    fn key_code_from_name_rejects_unknown_keys() {
        for key in ["Unknown", "NopeKey", ""] {
            assert!(
                key_code_from_name(key).is_none(),
                "unexpected mapping for unsupported key {key:?}"
            );
        }
    }

    #[test]
    fn key_code_from_name_supports_all_non_mouse_key_enum_values() {
        for (enum_name, value) in KEY_ENUM_MEMBERS {
            if enum_name.starts_with("MOUSE_") {
                continue;
            }
            assert!(
                key_code_from_name(value).is_some(),
                "missing key mapping for KEY.{enum_name} = {value:?}"
            );
        }
    }

    #[test]
    fn mouse_button_from_name_supports_expected_aliases() {
        for key in [
            "MOUSE_LEFT",
            "mouse_left",
            "MouseLeft",
            "MOUSE_RIGHT",
            "mouse_right",
            "MouseRight",
            "MOUSE_MIDDLE",
            "mouse_middle",
            "MouseMiddle",
        ] {
            assert!(
                mouse_button_from_name(key).is_some(),
                "expected known mouse mapping for {key}"
            );
        }
    }
}
