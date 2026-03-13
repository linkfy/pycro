//! Canonical Python API metadata and deterministic stub rendering.

use crate::backend::{Color, EngineBackend, TextureHandle, Vec2, VectorRenderMode};
use std::fmt::Write;

/// The exported Python module name.
pub const MODULE_NAME: &str = "pycro";
/// The script entrypoint file name.
pub const ENTRYPOINT_SCRIPT: &str = "main.py";
/// The required frame update function name.
pub const UPDATE_FUNCTION: &str = "update";

/// A top-level API family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApiFamily {
    /// Rendering primitives.
    Render,
    /// Input polling.
    Input,
    /// Frame timing.
    Timing,
    /// Texture and asset handling.
    Assets,
    /// Camera controls.
    Camera,
}

/// Declares current support on a target platform.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformSupportLevel {
    /// The contract is supported on this target.
    Supported,
    /// The contract is planned and must be tracked.
    Planned,
}

impl PlatformSupportLevel {
    /// Returns the machine-readable support label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Planned => "planned",
        }
    }
}

/// Declares support across supported targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformMatrix {
    /// Desktop support status.
    pub desktop: PlatformSupportLevel,
    /// Web support status.
    pub web: PlatformSupportLevel,
    /// Android support status.
    pub android: PlatformSupportLevel,
    /// iOS support status.
    pub ios: PlatformSupportLevel,
}

impl PlatformMatrix {
    /// Returns a matrix that marks API as cross-platform-safe.
    #[must_use]
    pub const fn cross_platform_safe() -> Self {
        Self {
            desktop: PlatformSupportLevel::Planned,
            web: PlatformSupportLevel::Planned,
            android: PlatformSupportLevel::Planned,
            ios: PlatformSupportLevel::Planned,
        }
    }
}

/// A Python type alias exposed through the engine module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PythonAlias {
    /// Alias name.
    pub name: &'static str,
    /// Right-hand side type definition.
    pub definition: &'static str,
    /// Short user-facing summary.
    pub summary: &'static str,
}

/// A Python function argument.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PythonArg {
    /// Argument name.
    pub name: &'static str,
    /// Python type hint.
    pub type_hint: &'static str,
    /// Short summary for docs and validation.
    pub summary: &'static str,
}

/// A public Python function exposed by `pycro`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PythonFunction {
    /// Function name.
    pub name: &'static str,
    /// API family.
    pub family: ApiFamily,
    /// Short user-facing summary.
    pub summary: &'static str,
    /// Ordered function arguments.
    pub args: &'static [PythonArg],
    /// Python return type hint.
    pub return_type: &'static str,
    /// Per-target support declaration.
    pub platforms: PlatformMatrix,
}

/// The canonical module spec for `pycro`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleSpec {
    /// Module name.
    pub module_name: &'static str,
    /// Public type aliases.
    pub aliases: &'static [PythonAlias],
    /// Public functions.
    pub functions: &'static [PythonFunction],
}

/// A runtime registration plan entry derived from the module spec.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegistrationFunction {
    /// Target module name.
    pub module_name: &'static str,
    /// Exported function name.
    pub function_name: &'static str,
    /// Family label.
    pub family: ApiFamily,
    /// Ordered function arguments sourced from canonical metadata.
    pub args: &'static [PythonArg],
    /// Python return type hint sourced from canonical metadata.
    pub return_type: &'static str,
    /// Support declaration.
    pub platforms: PlatformMatrix,
}

