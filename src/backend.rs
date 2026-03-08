//! Backend contract used by the API layer.
//! The first backend owner is Macroquad, but the contract is backend-agnostic.

use std::env;

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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameLoopConfig {
    /// Fixed timestep in seconds.
    pub frame_dt_seconds: f32,
    /// Number of frames to dispatch before returning.
    pub frame_count: usize,
}

impl Default for FrameLoopConfig {
    fn default() -> Self {
        Self {
            frame_dt_seconds: 0.016,
            frame_count: 1,
        }
    }
}

impl FrameLoopConfig {
    /// Parses loop options from environment.
    #[must_use]
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(value) = env::var("PYCRO_FRAME_DT")
            && let Ok(parsed) = value.parse::<f32>()
        {
            config.frame_dt_seconds = parsed.max(0.0);
        }

        if let Ok(value) = env::var("PYCRO_FRAMES")
            && let Ok(parsed) = value.parse::<usize>()
        {
            config.frame_count = parsed.max(1);
        }

        config
    }
}

/// First desktop frame-loop owner. This is the location where Macroquad ownership plugs in.
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

    /// Runs the frame callback exactly `frame_count` times.
    pub fn run(
        &self,
        mut on_frame: impl FnMut(f32) -> Result<(), String>,
    ) -> Result<DesktopLoopReport, String> {
        for _ in 0..self.config.frame_count {
            on_frame(self.config.frame_dt_seconds)?;
        }
        Ok(DesktopLoopReport {
            frames_executed: self.config.frame_count,
        })
    }
}

/// First backend implementation behind the contract boundary.
#[derive(Debug, Default)]
pub struct MacroquadBackendContract {
    frame_time: f32,
    dispatch_log: Vec<BackendDispatch>,
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

impl EngineBackend for MacroquadBackendContract {
    fn clear_background(&mut self, color: Color) {
        self.dispatch_log
            .push(BackendDispatch::ClearBackground(color));
    }

    fn draw_circle(&mut self, position: Vec2, radius: f32, color: Color) {
        self.dispatch_log.push(BackendDispatch::DrawCircle {
            position,
            radius,
            color,
        });
    }

    fn is_key_down(&self, _key: &str) -> bool {
        false
    }

    fn frame_time(&self) -> f32 {
        self.frame_time
    }

    fn load_texture(&mut self, path: &str) -> Result<TextureHandle, String> {
        self.dispatch_log
            .push(BackendDispatch::LoadTexture(path.to_owned()));
        Ok(TextureHandle(path.to_owned()))
    }

    fn draw_texture(&mut self, texture: &TextureHandle, position: Vec2, size: Vec2) {
        self.dispatch_log.push(BackendDispatch::DrawTexture {
            texture: texture.clone(),
            position,
            size,
        });
    }

    fn set_camera_target(&mut self, target: Vec2) {
        self.dispatch_log
            .push(BackendDispatch::SetCameraTarget(target));
    }
}