/// Backend command emitted by the Python-facing `pycro` shim.
#[derive(Clone, Debug, PartialEq)]
pub enum BackendDispatchCommand {
    /// `clear_background(color)`
    ClearBackground(Color),
    /// `draw_circle(position, radius, color)`
    DrawCircle {
        /// Circle center.
        position: Vec2,
        /// Circle radius.
        radius: f32,
        /// Circle color.
        color: Color,
    },
    /// `load_texture(path)`
    LoadTexture(String),
    /// `draw_texture(texture, position, size)`
    DrawTexture {
        /// Texture handle.
        texture: TextureHandle,
        /// Destination top-left.
        position: Vec2,
        /// Destination size.
        size: Vec2,
    },
    /// `set_camera_target(target)`
    SetCameraTarget(Vec2),
    /// `draw_text(text, position, font_size, color)`
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

/// Parses a runtime dispatch record into a typed backend command.
pub fn parse_backend_dispatch_record(record: &str) -> Result<BackendDispatchCommand, String> {
    let parts = record.split('|').collect::<Vec<_>>();
    let parse_f32 = |value: &str| {
        value
            .parse::<f32>()
            .map_err(|error| format!("invalid float `{value}`: {error}"))
    };

    match parts.as_slice() {
        ["clear_background", r, g, b, a] => Ok(BackendDispatchCommand::ClearBackground(Color {
            r: parse_f32(r)?,
            g: parse_f32(g)?,
            b: parse_f32(b)?,
            a: parse_f32(a)?,
        })),
        ["draw_circle", x, y, radius, r, g, b, a] => Ok(BackendDispatchCommand::DrawCircle {
            position: Vec2 {
                x: parse_f32(x)?,
                y: parse_f32(y)?,
            },
            radius: parse_f32(radius)?,
            color: Color {
                r: parse_f32(r)?,
                g: parse_f32(g)?,
                b: parse_f32(b)?,
                a: parse_f32(a)?,
            },
        }),
        ["load_texture", path] => Ok(BackendDispatchCommand::LoadTexture((*path).to_owned())),
        ["draw_texture", texture, x, y, width, height] => Ok(BackendDispatchCommand::DrawTexture {
            texture: TextureHandle((*texture).to_owned()),
            position: Vec2 {
                x: parse_f32(x)?,
                y: parse_f32(y)?,
            },
            size: Vec2 {
                x: parse_f32(width)?,
                y: parse_f32(height)?,
            },
        }),
        ["set_camera_target", x, y] => Ok(BackendDispatchCommand::SetCameraTarget(Vec2 {
            x: parse_f32(x)?,
            y: parse_f32(y)?,
        })),
        ["draw_text", text, x, y, font_size, r, g, b, a] => Ok(BackendDispatchCommand::DrawText {
            text: (*text).to_owned(),
            position: Vec2 {
                x: parse_f32(x)?,
                y: parse_f32(y)?,
            },
            font_size: parse_f32(font_size)?,
            color: Color {
                r: parse_f32(r)?,
                g: parse_f32(g)?,
                b: parse_f32(b)?,
                a: parse_f32(a)?,
            },
        }),
        _ => Err(format!("unsupported dispatch record: {record}")),
    }
}

/// Dispatches a record into the backend contract.
pub fn dispatch_backend_record(
    record: &str,
    backend: &mut dyn EngineBackend,
) -> Result<(), String> {
    match parse_backend_dispatch_record(record)? {
        BackendDispatchCommand::ClearBackground(color) => backend.clear_background(color),
        BackendDispatchCommand::DrawCircle {
            position,
            radius,
            color,
        } => backend.draw_circle(position, radius, color, VectorRenderMode::Default),
        BackendDispatchCommand::LoadTexture(path) => {
            backend.load_texture(path.as_str())?;
        }
        BackendDispatchCommand::DrawTexture {
            texture,
            position,
            size,
        } => backend.draw_texture(&texture, position, size),
        BackendDispatchCommand::SetCameraTarget(target) => backend.set_camera_target(target),
        BackendDispatchCommand::DrawText {
            text,
            position,
            font_size,
            color,
        } => backend.draw_text(text.as_str(), position, font_size, color),
    }
    Ok(())
}

const ALIASES: [PythonAlias; 3] = [
    PythonAlias {
        name: "Color",
        definition: "tuple[float, float, float, float]",
        summary: "Normalized RGBA tuple.",
    },
    PythonAlias {
        name: "Vec2",
        definition: "tuple[float, float]",
        summary: "Two-dimensional vector tuple.",
    },
    PythonAlias {
        name: "TextureHandle",
        definition: "str",
        summary: "Opaque texture handle returned by the engine.",
    },
];

const CLEAR_BACKGROUND_ARGS: [PythonArg; 1] = [PythonArg {
    name: "color",
    type_hint: "Color",
    summary: "Background color for the current frame.",
}];

const DRAW_CIRCLE_ARGS: [PythonArg; 3] = [
    PythonArg {
        name: "position",
        type_hint: "Vec2",
        summary: "Center point in world space.",
    },
    PythonArg {
        name: "radius",
        type_hint: "float",
        summary: "Circle radius in world units.",
    },
    PythonArg {
        name: "color",
        type_hint: "Color",
        summary: "Fill color for the circle.",
    },
];

const IS_KEY_DOWN_ARGS: [PythonArg; 1] = [PythonArg {
    name: "key",
    type_hint: "str",
    summary: "Platform-neutral key identifier.",
}];

const LOAD_TEXTURE_ARGS: [PythonArg; 1] = [PythonArg {
    name: "path",
    type_hint: "str",
    summary: "Engine-relative asset path.",
}];

const DRAW_TEXTURE_ARGS: [PythonArg; 3] = [
    PythonArg {
        name: "texture",
        type_hint: "TextureHandle",
        summary: "Texture handle from load_texture.",
    },
    PythonArg {
        name: "position",
        type_hint: "Vec2",
        summary: "Top-left destination point.",
    },
    PythonArg {
        name: "size",
        type_hint: "Vec2",
        summary: "Destination size in world units.",
    },
];

const SET_CAMERA_TARGET_ARGS: [PythonArg; 1] = [PythonArg {
    name: "target",
    type_hint: "Vec2",
    summary: "Camera target in world space.",
}];

const DRAW_TEXT_ARGS: [PythonArg; 4] = [
    PythonArg {
        name: "text",
        type_hint: "str",
        summary: "Text content to draw.",
    },
    PythonArg {
        name: "position",
        type_hint: "Vec2",
        summary: "Screen-space baseline anchor.",
    },
    PythonArg {
        name: "font_size",
        type_hint: "float",
        summary: "Font size in pixels.",
    },
    PythonArg {
        name: "color",
        type_hint: "Color",
        summary: "Text color.",
    },
];

const SUBMIT_RENDER_ARGS: [PythonArg; 1] = [PythonArg {
    name: "commands",
    type_hint: "list[tuple[object, ...]]",
    summary: "Ordered render command payload for batched submission.",
}];

const SUBMIT_CIRCLE_BATCH_ARGS: [PythonArg; 3] = [
    PythonArg {
        name: "positions",
        type_hint: "list[Vec2]",
        summary: "Ordered circle center positions.",
    },
    PythonArg {
        name: "radii",
        type_hint: "list[float]",
        summary: "Ordered circle radii.",
    },
    PythonArg {
        name: "colors",
        type_hint: "list[Color]",
        summary: "Ordered circle colors.",
    },
];

const FUNCTIONS: [PythonFunction; 10] = [
    PythonFunction {
        name: "clear_background",
        family: ApiFamily::Render,
        summary: "Clear the current frame to a normalized RGBA color.",
        args: &CLEAR_BACKGROUND_ARGS,
        return_type: "None",
        platforms: PlatformMatrix::cross_platform_safe(),
    },
    PythonFunction {
        name: "draw_circle",
        family: ApiFamily::Render,
        summary: "Draw a filled circle using world-space coordinates.",
        args: &DRAW_CIRCLE_ARGS,
        return_type: "None",
        platforms: PlatformMatrix::cross_platform_safe(),
    },
    PythonFunction {
        name: "is_key_down",
        family: ApiFamily::Input,
        summary: "Return whether a named key is held on the current frame.",
        args: &IS_KEY_DOWN_ARGS,
        return_type: "bool",
        platforms: PlatformMatrix::cross_platform_safe(),
    },
    PythonFunction {
        name: "frame_time",
        family: ApiFamily::Timing,
        summary: "Return the last frame delta time in seconds.",
        args: &[],
        return_type: "float",
        platforms: PlatformMatrix::cross_platform_safe(),
    },
    PythonFunction {
        name: "load_texture",
        family: ApiFamily::Assets,
        summary: "Load a texture asset and return an opaque handle.",
        args: &LOAD_TEXTURE_ARGS,
        return_type: "TextureHandle",
        platforms: PlatformMatrix::cross_platform_safe(),
    },
    PythonFunction {
        name: "draw_texture",
        family: ApiFamily::Assets,
        summary: "Draw a texture at a world-space position and size.",
        args: &DRAW_TEXTURE_ARGS,
        return_type: "None",
        platforms: PlatformMatrix::cross_platform_safe(),
    },
    PythonFunction {
        name: "set_camera_target",
        family: ApiFamily::Camera,
        summary: "Move the active camera target to the provided world position.",
        args: &SET_CAMERA_TARGET_ARGS,
        return_type: "None",
        platforms: PlatformMatrix::cross_platform_safe(),
    },
    PythonFunction {
        name: "draw_text",
        family: ApiFamily::Render,
        summary: "Draw text in screen space using a baseline anchor.",
        args: &DRAW_TEXT_ARGS,
        return_type: "None",
        platforms: PlatformMatrix::cross_platform_safe(),
    },
    PythonFunction {
        name: "submit_render",
        family: ApiFamily::Render,
        summary: "Queue multiple render commands in one Python-to-runtime call.",
        args: &SUBMIT_RENDER_ARGS,
        return_type: "None",
        platforms: PlatformMatrix::cross_platform_safe(),
    },
    PythonFunction {
        name: "submit_circle_batch",
        family: ApiFamily::Render,
        summary: "Queue many draw_circle operations in one specialized batch call.",
        args: &SUBMIT_CIRCLE_BATCH_ARGS,
        return_type: "None",
        platforms: PlatformMatrix::cross_platform_safe(),
    },
];

const MODULE_SPEC: ModuleSpec = ModuleSpec {
    module_name: MODULE_NAME,
    aliases: &ALIASES,
    functions: &FUNCTIONS,
};

/// Returns the canonical module specification.
#[must_use]
pub const fn module_spec() -> &'static ModuleSpec {
    &MODULE_SPEC
}

/// Returns the runtime registration plan derived from the canonical module spec.
#[must_use]
pub fn registration_plan() -> Vec<RegistrationFunction> {
    module_spec()
        .functions
        .iter()
        .map(|function| RegistrationFunction {
            module_name: module_spec().module_name,
            function_name: function.name,
            family: function.family,
            args: function.args,
            return_type: function.return_type,
            platforms: function.platforms,
        })
        .collect()
}

/// Renders a canonical `__init__.pyi` file from the module spec.
#[must_use]
pub fn render_stub(spec: &ModuleSpec) -> String {
    let mut output = String::new();
    writeln!(
        output,
        "\"\"\"Generated stubs for `{}`.\n\nDo not edit this file manually.\nThe engine loads `{}` and dispatches required `{}(dt)` on each frame.\n\"\"\"",
        MODULE_NAME, ENTRYPOINT_SCRIPT, UPDATE_FUNCTION
    )
    .expect("writing to String cannot fail");
    writeln!(output).expect("writing to String cannot fail");
    writeln!(output, "from typing import TypeAlias").expect("writing to String cannot fail");
    writeln!(output).expect("writing to String cannot fail");

    for alias in spec.aliases {
        writeln!(output, "{}: TypeAlias = {}", alias.name, alias.definition)
            .expect("writing to String cannot fail");
        writeln!(output, "\"\"\"{}\"\"\"", alias.summary).expect("writing to String cannot fail");
        writeln!(output).expect("writing to String cannot fail");
    }

    let mut exports: Vec<&str> = spec.aliases.iter().map(|alias| alias.name).collect();
    exports.extend(spec.functions.iter().map(|function| function.name));

    writeln!(
        output,
        "__all__ = [{}]",
        exports
            .iter()
            .map(|name| format!("\"{name}\""))
            .collect::<Vec<_>>()
            .join(", ")
    )
    .expect("writing to String cannot fail");
    writeln!(output).expect("writing to String cannot fail");

    for function in spec.functions {
        let signature = function
            .args
            .iter()
            .map(|arg| format!("{}: {}", arg.name, arg.type_hint))
            .collect::<Vec<_>>()
            .join(", ");

        writeln!(
            output,
            "def {}({}) -> {}:",
            function.name, signature, function.return_type
        )
        .expect("writing to String cannot fail");
        writeln!(
            output,
            "    \"\"\"{} Supported on desktop={}, web={}, android={}, ios={}.\"\"\"",
            function.summary,
            function.platforms.desktop.as_str(),
            function.platforms.web.as_str(),
            function.platforms.android.as_str(),
            function.platforms.ios.as_str()
        )
        .expect("writing to String cannot fail");
        writeln!(output, "    ...").expect("writing to String cannot fail");
        writeln!(output).expect("writing to String cannot fail");
    }

    output
}

#[cfg(test)]
mod tests {
    use super::{
        ApiFamily, BackendDispatchCommand, dispatch_backend_record, module_spec,
        parse_backend_dispatch_record, registration_plan, render_stub,
    };
    use crate::backend::{Color, EngineBackend, TextureHandle, Vec2, VectorRenderMode};

    #[test]
    fn metadata_covers_each_initial_family() {
        let spec = module_spec();
        let families = spec
            .functions
            .iter()
            .map(|function| function.family)
            .collect::<Vec<_>>();

        assert!(families.contains(&ApiFamily::Render));
        assert!(families.contains(&ApiFamily::Input));
        assert!(families.contains(&ApiFamily::Timing));
        assert!(families.contains(&ApiFamily::Assets));
        assert!(families.contains(&ApiFamily::Camera));
    }

    #[test]
    fn registration_plan_matches_metadata() {
        let spec = module_spec();
        let plan = registration_plan();

        assert_eq!(plan.len(), spec.functions.len());
        assert_eq!(plan[0].module_name, spec.module_name);
        assert_eq!(plan[0].args, spec.functions[0].args);
        assert_eq!(plan[0].return_type, spec.functions[0].return_type);
    }

    #[test]
    fn stub_output_mentions_each_function() {
        let stub = render_stub(module_spec());

        for function in module_spec().functions {
            assert!(stub.contains(function.name), "missing {}", function.name);
        }
    }

    #[test]
    fn metadata_and_stub_keep_direct_bridge_return_signatures() {
        let stub = render_stub(module_spec());
        let plan = registration_plan();

        let is_key_down = plan
            .iter()
            .find(|entry| entry.function_name == "is_key_down")
            .expect("is_key_down metadata should exist");
        assert_eq!(is_key_down.return_type, "bool");
        assert!(stub.contains("def is_key_down(key: str) -> bool:"));

        let frame_time = plan
            .iter()
            .find(|entry| entry.function_name == "frame_time")
            .expect("frame_time metadata should exist");
        assert_eq!(frame_time.return_type, "float");
        assert!(stub.contains("def frame_time() -> float:"));

        let load_texture = plan
            .iter()
            .find(|entry| entry.function_name == "load_texture")
            .expect("load_texture metadata should exist");
        assert_eq!(load_texture.return_type, "TextureHandle");
        assert!(stub.contains("def load_texture(path: str) -> TextureHandle:"));
    }

    #[test]
    fn parses_and_dispatches_backend_record() {
        #[derive(Default)]
        struct TestBackend {
            clear_calls: usize,
        }

        impl EngineBackend for TestBackend {
            fn clear_background(&mut self, _color: Color) {
                self.clear_calls += 1;
            }
            fn draw_circle(
                &mut self,
                _position: Vec2,
                _radius: f32,
                _color: Color,
                _render_mode: VectorRenderMode,
            ) {
            }
            fn is_key_down(&self, _key: &str) -> bool {
                false
            }
            fn frame_time(&self) -> f32 {
                0.016
            }
            fn load_texture(&mut self, path: &str) -> Result<TextureHandle, String> {
                Ok(TextureHandle(path.to_owned()))
            }
            fn draw_texture(&mut self, _texture: &TextureHandle, _position: Vec2, _size: Vec2) {}
            fn set_camera_target(&mut self, _target: Vec2) {}
            fn draw_text(&mut self, _text: &str, _position: Vec2, _font_size: f32, _color: Color) {}
        }

        let command = parse_backend_dispatch_record("clear_background|0.1|0.2|0.3|1.0")
            .expect("record should parse");
        assert_eq!(
            command,
            BackendDispatchCommand::ClearBackground(Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0
            })
        );

        let mut backend = TestBackend::default();
        dispatch_backend_record("clear_background|0.1|0.2|0.3|1.0", &mut backend)
            .expect("dispatch should succeed");
        assert_eq!(backend.clear_calls, 1);
    }

    #[test]
    fn parse_backend_dispatch_record_validates_draw_texture_payload_shape() {
        let parsed = parse_backend_dispatch_record("draw_texture|tex|1.0|2.0|3.0|4.0")
            .expect("draw_texture record with full payload should parse");
        assert_eq!(
            parsed,
            BackendDispatchCommand::DrawTexture {
                texture: TextureHandle("tex".to_owned()),
                position: Vec2 { x: 1.0, y: 2.0 },
                size: Vec2 { x: 3.0, y: 4.0 }
            }
        );

        let error = parse_backend_dispatch_record("draw_texture|tex|1.0|2.0|3.0")
            .expect_err("draw_texture record missing size component should fail");
        assert!(
            error.contains("unsupported dispatch record"),
            "unexpected parse error: {error}"
        );
    }

    #[test]
    fn dispatch_backend_record_surfaces_invalid_float_for_draw_texture() {
        #[derive(Default)]
        struct NoopBackend;

        impl EngineBackend for NoopBackend {
            fn clear_background(&mut self, _color: Color) {}
            fn draw_circle(
                &mut self,
                _position: Vec2,
                _radius: f32,
                _color: Color,
                _render_mode: VectorRenderMode,
            ) {
            }
            fn is_key_down(&self, _key: &str) -> bool {
                false
            }
            fn frame_time(&self) -> f32 {
                0.016
            }
            fn load_texture(&mut self, path: &str) -> Result<TextureHandle, String> {
                Ok(TextureHandle(path.to_owned()))
            }
            fn draw_texture(&mut self, _texture: &TextureHandle, _position: Vec2, _size: Vec2) {}
            fn set_camera_target(&mut self, _target: Vec2) {}
            fn draw_text(&mut self, _text: &str, _position: Vec2, _font_size: f32, _color: Color) {}
        }

        let mut backend = NoopBackend;
        let error = dispatch_backend_record("draw_texture|tex|oops|2.0|3.0|4.0", &mut backend)
            .expect_err("invalid draw_texture float should fail dispatch");
        assert!(
            error.contains("invalid float"),
            "unexpected dispatch error: {error}"
        );
    }
}
