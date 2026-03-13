//! Script runtime lifecycle contract.

use crate::api::{
    ENTRYPOINT_SCRIPT, MODULE_NAME, SETUP_FUNCTION, UPDATE_FUNCTION, registration_plan,
};
use crate::backend::{
    CircleDraw, Color, EngineBackend, MacroquadBackendContract, TextureHandle, Vec2,
    VectorRenderMode,
};
use parking_lot::{Mutex, MutexGuard};
use rustpython_vm::builtins::{PyBaseExceptionRef, PyDictRef, PyFloat, PyList, PyStr, PyTuple};
use rustpython_vm::scope::Scope;
use rustpython_vm::{AsObject, Interpreter, PyObjectRef, PyResult, Settings, VirtualMachine};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

/// Runtime configuration for Python script loading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeConfig {
    /// Path to the expected entry script.
    pub entry_script: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            entry_script: ENTRYPOINT_SCRIPT.to_owned(),
        }
    }
}

/// A runtime value passed into lifecycle functions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RuntimeValue {
    /// A floating-point value.
    Float(f32),
}

/// Errors surfaced by the runtime lifecycle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeError {
    /// The script was not loaded before use.
    NotLoaded,
    /// The script did not define the required `update(dt)` function.
    MissingUpdateFunction,
    /// Script loading failed.
    ScriptLoad {
        /// The script path that failed to load.
        path: String,
        /// The backend-specific error details.
        details: String,
    },
    /// Python function dispatch failed.
    FunctionCall {
        /// The lifecycle or API function that failed.
        function: String,
        /// The backend-specific error details.
        details: String,
    },
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotLoaded => write!(formatter, "runtime used before main.py was loaded"),
            Self::MissingUpdateFunction => {
                write!(formatter, "main.py must define update(dt: float)")
            }
            Self::ScriptLoad { path, details } => {
                write!(formatter, "failed to load {path}: {details}")
            }
            Self::FunctionCall { function, details } => {
                write!(formatter, "failed to call {function}: {details}")
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

/// Captures how the runtime will register the public Python module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleInstallPlan {
    /// Module name registered into RustPython.
    pub module_name: &'static str,
    /// Exported function count.
    pub exported_function_count: usize,
    /// Exported function names from API metadata.
    pub exported_function_names: Vec<&'static str>,
}

/// Abstract VM behavior required by lifecycle dispatch.
pub trait PythonVm {
    /// Registers the generated `pycro` module.
    fn install_module(&mut self, plan: ModuleInstallPlan) -> Result<(), RuntimeError>;
    /// Loads the user entry script.
    fn load_script(&mut self, path: &str) -> Result<(), RuntimeError>;
    /// Returns whether the loaded script defines a named function.
    fn has_function(&self, function: &str) -> Result<bool, RuntimeError>;
    /// Calls a function with runtime values.
    fn call_function(&mut self, function: &str, args: &[RuntimeValue]) -> Result<(), RuntimeError>;
    /// Flushes render operations queued during `update(dt)`.
    fn flush_draw_batch(&mut self) -> Result<(), RuntimeError>;
    /// Discards queued render operations without dispatching them.
    fn discard_draw_batch(&mut self) -> Result<(), RuntimeError>;
    /// Flushes Python stdio streams.
    fn flush_io(&mut self) -> Result<(), RuntimeError>;
}

/// Coordinates the runtime lifecycle contract.
#[derive(Debug)]
pub struct ScriptRuntime<Vm> {
    vm: Vm,
    config: RuntimeConfig,
    loaded: bool,
}

impl<Vm> ScriptRuntime<Vm>
where
    Vm: PythonVm,
{
    /// Creates a runtime around the provided VM adapter.
    #[must_use]
    pub fn new(vm: Vm, config: RuntimeConfig) -> Self {
        Self {
            vm,
            config,
            loaded: false,
        }
    }

    /// Installs module, loads script, and runs optional `setup()`.
    pub fn load_main(&mut self) -> Result<(), RuntimeError> {
        let registration = registration_plan();
        let plan = ModuleInstallPlan {
            module_name: MODULE_NAME,
            exported_function_count: registration.len(),
            exported_function_names: registration
                .iter()
                .map(|entry| entry.function_name)
                .collect(),
        };

        self.vm.install_module(plan)?;
        self.vm.load_script(&self.config.entry_script)?;

        if !self.vm.has_function(UPDATE_FUNCTION)? {
            return Err(RuntimeError::MissingUpdateFunction);
        }

        if self.vm.has_function(SETUP_FUNCTION)? {
            println!("[pycro] dispatch setup()");
            self.vm.call_function(SETUP_FUNCTION, &[])?;
        }

        self.loaded = true;
        Ok(())
    }

    /// Dispatches `update(dt)` once per frame.
    pub fn update(&mut self, dt: f32) -> Result<(), RuntimeError> {
        if !self.loaded {
            return Err(RuntimeError::NotLoaded);
        }
        let result = self
            .vm
            .call_function(UPDATE_FUNCTION, &[RuntimeValue::Float(dt)]);
        if result.is_err() {
            let _ = self.vm.discard_draw_batch();
        }
        result
    }

    /// Flushes the queued draw batch for the current frame.
    pub fn flush_draw_batch(&mut self) -> Result<(), RuntimeError> {
        if !self.loaded {
            return Err(RuntimeError::NotLoaded);
        }
        self.vm.flush_draw_batch()
    }

    /// Flushes runtime stdio buffers.
    pub fn flush_io(&mut self) -> Result<(), RuntimeError> {
        if !self.loaded {
            return Err(RuntimeError::NotLoaded);
        }
        self.vm.flush_io()
    }

    /// Returns immutable reference to underlying VM.
    #[must_use]
    pub fn vm(&self) -> &Vm {
        &self.vm
    }
}

/// First real RustPython-backed VM adapter for lifecycle dispatch.
pub struct RustPythonVm {
    interpreter: Interpreter,
    scope: Option<Scope>,
    setup_callable: Option<PyObjectRef>,
    update_callable: Option<PyObjectRef>,
    backend: Arc<Mutex<MacroquadBackendContract>>,
    draw_batch: Arc<Mutex<QueuedDrawBatch>>,
    submit_render_circle_cache: Arc<Mutex<Option<Arc<SubmitRenderCircleCache>>>>,
    circle_batch_cache: Arc<Mutex<Option<CircleBatchCache>>>,
    frame_time_seconds: Arc<AtomicU32>,
    flush_stdio_on_update: bool,
}

#[derive(Clone, Debug, PartialEq)]
enum QueuedDrawOp {
    ClearBackground(Color),
    DrawCircle {
        position: Vec2,
        radius: f32,
        color: Color,
        render_mode: VectorRenderMode,
    },
    DrawTexture {
        texture: String,
        position: Vec2,
        size: Vec2,
    },
    SetCameraTarget(Vec2),
    DrawText {
        text: String,
        position: Vec2,
        font_size: f32,
        color: Color,
    },
}

type QueuedCircle = CircleDraw;

#[derive(Clone, Debug, PartialEq)]
enum QueuedBatchEntry {
    Op(QueuedDrawOp),
    CircleRun { start: usize, len: usize },
}

#[derive(Clone, Copy, Debug)]
struct DrawBatchMark {
    entry_len: usize,
    circle_len: usize,
}

#[derive(Default, Debug)]
struct QueuedDrawBatch {
    entries: Vec<QueuedBatchEntry>,
    circles: Vec<QueuedCircle>,
}

impl QueuedDrawBatch {
    fn mark(&self) -> DrawBatchMark {
        DrawBatchMark {
            entry_len: self.entries.len(),
            circle_len: self.circles.len(),
        }
    }

    fn rollback(&mut self, mark: DrawBatchMark) {
        self.entries.truncate(mark.entry_len);
        self.circles.truncate(mark.circle_len);
    }

    fn reserve_ops(&mut self, additional_ops: usize) {
        self.entries.reserve(additional_ops);
        self.circles.reserve(additional_ops);
    }

    fn push_circle(&mut self, circle: QueuedCircle) {
        let start = self.circles.len();
        self.circles.push(circle);
        match self.entries.last_mut() {
            Some(QueuedBatchEntry::CircleRun { len, .. }) => *len += 1,
            _ => self
                .entries
                .push(QueuedBatchEntry::CircleRun { start, len: 1 }),
        }
    }

    fn finish_circle_run(&mut self, start: usize) {
        let added_len = self.circles.len().saturating_sub(start);
        if added_len == 0 {
            return;
        }
        match self.entries.last_mut() {
            Some(QueuedBatchEntry::CircleRun { len, .. }) => *len += added_len,
            _ => self.entries.push(QueuedBatchEntry::CircleRun {
                start,
                len: added_len,
            }),
        }
    }

    fn push_op(&mut self, op: QueuedDrawOp) {
        match op {
            QueuedDrawOp::DrawCircle {
                position,
                radius,
                color,
                render_mode,
            } => self.push_circle(QueuedCircle {
                position,
                radius,
                color,
                render_mode,
            }),
            other => self.entries.push(QueuedBatchEntry::Op(other)),
        }
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.circles.clear();
    }

    #[cfg(test)]
    fn to_ops_vec(&self) -> Vec<QueuedDrawOp> {
        let mut ops = Vec::with_capacity(self.len_ops());
        for entry in &self.entries {
            match entry {
                QueuedBatchEntry::Op(op) => ops.push(op.clone()),
                QueuedBatchEntry::CircleRun { start, len } => {
                    for circle in &self.circles[*start..(*start + *len)] {
                        ops.push(QueuedDrawOp::DrawCircle {
                            position: circle.position,
                            radius: circle.radius,
                            color: circle.color,
                            render_mode: circle.render_mode,
                        });
                    }
                }
            }
        }
        ops
    }

    #[cfg(test)]
    fn take_ops_vec(&mut self) -> Vec<QueuedDrawOp> {
        let ops = self.to_ops_vec();
        self.clear();
        ops
    }

    fn len_ops(&self) -> usize {
        self.entries
            .iter()
            .map(|entry| match entry {
                QueuedBatchEntry::Op(_) => 1,
                QueuedBatchEntry::CircleRun { len, .. } => *len,
            })
            .sum()
    }
}

#[derive(Clone, Debug)]
struct CircleBatchCache {
    radii_list_id: usize,
    colors_list_id: usize,
    radii: Vec<f32>,
    colors: Vec<Color>,
}

#[derive(Clone, Debug)]
struct CachedSubmitRenderCircle {
    index: usize,
    command_id: usize,
    position_obj: PyObjectRef,
    radius: f32,
    color: Color,
}

#[derive(Clone, Debug)]
enum CachedSubmitRenderLayoutEntry {
    CircleRun { start: usize, len: usize },
    NonCircle { command_index: usize },
}

#[derive(Clone, Debug)]
struct SubmitRenderCircleCache {
    commands_list_id: usize,
    command_count: usize,
    first_circle_command_id: usize,
    last_circle_command_id: usize,
    circles: Vec<CachedSubmitRenderCircle>,
    layout: Vec<CachedSubmitRenderLayoutEntry>,
}

impl std::fmt::Debug for RustPythonVm {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let backend_dispatches = self.backend.lock().dispatch_count();
        let queued_draw_ops = self.draw_batch.lock().len_ops();
        formatter
            .debug_struct("RustPythonVm")
            .field("initialized", &self.scope.is_some())
            .field("backend_dispatches", &backend_dispatches)
            .field("queued_draw_ops", &queued_draw_ops)
            .finish()
    }
}

impl Default for RustPythonVm {
    fn default() -> Self {
        Self::new()
    }
}

impl RustPythonVm {
    fn submit_render_noop_enabled() -> bool {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        *ENABLED.get_or_init(|| {
            std::env::var("PYCRO_SUBMIT_RENDER_NOOP").is_ok_and(|value| value == "1")
        })
    }

    fn submit_render_direct_enabled() -> bool {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        *ENABLED.get_or_init(|| {
            std::env::var("PYCRO_SUBMIT_RENDER_DIRECT").is_ok_and(|value| value == "1")
        })
    }

    fn draw_text_noop_enabled() -> bool {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        *ENABLED
            .get_or_init(|| std::env::var("PYCRO_DRAW_TEXT_NOOP").is_ok_and(|value| value == "1"))
    }

    fn interpreter_settings() -> Settings {
        let mut settings = Settings::default();
        settings.optimize = std::env::var("PYCRO_PY_OPTIMIZE")
            .ok()
            .and_then(|value| value.parse::<u8>().ok())
            .unwrap_or(2);
        settings
    }

    /// Creates a VM backed by a persistent RustPython interpreter.
    #[allow(clippy::arc_with_non_send_sync)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::without_stdlib(Self::interpreter_settings()),
            scope: None,
            setup_callable: None,
            update_callable: None,
            backend: Arc::new(Mutex::new(MacroquadBackendContract::default())),
            draw_batch: Arc::new(Mutex::new(QueuedDrawBatch::default())),
            submit_render_circle_cache: Arc::new(Mutex::new(None)),
            circle_batch_cache: Arc::new(Mutex::new(None)),
            frame_time_seconds: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            flush_stdio_on_update: std::env::var("PYCRO_FLUSH_STDIO_ON_UPDATE")
                .map(|value| value != "0")
                .unwrap_or(false),
        }
    }

    /// Exposes the current backend for smoke evidence.
    pub fn backend(&self) -> MutexGuard<'_, MacroquadBackendContract> {
        self.backend.lock()
    }

    #[cfg(test)]
    fn queued_draw_batch_snapshot(&self) -> Vec<QueuedDrawOp> {
        self.draw_batch.lock().to_ops_vec()
    }

    #[cfg(test)]
    fn take_queued_draw_batch_for_test(&self) -> Vec<QueuedDrawOp> {
        let mut draw_batch = self.draw_batch.lock();
        draw_batch.take_ops_vec()
    }

    fn module_bootstrap_source() -> &'static str {
        "Color = tuple\nVec2 = tuple\nTextureHandle = str\n"
    }

    fn with_scope<T>(
        &self,
        scope: Scope,
        f: impl FnOnce(&VirtualMachine, Scope) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        self.interpreter.enter(|vm| f(vm, scope))
    }

    fn exception_details(vm: &VirtualMachine, error: &PyBaseExceptionRef) -> String {
        vm.print_exception(error.clone());
        let type_name = error.class().name().to_owned();
        let message = error
            .as_object()
            .str(vm)
            .map(|value| value.as_str().to_owned())
            .unwrap_or_else(|_| String::new());
        if message.is_empty() {
            type_name
        } else {
            format!("{type_name}: {message}")
        }
    }

    fn flush_stdio(vm: &VirtualMachine) {
        if let Ok(stdout) = vm.sys_module.get_attr("stdout", vm) {
            let _ = vm.call_method(stdout.as_object(), "flush", ());
        }
        if let Ok(stderr) = vm.sys_module.get_attr("stderr", vm) {
            let _ = vm.call_method(stderr.as_object(), "flush", ());
        }
    }

    fn parse_vec2_from_items_py(
        vm: &VirtualMachine,
        items: &[PyObjectRef],
        context: &str,
    ) -> PyResult<Vec2> {
        if items.len() != 2 {
            return Err(vm.new_value_error(format!("{context}: expected Vec2 tuple length 2")));
        }
        let x = Self::parse_number_item_at_py(vm, &items[0], context, 0)?;
        let y = Self::parse_number_item_at_py(vm, &items[1], context, 1)?;
        Ok(Vec2 { x, y })
    }

    fn parse_vec2_py(vm: &VirtualMachine, object: &PyObjectRef, context: &str) -> PyResult<Vec2> {
        if let Some(tuple) = object.payload_if_subclass::<PyTuple>(vm) {
            return Self::parse_vec2_from_items_py(vm, tuple.as_slice(), context);
        }
        if let Some(list) = object.payload_if_subclass::<PyList>(vm) {
            let items = list.borrow_vec();
            return Self::parse_vec2_from_items_py(vm, &items, context);
        }
        Self::with_sequence_items_py(vm, object, "Vec2 tuple", |items| {
            Self::parse_vec2_from_items_py(vm, items, context)
        })
    }

    fn parse_vec2_cached_position_py(
        vm: &VirtualMachine,
        object: &PyObjectRef,
        context: &str,
    ) -> PyResult<Vec2> {
        if let Some(list) = object.payload_if_subclass::<PyList>(vm) {
            let items = list.borrow_vec();
            if items.len() == 2 {
                let x = Self::parse_number_item_at_py(vm, &items[0], context, 0)?;
                let y = Self::parse_number_item_at_py(vm, &items[1], context, 1)?;
                return Ok(Vec2 { x, y });
            }
        }
        Self::parse_vec2_py(vm, object, context)
    }

    fn parse_vec2_cached_position_fast_py(
        vm: &VirtualMachine,
        object: &PyObjectRef,
    ) -> Option<Vec2> {
        let list = object.payload_if_subclass::<PyList>(vm)?;
        let items = list.borrow_vec();
        let x = items.first()?.payload_if_subclass::<PyFloat>(vm)?.to_f64() as f32;
        let y = items.get(1)?.payload_if_subclass::<PyFloat>(vm)?.to_f64() as f32;
        if items.len() == 2 {
            Some(Vec2 { x, y })
        } else {
            None
        }
    }

    fn parse_color_from_items_py(
        vm: &VirtualMachine,
        items: &[PyObjectRef],
        context: &str,
    ) -> PyResult<Color> {
        if items.len() != 4 {
            return Err(vm.new_value_error(format!("{context}: expected Color tuple length 4")));
        }
        let r = Self::parse_number_item_at_py(vm, &items[0], context, 0)?;
        let g = Self::parse_number_item_at_py(vm, &items[1], context, 1)?;
        let b = Self::parse_number_item_at_py(vm, &items[2], context, 2)?;
        let a = Self::parse_number_item_at_py(vm, &items[3], context, 3)?;
        Ok(Color { r, g, b, a })
    }

    fn parse_number_item_at_py(
        vm: &VirtualMachine,
        item: &PyObjectRef,
        context: &str,
        index: usize,
    ) -> PyResult<f32> {
        if let Some(float) = item.payload_if_subclass::<PyFloat>(vm) {
            return Ok(float.to_f64() as f32);
        }
        let value: f64 = item.clone().try_into_value(vm).map_err(|_| {
            vm.new_value_error(format!("{context}: expected float at index {index}"))
        })?;
        Ok(value as f32)
    }

    fn parse_color_py(vm: &VirtualMachine, object: &PyObjectRef, context: &str) -> PyResult<Color> {
        if let Some(tuple) = object.payload_if_subclass::<PyTuple>(vm) {
            return Self::parse_color_from_items_py(vm, tuple.as_slice(), context);
        }
        if let Some(list) = object.payload_if_subclass::<PyList>(vm) {
            let items = list.borrow_vec();
            return Self::parse_color_from_items_py(vm, &items, context);
        }
        Self::with_sequence_items_py(vm, object, "Color tuple", |items| {
            Self::parse_color_from_items_py(vm, items, context)
        })
    }

    fn with_sequence_items_py<T>(
        vm: &VirtualMachine,
        object: &PyObjectRef,
        expected: &str,
        f: impl FnOnce(&[PyObjectRef]) -> PyResult<T>,
    ) -> PyResult<T> {
        if let Some(tuple) = object.payload_if_subclass::<PyTuple>(vm) {
            return f(tuple.as_slice());
        }
        if let Some(list) = object.payload_if_subclass::<PyList>(vm) {
            let items = list.borrow_vec();
            return f(&items);
        }

        let values: Vec<PyObjectRef> = object
            .clone()
            .try_into_value(vm)
            .map_err(|_| vm.new_value_error(format!("expected {expected}")))?;
        f(&values)
    }

    fn with_backend_py<T>(
        vm: &VirtualMachine,
        backend: &Arc<Mutex<MacroquadBackendContract>>,
        f: impl FnOnce(&mut MacroquadBackendContract) -> PyResult<T>,
    ) -> PyResult<T> {
        let mut backend = backend.lock();
        let _ = vm;
        f(&mut backend)
    }

    fn queue_draw_op_py(
        vm: &VirtualMachine,
        draw_batch: &Arc<Mutex<QueuedDrawBatch>>,
        op: QueuedDrawOp,
    ) -> PyResult<()> {
        let mut draw_batch = draw_batch.lock();
        let _ = vm;
        draw_batch.push_op(op);
        Ok(())
    }

    fn parse_submit_render_command_py(
        vm: &VirtualMachine,
        command: &PyObjectRef,
        index: usize,
    ) -> PyResult<QueuedDrawOp> {
        Self::with_sequence_items_py(vm, command, "command tuple/list", |fields| {
            if fields.is_empty() {
                return Err(vm.new_value_error(format!(
                    "submit_render commands[{index}]: command must not be empty"
                )));
            }

            let command_name = fields[0]
                .payload_if_subclass::<PyStr>(vm)
                .ok_or_else(|| {
                    vm.new_value_error(format!(
                        "submit_render commands[{index}][0]: expected command name string"
                    ))
                })?
                .as_str();

            match command_name {
                "clear_background" => {
                    if fields.len() != 2 {
                        return Err(vm.new_value_error(format!(
                            "submit_render commands[{index}]: clear_background expects 1 argument"
                        )));
                    }
                    let color = Self::parse_color_py(
                        vm,
                        &fields[1],
                        &format!("submit_render commands[{index}] clear_background"),
                    )?;
                    Ok(QueuedDrawOp::ClearBackground(color))
                }
                "draw_circle" => {
                    if fields.len() != 4 {
                        return Err(vm.new_value_error(format!(
                            "submit_render commands[{index}]: draw_circle expects 3 arguments"
                        )));
                    }
                    let position =
                        Self::parse_vec2_py(vm, &fields[1], "submit_render draw_circle position")?;
                    let radius: f64 = fields[2].clone().try_into_value(vm).map_err(|_| {
                        vm.new_value_error(format!(
                            "submit_render commands[{index}] draw_circle radius: expected float"
                        ))
                    })?;
                    let color =
                        Self::parse_color_py(vm, &fields[3], "submit_render draw_circle color")?;
                    Ok(QueuedDrawOp::DrawCircle {
                        position,
                        radius: radius as f32,
                        color,
                        render_mode: VectorRenderMode::Default,
                    })
                }
                "draw_texture" => {
                    if fields.len() != 4 {
                        return Err(vm.new_value_error(format!(
                            "submit_render commands[{index}]: draw_texture expects 3 arguments"
                        )));
                    }
                    let texture: String = fields[1].clone().try_into_value(vm).map_err(|_| {
                        vm.new_value_error(format!(
                            "submit_render commands[{index}] draw_texture texture: expected TextureHandle"
                        ))
                    })?;
                    let position =
                        Self::parse_vec2_py(vm, &fields[2], "submit_render draw_texture position")?;
                    let size =
                        Self::parse_vec2_py(vm, &fields[3], "submit_render draw_texture size")?;
                    Ok(QueuedDrawOp::DrawTexture {
                        texture,
                        position,
                        size,
                    })
                }
                "set_camera_target" => {
                    if fields.len() != 2 {
                        return Err(vm.new_value_error(format!(
                            "submit_render commands[{index}]: set_camera_target expects 1 argument"
                        )));
                    }
                    let target =
                        Self::parse_vec2_py(vm, &fields[1], "submit_render set_camera_target")?;
                    Ok(QueuedDrawOp::SetCameraTarget(target))
                }
                "draw_text" => {
                    if Self::draw_text_noop_enabled() {
                        return Ok(QueuedDrawOp::DrawText {
                            text: String::new(),
                            position: Vec2 { x: 0.0, y: 0.0 },
                            font_size: 0.0,
                            color: Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            },
                        });
                    }
                    if fields.len() != 5 {
                        return Err(vm.new_value_error(format!(
                            "submit_render commands[{index}]: draw_text expects 4 arguments"
                        )));
                    }
                    let text: String = fields[1].clone().try_into_value(vm).map_err(|_| {
                        vm.new_value_error(format!(
                            "submit_render commands[{index}] draw_text text: expected str"
                        ))
                    })?;
                    let position =
                        Self::parse_vec2_py(vm, &fields[2], "submit_render draw_text position")?;
                    let font_size: f64 = fields[3].clone().try_into_value(vm).map_err(|_| {
                        vm.new_value_error(format!(
                            "submit_render commands[{index}] draw_text font_size: expected float"
                        ))
                    })?;
                    let color =
                        Self::parse_color_py(vm, &fields[4], "submit_render draw_text color")?;
                    Ok(QueuedDrawOp::DrawText {
                        text,
                        position,
                        font_size: font_size as f32,
                        color,
                    })
                }
                _ => Err(vm.new_value_error(format!(
                    "submit_render commands[{index}]: unsupported command `{command_name}`"
                ))),
            }
        })
    }

    fn queue_submit_circle_batch_py(
        vm: &VirtualMachine,
        draw_batch: &Arc<Mutex<QueuedDrawBatch>>,
        circle_batch_cache: &Arc<Mutex<Option<CircleBatchCache>>>,
        positions: &PyObjectRef,
        radii: &PyObjectRef,
        colors: &PyObjectRef,
        render_mode: VectorRenderMode,
    ) -> PyResult<()> {
        Self::with_sequence_items_py(vm, positions, "list of positions", |position_items| {
            Self::with_sequence_items_py(vm, radii, "list of radii", |radius_items| {
                Self::with_sequence_items_py(vm, colors, "list of colors", |color_items| {
                    if position_items.len() != radius_items.len()
                        || position_items.len() != color_items.len()
                    {
                        return Err(vm.new_value_error(
                            "submit_circle_batch: positions/radii/colors length mismatch"
                                .to_owned(),
                        ));
                    }

                    let radii_list_id = radii.get_id();
                    let colors_list_id = colors.get_id();
                    let mut circle_batch_cache = circle_batch_cache.lock();
                    let needs_rebuild = match circle_batch_cache.as_ref() {
                        Some(cache) => {
                            cache.radii_list_id != radii_list_id
                                || cache.colors_list_id != colors_list_id
                                || cache.radii.len() != position_items.len()
                        }
                        None => true,
                    };
                    if needs_rebuild {
                        let mut parsed_radii = Vec::with_capacity(position_items.len());
                        let mut parsed_colors = Vec::with_capacity(position_items.len());
                        for (index, (radius_obj, color_obj)) in
                            radius_items.iter().zip(color_items.iter()).enumerate()
                        {
                            let radius: f64 =
                                radius_obj.clone().try_into_value(vm).map_err(|_| {
                                    vm.new_value_error(format!(
                                        "submit_circle_batch radii[{index}]: expected float"
                                    ))
                                })?;
                            let color =
                                Self::parse_color_py(vm, color_obj, "submit_circle_batch color")?;
                            parsed_radii.push(radius as f32);
                            parsed_colors.push(color);
                        }
                        *circle_batch_cache = Some(CircleBatchCache {
                            radii_list_id,
                            colors_list_id,
                            radii: parsed_radii,
                            colors: parsed_colors,
                        });
                    }
                    let cache = circle_batch_cache
                        .as_ref()
                        .expect("cache must exist after rebuild check");

                    let mut draw_batch = draw_batch.lock();
                    let rollback_mark = draw_batch.mark();
                    draw_batch.reserve_ops(position_items.len());
                    let run_start = draw_batch.circles.len();

                    for (index, position_item) in position_items.iter().enumerate() {
                        let position = if let Some(value) =
                            Self::parse_vec2_cached_position_fast_py(vm, position_item)
                        {
                            value
                        } else {
                            match Self::parse_vec2_cached_position_py(
                                vm,
                                position_item,
                                "submit_circle_batch position",
                            ) {
                                Ok(value) => value,
                                Err(error) => {
                                    draw_batch.rollback(rollback_mark);
                                    return Err(error);
                                }
                            }
                        };
                        draw_batch.circles.push(QueuedCircle {
                            position,
                            radius: cache.radii[index],
                            color: cache.colors[index],
                            render_mode,
                        });
                    }
                    draw_batch.finish_circle_run(run_start);
                    Ok(())
                })
            })
        })
    }

    fn parse_submit_render_draw_circle_cache_entry_py(
        vm: &VirtualMachine,
        command: &PyObjectRef,
        index: usize,
    ) -> Option<CachedSubmitRenderCircle> {
        let fields = if let Some(tuple) = command.payload_if_subclass::<PyTuple>(vm) {
            tuple.as_slice().to_vec()
        } else if let Some(list) = command.payload_if_subclass::<PyList>(vm) {
            list.borrow_vec().to_vec()
        } else {
            return None;
        };
        if fields.len() != 4 {
            return None;
        }
        let command_name = fields[0].payload_if_subclass::<PyStr>(vm)?.as_str();
        if command_name != "draw_circle" {
            return None;
        }
        let radius: f64 = fields[2].clone().try_into_value(vm).ok()?;
        let color = Self::parse_color_py(vm, &fields[3], "submit_render draw_circle color").ok()?;
        Some(CachedSubmitRenderCircle {
            index,
            command_id: command.get_id(),
            position_obj: fields[1].clone(),
            radius: radius as f32,
            color,
        })
    }

    fn rebuild_submit_render_circle_cache_py(
        vm: &VirtualMachine,
        submit_render_circle_cache: &Arc<Mutex<Option<Arc<SubmitRenderCircleCache>>>>,
        commands: &PyObjectRef,
        items: &[PyObjectRef],
    ) {
        let mut circles = Vec::new();
        for (index, command) in items.iter().enumerate() {
            if let Some(entry) =
                Self::parse_submit_render_draw_circle_cache_entry_py(vm, command, index)
            {
                circles.push(entry);
            }
        }
        if circles.is_empty() {
            return;
        }
        let mut layout = Vec::new();
        let mut item_index = 0usize;
        let mut circle_index = 0usize;
        while item_index < items.len() {
            if circles
                .get(circle_index)
                .is_some_and(|circle| circle.index == item_index)
            {
                let run_start = circle_index;
                let mut run_len = 0usize;
                while circles
                    .get(circle_index)
                    .is_some_and(|circle| circle.index == item_index + run_len)
                {
                    circle_index += 1;
                    run_len += 1;
                }
                layout.push(CachedSubmitRenderLayoutEntry::CircleRun {
                    start: run_start,
                    len: run_len,
                });
                item_index += run_len;
            } else {
                layout.push(CachedSubmitRenderLayoutEntry::NonCircle {
                    command_index: item_index,
                });
                item_index += 1;
            }
        }
        let mut cache_guard = submit_render_circle_cache.lock();
        #[allow(clippy::arc_with_non_send_sync)]
        let cache = Arc::new(SubmitRenderCircleCache {
            commands_list_id: commands.get_id(),
            command_count: items.len(),
            first_circle_command_id: circles
                .first()
                .map(|entry| entry.command_id)
                .unwrap_or_default(),
            last_circle_command_id: circles
                .last()
                .map(|entry| entry.command_id)
                .unwrap_or_default(),
            circles,
            layout,
        });
        *cache_guard = Some(cache);
    }

    fn queue_submit_render_from_circle_cache_py(
        vm: &VirtualMachine,
        draw_batch: &Arc<Mutex<QueuedDrawBatch>>,
        submit_render_circle_cache: &Arc<Mutex<Option<Arc<SubmitRenderCircleCache>>>>,
        commands: &PyObjectRef,
        items: &[PyObjectRef],
    ) -> PyResult<bool> {
        let cache = {
            let cache_guard = submit_render_circle_cache.lock();
            cache_guard.clone()
        };
        let Some(cache) = cache else {
            return Ok(false);
        };
        if cache.commands_list_id != commands.get_id() || cache.command_count != items.len() {
            return Ok(false);
        }
        let Some(first_circle) = cache.circles.first() else {
            return Ok(false);
        };
        let Some(last_circle) = cache.circles.last() else {
            return Ok(false);
        };
        if items
            .get(first_circle.index)
            .is_none_or(|command| command.get_id() != cache.first_circle_command_id)
            || items
                .get(last_circle.index)
                .is_none_or(|command| command.get_id() != cache.last_circle_command_id)
        {
            return Ok(false);
        }

        let mut draw_batch = draw_batch.lock();
        let rollback_mark = draw_batch.mark();
        draw_batch.reserve_ops(items.len());

        for entry in &cache.layout {
            match *entry {
                CachedSubmitRenderLayoutEntry::CircleRun { start, len } => {
                    let run_start = draw_batch.circles.len();
                    for circle in &cache.circles[start..(start + len)] {
                        let position = if let Some(value) =
                            Self::parse_vec2_cached_position_fast_py(vm, &circle.position_obj)
                        {
                            value
                        } else {
                            match Self::parse_vec2_cached_position_py(
                                vm,
                                &circle.position_obj,
                                "submit_render draw_circle position",
                            ) {
                                Ok(value) => value,
                                Err(error) => {
                                    draw_batch.rollback(rollback_mark);
                                    return Err(error);
                                }
                            }
                        };
                        draw_batch.circles.push(QueuedCircle {
                            position,
                            radius: circle.radius,
                            color: circle.color,
                            render_mode: VectorRenderMode::Default,
                        });
                    }
                    draw_batch.finish_circle_run(run_start);
                }
                CachedSubmitRenderLayoutEntry::NonCircle { command_index } => {
                    match Self::parse_submit_render_command_py(
                        vm,
                        &items[command_index],
                        command_index,
                    ) {
                        Ok(op) => draw_batch.push_op(op),
                        Err(error) => {
                            draw_batch.rollback(rollback_mark);
                            return Err(error);
                        }
                    }
                }
            }
        }
        Ok(true)
    }

    fn dispatch_submit_render_direct_py(
        vm: &VirtualMachine,
        backend: &Arc<Mutex<MacroquadBackendContract>>,
        submit_render_circle_cache: &Arc<Mutex<Option<Arc<SubmitRenderCircleCache>>>>,
        commands: &PyObjectRef,
        items: &[PyObjectRef],
    ) -> PyResult<()> {
        let cache = {
            let cache_guard = submit_render_circle_cache.lock();
            cache_guard.clone()
        };

        if let Some(cache) = cache
            && cache.commands_list_id == commands.get_id()
            && cache.command_count == items.len()
            && !cache.circles.is_empty()
            && cache.circles.last().is_some()
            && items
                .get(cache.circles.first().map(|c| c.index).unwrap_or_default())
                .is_some_and(|command| command.get_id() == cache.first_circle_command_id)
            && items
                .get(cache.circles.last().map(|c| c.index).unwrap_or_default())
                .is_some_and(|command| command.get_id() == cache.last_circle_command_id)
        {
            let mut backend = backend.lock();
            for entry in &cache.layout {
                match *entry {
                    CachedSubmitRenderLayoutEntry::CircleRun { start, len } => {
                        let mut circles = Vec::with_capacity(len);
                        for circle in &cache.circles[start..(start + len)] {
                            let position =
                                Self::parse_vec2_cached_position_fast_py(vm, &circle.position_obj)
                                    .or_else(|| {
                                        Self::parse_vec2_cached_position_py(
                                            vm,
                                            &circle.position_obj,
                                            "submit_render draw_circle position",
                                        )
                                        .ok()
                                    })
                                    .ok_or_else(|| {
                                        vm.new_value_error(
                                            "submit_render draw_circle position parse failed"
                                                .to_owned(),
                                        )
                                    })?;
                            circles.push(CircleDraw {
                                position,
                                radius: circle.radius,
                                color: circle.color,
                                render_mode: VectorRenderMode::Default,
                            });
                        }
                        if circles.len() == 1 {
                            let circle = circles[0];
                            backend.draw_circle(
                                circle.position,
                                circle.radius,
                                circle.color,
                                circle.render_mode,
                            );
                        } else if !circles.is_empty() {
                            backend.draw_circle_batch(&circles);
                        }
                    }
                    CachedSubmitRenderLayoutEntry::NonCircle { command_index } => {
                        match Self::parse_submit_render_command_py(
                            vm,
                            &items[command_index],
                            command_index,
                        )? {
                            QueuedDrawOp::ClearBackground(color) => backend.clear_background(color),
                            QueuedDrawOp::DrawCircle {
                                position,
                                radius,
                                color,
                                render_mode,
                            } => backend.draw_circle(position, radius, color, render_mode),
                            QueuedDrawOp::DrawTexture {
                                texture,
                                position,
                                size,
                            } => backend.draw_texture(&TextureHandle(texture), position, size),
                            QueuedDrawOp::SetCameraTarget(target) => {
                                backend.set_camera_target(target)
                            }
                            QueuedDrawOp::DrawText {
                                text,
                                position,
                                font_size,
                                color,
                            } => backend.draw_text(text.as_str(), position, font_size, color),
                        }
                    }
                }
            }
            return Ok(());
        }

        let mut backend = backend.lock();
        for (index, command) in items.iter().enumerate() {
            match Self::parse_submit_render_command_py(vm, command, index)? {
                QueuedDrawOp::ClearBackground(color) => backend.clear_background(color),
                QueuedDrawOp::DrawCircle {
                    position,
                    radius,
                    color,
                    render_mode,
                } => backend.draw_circle(position, radius, color, render_mode),
                QueuedDrawOp::DrawTexture {
                    texture,
                    position,
                    size,
                } => backend.draw_texture(&TextureHandle(texture), position, size),
                QueuedDrawOp::SetCameraTarget(target) => backend.set_camera_target(target),
                QueuedDrawOp::DrawText {
                    text,
                    position,
                    font_size,
                    color,
                } => backend.draw_text(text.as_str(), position, font_size, color),
            }
        }
        Self::rebuild_submit_render_circle_cache_py(
            vm,
            submit_render_circle_cache,
            commands,
            items,
        );
        Ok(())
    }

    fn flush_draw_batch_ops(&mut self) -> Result<(), RuntimeError> {
        let mut backend = self.backend.lock();

        let mut draw_batch = self.draw_batch.lock();

        for entry in &draw_batch.entries {
            match entry {
                QueuedBatchEntry::Op(op) => match *op {
                    QueuedDrawOp::ClearBackground(color) => backend.clear_background(color),
                    QueuedDrawOp::DrawCircle {
                        position,
                        radius,
                        color,
                        render_mode,
                    } => backend.draw_circle(position, radius, color, render_mode),
                    QueuedDrawOp::DrawTexture {
                        ref texture,
                        position,
                        size,
                    } => backend.draw_texture(&TextureHandle(texture.clone()), position, size),
                    QueuedDrawOp::SetCameraTarget(target) => backend.set_camera_target(target),
                    QueuedDrawOp::DrawText {
                        ref text,
                        position,
                        font_size,
                        color,
                    } => backend.draw_text(text.as_str(), position, font_size, color),
                },
                QueuedBatchEntry::CircleRun { start, len } => {
                    backend.draw_circle_batch(&draw_batch.circles[*start..(*start + *len)]);
                }
            }
        }
        draw_batch.clear();

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn install_direct_api_functions(
        vm: &VirtualMachine,
        module_dict: &PyDictRef,
        plan: &ModuleInstallPlan,
        backend: Arc<Mutex<MacroquadBackendContract>>,
        draw_batch: Arc<Mutex<QueuedDrawBatch>>,
        submit_render_circle_cache: Arc<Mutex<Option<Arc<SubmitRenderCircleCache>>>>,
        circle_batch_cache: Arc<Mutex<Option<CircleBatchCache>>>,
        frame_time_seconds: Arc<AtomicU32>,
    ) -> Result<(), RuntimeError> {
        for function_name in &plan.exported_function_names {
            let function_obj = match *function_name {
                "clear_background" => {
                    let draw_batch = Arc::clone(&draw_batch);
                    vm.new_function(
                        "clear_background",
                        move |color: PyObjectRef, vm: &VirtualMachine| {
                            let color = Self::parse_color_py(vm, &color, "clear_background")?;
                            Self::queue_draw_op_py(
                                vm,
                                &draw_batch,
                                QueuedDrawOp::ClearBackground(color),
                            )
                        },
                    )
                    .into()
                }
                "draw_circle" => {
                    let draw_batch = Arc::clone(&draw_batch);
                    vm.new_function(
                        "draw_circle",
                        move |position: PyObjectRef,
                              radius: f64,
                              color: PyObjectRef,
                              vm: &VirtualMachine| {
                            let position =
                                Self::parse_vec2_py(vm, &position, "draw_circle position")?;
                            let color = Self::parse_color_py(vm, &color, "draw_circle color")?;
                            Self::queue_draw_op_py(
                                vm,
                                &draw_batch,
                                QueuedDrawOp::DrawCircle {
                                    position,
                                    radius: radius as f32,
                                    color,
                                    render_mode: VectorRenderMode::Default,
                                },
                            )
                        },
                    )
                    .into()
                }
                "is_key_down" => {
                    let backend = Arc::clone(&backend);
                    vm.new_function("is_key_down", move |key: String, vm: &VirtualMachine| {
                        Self::with_backend_py(vm, &backend, |backend| Ok(backend.is_key_down(&key)))
                    })
                    .into()
                }
                "frame_time" => {
                    let frame_time_seconds = Arc::clone(&frame_time_seconds);
                    vm.new_function("frame_time", move |_vm: &VirtualMachine| -> PyResult<f64> {
                        let dt = f32::from_bits(frame_time_seconds.load(Ordering::Relaxed));
                        Ok(f64::from(dt))
                    })
                    .into()
                }
                "load_texture" => {
                    let backend = Arc::clone(&backend);
                    vm.new_function("load_texture", move |path: String, vm: &VirtualMachine| {
                        Self::with_backend_py(vm, &backend, |backend| {
                            let handle = backend
                                .load_texture(&path)
                                .map_err(|error| vm.new_runtime_error(error))?;
                            Ok(handle.0)
                        })
                    })
                    .into()
                }
                "draw_texture" => {
                    let draw_batch = Arc::clone(&draw_batch);
                    vm.new_function(
                        "draw_texture",
                        move |texture: String,
                              position: PyObjectRef,
                              size: PyObjectRef,
                              vm: &VirtualMachine| {
                            let position =
                                Self::parse_vec2_py(vm, &position, "draw_texture position")?;
                            let size = Self::parse_vec2_py(vm, &size, "draw_texture size")?;
                            Self::queue_draw_op_py(
                                vm,
                                &draw_batch,
                                QueuedDrawOp::DrawTexture {
                                    texture,
                                    position,
                                    size,
                                },
                            )
                        },
                    )
                    .into()
                }
                "set_camera_target" => {
                    let draw_batch = Arc::clone(&draw_batch);
                    vm.new_function(
                        "set_camera_target",
                        move |target: PyObjectRef, vm: &VirtualMachine| {
                            let target = Self::parse_vec2_py(vm, &target, "set_camera_target")?;
                            Self::queue_draw_op_py(
                                vm,
                                &draw_batch,
                                QueuedDrawOp::SetCameraTarget(target),
                            )
                        },
                    )
                    .into()
                }
                "draw_text" => {
                    let draw_batch = Arc::clone(&draw_batch);
                    vm.new_function(
                        "draw_text",
                        move |text: String,
                              position: PyObjectRef,
                              font_size: f64,
                              color: PyObjectRef,
                              vm: &VirtualMachine| {
                            let position =
                                Self::parse_vec2_py(vm, &position, "draw_text position")?;
                            let color = Self::parse_color_py(vm, &color, "draw_text color")?;
                            Self::queue_draw_op_py(
                                vm,
                                &draw_batch,
                                QueuedDrawOp::DrawText {
                                    text,
                                    position,
                                    font_size: font_size as f32,
                                    color,
                                },
                            )
                        },
                    )
                    .into()
                }
                "submit_render" => {
                    let draw_batch = Arc::clone(&draw_batch);
                    let backend = Arc::clone(&backend);
                    let submit_render_circle_cache = Arc::clone(&submit_render_circle_cache);
                    vm.new_function(
                        "submit_render",
                        move |commands: PyObjectRef, vm: &VirtualMachine| {
                            if Self::submit_render_noop_enabled() {
                                let _ = (&commands, vm);
                                return Ok(());
                            }
                            if Self::submit_render_direct_enabled() {
                                return Self::with_sequence_items_py(
                                    vm,
                                    &commands,
                                    "list of commands",
                                    |items| {
                                        Self::dispatch_submit_render_direct_py(
                                            vm,
                                            &backend,
                                            &submit_render_circle_cache,
                                            &commands,
                                            items,
                                        )
                                    },
                                );
                            }
                            Self::with_sequence_items_py(
                                vm,
                                &commands,
                                "list of commands",
                                |items| {
                                    if Self::queue_submit_render_from_circle_cache_py(
                                        vm,
                                        &draw_batch,
                                        &submit_render_circle_cache,
                                        &commands,
                                        items,
                                    )? {
                                        return Ok(());
                                    }

                                    let mut draw_batch = draw_batch.lock();
                                    let rollback_mark = draw_batch.mark();
                                    draw_batch.reserve_ops(items.len());
                                    for (index, command) in items.iter().enumerate() {
                                        match Self::parse_submit_render_command_py(
                                            vm, command, index,
                                        ) {
                                            Ok(op) => draw_batch.push_op(op),
                                            Err(error) => {
                                                draw_batch.rollback(rollback_mark);
                                                return Err(error);
                                            }
                                        }
                                    }
                                    drop(draw_batch);
                                    Self::rebuild_submit_render_circle_cache_py(
                                        vm,
                                        &submit_render_circle_cache,
                                        &commands,
                                        items,
                                    );
                                    Ok(())
                                },
                            )
                        },
                    )
                    .into()
                }
                "submit_circle_batch" => {
                    let draw_batch = Arc::clone(&draw_batch);
                    let circle_batch_cache = Arc::clone(&circle_batch_cache);
                    vm.new_function(
                        "submit_circle_batch",
                        move |positions: PyObjectRef,
                              radii: PyObjectRef,
                              colors: PyObjectRef,
                              vm: &VirtualMachine| {
                            Self::queue_submit_circle_batch_py(
                                vm,
                                &draw_batch,
                                &circle_batch_cache,
                                &positions,
                                &radii,
                                &colors,
                                VectorRenderMode::Default,
                            )
                        },
                    )
                    .into()
                }
                _ => {
                    return Err(RuntimeError::FunctionCall {
                        function: format!("module function install: {function_name}"),
                        details: "missing runtime direct-bridge binding for API metadata entry"
                            .to_owned(),
                    });
                }
            };

            module_dict
                .set_item(*function_name, function_obj, vm)
                .map_err(|error| RuntimeError::FunctionCall {
                    function: format!("module function install: {function_name}"),
                    details: Self::exception_details(vm, &error),
                })?;
        }

        Ok(())
    }

    fn configure_import_path_for_script(
        vm: &VirtualMachine,
        path: &str,
    ) -> Result<(), RuntimeError> {
        let script_dir = Path::new(path)
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let script_dir =
            fs::canonicalize(script_dir).unwrap_or_else(|_| Path::new(".").to_path_buf());
        let script_dir = script_dir.to_string_lossy().to_string();
        let sys_path =
            vm.sys_module
                .get_attr("path", vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
        vm.call_method(sys_path.as_object(), "insert", (0, script_dir))
            .map_err(|error| RuntimeError::ScriptLoad {
                path: path.to_owned(),
                details: Self::exception_details(vm, &error),
            })?;
        Ok(())
    }

    fn install_stdlib_compat_modules(vm: &VirtualMachine, path: &str) -> Result<(), RuntimeError> {
        let sys_modules =
            vm.sys_module
                .get_attr("modules", vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;

        let script_dir = Path::new(path).parent().unwrap_or_else(|| Path::new("."));

        if !script_dir.join("math.py").exists() {
            let math_attrs = vm.ctx.new_dict();
            math_attrs
                .set_item("__name__", vm.ctx.new_str("math").into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            math_attrs
                .set_item("pi", vm.ctx.new_float(std::f64::consts::PI).into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            let math_sqrt = vm.new_function("sqrt", move |value: f64, vm: &VirtualMachine| {
                if value < 0.0 {
                    return Err(vm.new_value_error("math domain error".to_owned()));
                }
                Ok(value.sqrt())
            });
            let math_sin = vm.new_function("sin", move |value: f64| value.sin());
            let math_cos = vm.new_function("cos", move |value: f64| value.cos());
            let math_hypot = vm.new_function("hypot", move |x: f64, y: f64| x.hypot(y));
            math_attrs
                .set_item("sqrt", math_sqrt.into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            math_attrs
                .set_item("sin", math_sin.into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            math_attrs
                .set_item("cos", math_cos.into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            math_attrs
                .set_item("hypot", math_hypot.into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            let math_module = vm.new_module("math", math_attrs.clone(), None);
            sys_modules
                .set_item("math", math_module.into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
        }

        if !script_dir.join("os.py").exists() {
            let os_attrs = vm.ctx.new_dict();
            os_attrs
                .set_item("__name__", vm.ctx.new_str("os").into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            os_attrs
                .set_item(
                    "name",
                    vm.ctx.new_str(std::env::consts::OS.to_owned()).into(),
                    vm,
                )
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            os_attrs
                .set_item(
                    "sep",
                    vm.ctx.new_str(std::path::MAIN_SEPARATOR.to_string()).into(),
                    vm,
                )
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            let pathsep = if cfg!(windows) { ";" } else { ":" };
            os_attrs
                .set_item("pathsep", vm.ctx.new_str(pathsep).into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            let linesep = if cfg!(windows) { "\r\n" } else { "\n" };
            os_attrs
                .set_item("linesep", vm.ctx.new_str(linesep).into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            let os_getcwd = vm.new_function("getcwd", move |vm: &VirtualMachine| {
                std::env::current_dir()
                    .map(|cwd| cwd.to_string_lossy().into_owned())
                    .map_err(|error| vm.new_runtime_error(format!("os.getcwd failed: {error}")))
            });
            let os_getenv = vm
                .new_function("getenv", move |key: String, default: Option<String>| {
                    std::env::var(&key).ok().or(default)
                });
            os_attrs
                .set_item("getcwd", os_getcwd.into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            os_attrs
                .set_item("getenv", os_getenv.into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;

            let os_path_attrs = vm.ctx.new_dict();
            os_path_attrs
                .set_item("__name__", vm.ctx.new_str("os.path").into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            let os_path_basename = vm.new_function("basename", move |value: String| {
                Path::new(value.as_str())
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default()
            });
            os_path_attrs
                .set_item("basename", os_path_basename.into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            let os_path_module = vm.new_module("os.path", os_path_attrs.clone(), None);
            sys_modules
                .set_item("os.path", os_path_module.clone().into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            os_attrs
                .set_item("path", os_path_module.into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;

            let os_module = vm.new_module("os", os_attrs.clone(), None);
            sys_modules
                .set_item("os", os_module.into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
        }

        Ok(())
    }

    fn imported_sidecar_module_names(source: &str) -> Vec<String> {
        let mut modules = Vec::new();
        for raw_line in source.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(remainder) = line.strip_prefix("import ") {
                for chunk in remainder.split(',') {
                    let name = chunk
                        .split_whitespace()
                        .next()
                        .unwrap_or_default()
                        .split('.')
                        .next()
                        .unwrap_or_default();
                    if !name.is_empty() {
                        modules.push(name.to_owned());
                    }
                }
                continue;
            }
            if let Some(remainder) = line.strip_prefix("from ") {
                let name = remainder
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .split('.')
                    .next()
                    .unwrap_or_default();
                if !name.is_empty() {
                    modules.push(name.to_owned());
                }
            }
        }
        modules.sort();
        modules.dedup();
        modules
    }

    fn preload_sidecar_modules_for_script(
        vm: &VirtualMachine,
        path: &str,
        entry_source: &str,
    ) -> Result<(), RuntimeError> {
        let sys_modules =
            vm.sys_module
                .get_attr("modules", vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;

        let script_path = Path::new(path);
        let script_dir = script_path.parent().unwrap_or_else(|| Path::new("."));

        fn preload_one_sidecar_module(
            vm: &VirtualMachine,
            path: &str,
            script_dir: &Path,
            sys_modules: &PyObjectRef,
            module_name: &str,
            visiting: &mut HashSet<String>,
            loaded: &mut HashSet<String>,
        ) -> Result<(), RuntimeError> {
            if loaded.contains(module_name) || visiting.contains(module_name) {
                return Ok(());
            }
            let module_path = script_dir.join(format!("{module_name}.py"));
            if !module_path.exists() {
                return Ok(());
            }

            visiting.insert(module_name.to_owned());

            let source =
                fs::read_to_string(&module_path).map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: format!(
                        "failed to read sidecar module {}: {error}",
                        module_path.display()
                    ),
                })?;

            for dependency in RustPythonVm::imported_sidecar_module_names(&source) {
                preload_one_sidecar_module(
                    vm,
                    path,
                    script_dir,
                    sys_modules,
                    dependency.as_str(),
                    visiting,
                    loaded,
                )?;
            }

            let attrs = vm.ctx.new_dict();
            attrs
                .set_item("__name__", vm.ctx.new_str(module_name).into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: RustPythonVm::exception_details(vm, &error),
                })?;
            attrs
                .set_item(
                    "__file__",
                    vm.ctx
                        .new_str(module_path.to_string_lossy().as_ref())
                        .into(),
                    vm,
                )
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: RustPythonVm::exception_details(vm, &error),
                })?;

            let module = vm.new_module(module_name, attrs.clone(), None);
            sys_modules
                .set_item(module_name, module.into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: RustPythonVm::exception_details(vm, &error),
                })?;

            let module_scope = Scope::with_builtins(None, attrs, vm);
            vm.run_code_string(
                module_scope,
                &source,
                module_path.to_string_lossy().into_owned(),
            )
            .map_err(|error| RuntimeError::ScriptLoad {
                path: path.to_owned(),
                details: RustPythonVm::exception_details(vm, &error),
            })?;

            visiting.remove(module_name);
            loaded.insert(module_name.to_owned());
            Ok(())
        }

        let mut visiting = HashSet::new();
        let mut loaded = HashSet::new();
        for module_name in Self::imported_sidecar_module_names(entry_source) {
            preload_one_sidecar_module(
                vm,
                path,
                script_dir,
                &sys_modules,
                module_name.as_str(),
                &mut visiting,
                &mut loaded,
            )?;
        }
        Ok(())
    }

    fn maybe_disable_gc(vm: &VirtualMachine, path: &str) -> Result<(), RuntimeError> {
        if !std::env::var("PYCRO_DISABLE_GC").is_ok_and(|value| value == "1") {
            return Ok(());
        }
        let _ = vm.run_code_string(
            vm.new_scope_with_builtins(),
            "import gc\ngc.disable()\n",
            "<pycro-gc-config>".to_owned(),
        );
        let _ = path;
        Ok(())
    }

    fn maybe_jit_functions(
        vm: &VirtualMachine,
        scope: &Scope,
        update_callable: &Option<PyObjectRef>,
    ) -> Result<(), RuntimeError> {
        let jit_mode = std::env::var("PYCRO_JIT_MODE").unwrap_or_else(|_| "off".to_owned());
        let jit_report = std::env::var("PYCRO_JIT_REPORT").is_ok_and(|value| value == "1");
        if !cfg!(target_arch = "x86_64") {
            if jit_mode != "off" && jit_report {
                eprintln!("[pycro-jit] unsupported target_arch for runtime __jit__ path");
            }
            return Ok(());
        }
        if jit_mode == "off" {
            return Ok(());
        }
        if jit_mode == "update" {
            if let Some(update_callable) = update_callable {
                match vm.call_method(update_callable.as_object(), "__jit__", ()) {
                    Ok(_) => {
                        if jit_report {
                            eprintln!("[pycro-jit] update:ok");
                        }
                    }
                    Err(error) => {
                        if jit_report {
                            eprintln!(
                                "[pycro-jit] update:err {}",
                                Self::exception_details(vm, &error)
                            );
                        }
                    }
                }
            } else if jit_report {
                eprintln!("[pycro-jit] update:missing");
            }
            return Ok(());
        }
        if jit_mode == "all" {
            let py_bool = if jit_report { "True" } else { "False" };
            let jit_script = format!(
                r#"
_jit_ok = []
_jit_err = []
_callables = []
_with_jit = []
for _name, _obj in list(globals().items()):
    if callable(_obj):
        _callables.append(_name)
    if hasattr(_obj, "__jit__"):
        _with_jit.append(_name)
        try:
            _obj.__jit__()
            _jit_ok.append(_name)
        except Exception:
            _jit_err.append(_name)
if {py_bool}:
    print("[pycro-jit] all:callables=" + ",".join(_callables))
    print("[pycro-jit] all:has_jit=" + ",".join(_with_jit))
    print("[pycro-jit] all:ok=" + ",".join(_jit_ok))
    print("[pycro-jit] all:err=" + ",".join(_jit_err))
"#
            );
            let _ = vm.run_code_string(scope.clone(), &jit_script, "<pycro-jit-all>".to_owned());
            return Ok(());
        }
        Ok(())
    }
}

impl PythonVm for RustPythonVm {
    fn install_module(&mut self, plan: ModuleInstallPlan) -> Result<(), RuntimeError> {
        let backend = Arc::clone(&self.backend);
        let draw_batch = Arc::clone(&self.draw_batch);
        let submit_render_circle_cache = Arc::clone(&self.submit_render_circle_cache);
        let circle_batch_cache = Arc::clone(&self.circle_batch_cache);
        let frame_time_seconds = Arc::clone(&self.frame_time_seconds);
        let scope = self.interpreter.enter(|vm| {
            let scope = vm.new_scope_with_builtins();

            let attrs = vm.ctx.new_dict();
            attrs
                .set_item("__name__", vm.ctx.new_str(plan.module_name).into(), vm)
                .map_err(|error| RuntimeError::FunctionCall {
                    function: "module.__name__".to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;

            let module = vm.new_module(plan.module_name, attrs.clone(), None);
            let sys_modules = vm.sys_module.get_attr("modules", vm).map_err(|error| {
                RuntimeError::FunctionCall {
                    function: "sys.modules".to_owned(),
                    details: Self::exception_details(vm, &error),
                }
            })?;

            sys_modules
                .set_item(plan.module_name, module.into(), vm)
                .map_err(|error| RuntimeError::FunctionCall {
                    function: "sys.modules[pycro]".to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;

            let module_scope = Scope::with_builtins(None, attrs.clone(), vm);
            vm.run_code_string(
                module_scope,
                Self::module_bootstrap_source(),
                "<pycro-module>".to_owned(),
            )
            .map_err(|error| RuntimeError::FunctionCall {
                function: "pycro module bootstrap".to_owned(),
                details: Self::exception_details(vm, &error),
            })?;

            Self::install_direct_api_functions(
                vm,
                &attrs,
                &plan,
                backend,
                draw_batch,
                submit_render_circle_cache,
                circle_batch_cache,
                frame_time_seconds,
            )?;

            Ok(scope)
        })?;

        self.scope = Some(scope);
        self.setup_callable = None;
        self.update_callable = None;
        Ok(())
    }

    fn load_script(&mut self, path: &str) -> Result<(), RuntimeError> {
        let source = fs::read_to_string(path).map_err(|error| RuntimeError::ScriptLoad {
            path: path.to_owned(),
            details: error.to_string(),
        })?;

        let scope = self.scope.clone().ok_or(RuntimeError::NotLoaded)?;
        let (setup_callable, update_callable) = self.with_scope(scope, |vm, scope| {
            Self::configure_import_path_for_script(vm, path)?;
            Self::install_stdlib_compat_modules(vm, path)?;
            Self::maybe_disable_gc(vm, path)?;
            Self::preload_sidecar_modules_for_script(vm, path, &source)?;
            scope
                .globals
                .set_item("__file__", vm.ctx.new_str(path).into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            vm.run_code_string(scope.clone(), &source, path.to_owned())
                .map(|_| ())
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            let setup_callable = scope
                .globals
                .get_item_opt(SETUP_FUNCTION, vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?
                .filter(|value| value.as_object().to_callable().is_some());
            let update_callable = scope
                .globals
                .get_item_opt(UPDATE_FUNCTION, vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?
                .filter(|value| value.as_object().to_callable().is_some());
            Self::maybe_jit_functions(vm, &scope, &update_callable)?;
            Self::flush_stdio(vm);
            Ok((setup_callable, update_callable))
        })?;
        self.setup_callable = setup_callable;
        self.update_callable = update_callable;
        Ok(())
    }

    fn has_function(&self, function: &str) -> Result<bool, RuntimeError> {
        let scope = self.scope.clone().ok_or(RuntimeError::NotLoaded)?;
        self.with_scope(scope, |vm, scope| {
            let maybe_value = scope.globals.get_item_opt(function, vm).map_err(|error| {
                RuntimeError::FunctionCall {
                    function: function.to_owned(),
                    details: Self::exception_details(vm, &error),
                }
            })?;
            Ok(maybe_value
                .as_ref()
                .is_some_and(|value| value.as_object().to_callable().is_some()))
        })
    }

    fn call_function(&mut self, function: &str, args: &[RuntimeValue]) -> Result<(), RuntimeError> {
        let scope = self.scope.clone().ok_or(RuntimeError::NotLoaded)?;
        let cached_callable = match function {
            SETUP_FUNCTION => self.setup_callable.clone(),
            UPDATE_FUNCTION => self.update_callable.clone(),
            _ => None,
        };
        let frame_time_seconds = Arc::clone(&self.frame_time_seconds);
        let flush_stdio_on_update = self.flush_stdio_on_update;
        self.with_scope(scope, |vm, scope| {
            let callable = if let Some(callable) = cached_callable.clone() {
                callable
            } else {
                scope
                    .globals
                    .get_item_opt(function, vm)
                    .map_err(|error| RuntimeError::FunctionCall {
                        function: function.to_owned(),
                        details: Self::exception_details(vm, &error),
                    })?
                    .ok_or_else(|| RuntimeError::FunctionCall {
                        function: function.to_owned(),
                        details: "function not found in loaded script".to_owned(),
                    })?
            };

            if let [RuntimeValue::Float(dt)] = args {
                frame_time_seconds.store(dt.to_bits(), Ordering::Relaxed);
            }

            match args {
                [] => callable.call((), vm),
                [RuntimeValue::Float(dt)] => callable.call((f64::from(*dt),), vm),
                _ => {
                    return Err(RuntimeError::FunctionCall {
                        function: function.to_owned(),
                        details: "unsupported runtime argument shape".to_owned(),
                    });
                }
            }
            .map_err(|error| RuntimeError::FunctionCall {
                function: function.to_owned(),
                details: Self::exception_details(vm, &error),
            })?;

            if function != UPDATE_FUNCTION || flush_stdio_on_update {
                Self::flush_stdio(vm);
            }
            Ok(())
        })
    }

    fn flush_draw_batch(&mut self) -> Result<(), RuntimeError> {
        self.flush_draw_batch_ops()
    }

    fn discard_draw_batch(&mut self) -> Result<(), RuntimeError> {
        let mut draw_batch = self.draw_batch.lock();
        draw_batch.clear();
        Ok(())
    }

    fn flush_io(&mut self) -> Result<(), RuntimeError> {
        let scope = self.scope.clone().ok_or(RuntimeError::NotLoaded)?;
        self.with_scope(scope, |vm, _scope| {
            Self::flush_stdio(vm);
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ModuleInstallPlan, PythonVm, QueuedDrawOp, RuntimeConfig, RuntimeError, RuntimeValue,
        RustPythonVm, ScriptRuntime,
    };
    use crate::backend::{BackendDispatch, Color, Vec2, VectorRenderMode};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, Default)]
    struct FakeVm {
        setup_present: bool,
        update_present: bool,
        calls: Vec<String>,
        flush_calls: usize,
        discard_calls: usize,
        fail_on_update: bool,
    }

    impl PythonVm for FakeVm {
        fn install_module(&mut self, _plan: ModuleInstallPlan) -> Result<(), RuntimeError> {
            Ok(())
        }

        fn load_script(&mut self, _path: &str) -> Result<(), RuntimeError> {
            Ok(())
        }

        fn has_function(&self, function: &str) -> Result<bool, RuntimeError> {
            Ok(match function {
                "setup" => self.setup_present,
                "update" => self.update_present,
                _ => false,
            })
        }

        fn call_function(
            &mut self,
            function: &str,
            args: &[RuntimeValue],
        ) -> Result<(), RuntimeError> {
            if function == "update" && self.fail_on_update {
                return Err(RuntimeError::FunctionCall {
                    function: "update".to_owned(),
                    details: "simulated update failure".to_owned(),
                });
            }
            let mut label = function.to_owned();
            if let [RuntimeValue::Float(dt)] = args {
                label = format!("{label}({dt:.3})");
            }
            self.calls.push(label);
            Ok(())
        }

        fn flush_draw_batch(&mut self) -> Result<(), RuntimeError> {
            self.flush_calls += 1;
            Ok(())
        }

        fn discard_draw_batch(&mut self) -> Result<(), RuntimeError> {
            self.discard_calls += 1;
            Ok(())
        }

        fn flush_io(&mut self) -> Result<(), RuntimeError> {
            Ok(())
        }
    }

    fn write_temp_script(prefix: &str, source: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "pycro-runtime-{prefix}-{}-{timestamp}.py",
            std::process::id()
        ));
        fs::write(&path, source).expect("temporary script should be writable");
        path
    }

    fn write_temp_project(prefix: &str, files: &[(&str, &str)]) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "pycro-runtime-project-{prefix}-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temporary project root should be creatable");
        for (name, contents) in files {
            let path = root.join(name);
            fs::write(path, contents).expect("temporary project file should be writable");
        }
        root
    }

    fn backend_dispatches(runtime: &ScriptRuntime<RustPythonVm>) -> Vec<BackendDispatch> {
        runtime.vm().backend().dispatch_log().to_vec()
    }

    #[test]
    fn setup_runs_once_and_update_runs_per_frame() {
        let vm = FakeVm {
            setup_present: true,
            update_present: true,
            ..FakeVm::default()
        };
        let mut runtime = ScriptRuntime::new(vm, RuntimeConfig::default());

        runtime.load_main().expect("load_main should succeed");
        runtime.update(0.016).expect("first update should succeed");
        runtime.update(0.032).expect("second update should succeed");

        assert_eq!(
            runtime.vm().calls,
            vec![
                "setup".to_owned(),
                "update(0.016)".to_owned(),
                "update(0.032)".to_owned(),
            ]
        );
    }

    #[test]
    fn load_main_requires_update() {
        let vm = FakeVm {
            setup_present: true,
            update_present: false,
            ..FakeVm::default()
        };
        let mut runtime = ScriptRuntime::new(vm, RuntimeConfig::default());

        let error = runtime.load_main().expect_err("load_main should fail");
        assert_eq!(error, RuntimeError::MissingUpdateFunction);
    }

    #[test]
    fn update_failure_discards_queued_draw_batch() {
        let vm = FakeVm {
            setup_present: false,
            update_present: true,
            fail_on_update: true,
            ..FakeVm::default()
        };
        let mut runtime = ScriptRuntime::new(vm, RuntimeConfig::default());

        runtime.load_main().expect("load_main should succeed");
        let error = runtime
            .update(0.016)
            .expect_err("update should fail in fake vm");
        assert!(matches!(error, RuntimeError::FunctionCall { .. }));
        assert_eq!(runtime.vm().discard_calls, 1);
        assert_eq!(runtime.vm().flush_calls, 0);
    }

    #[test]
    fn draw_ops_are_queued_in_batch_order_until_frame_flush() {
        let script = r#"
import pycro

def update(dt):
    tex = pycro.load_texture("examples/assets/does-not-exist.png")
    pycro.clear_background((0.1, 0.2, 0.3, 1.0))
    pycro.draw_circle((10.0, 20.0), 5.0, (0.9, 0.8, 0.7, 1.0))
    pycro.draw_texture(tex, (30.0, 40.0), (64.0, 48.0))
    pycro.set_camera_target((70.0, 80.0))
    pycro.draw_text("queued", (90.0, 100.0), 18.0, (0.5, 0.6, 0.7, 1.0))
"#;
        let script_path = write_temp_script("draw-batch-order", script);
        let mut runtime = ScriptRuntime::new(
            RustPythonVm::new(),
            RuntimeConfig {
                entry_script: script_path.to_string_lossy().into_owned(),
            },
        );

        runtime.load_main().expect("load_main should succeed");
        runtime.update(0.016).expect("update should succeed");

        let before_flush = backend_dispatches(&runtime);
        assert_eq!(
            before_flush,
            vec![BackendDispatch::LoadTexture(
                "examples/assets/does-not-exist.png".to_owned()
            )]
        );

        let queued_draw_batch = runtime.vm().queued_draw_batch_snapshot();
        assert_eq!(
            queued_draw_batch,
            vec![
                QueuedDrawOp::ClearBackground(Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0
                }),
                QueuedDrawOp::DrawCircle {
                    position: Vec2 { x: 10.0, y: 20.0 },
                    radius: 5.0,
                    color: Color {
                        r: 0.9,
                        g: 0.8,
                        b: 0.7,
                        a: 1.0
                    },
                    render_mode: VectorRenderMode::Default,
                },
                QueuedDrawOp::DrawTexture {
                    texture: "examples/assets/does-not-exist.png".to_owned(),
                    position: Vec2 { x: 30.0, y: 40.0 },
                    size: Vec2 { x: 64.0, y: 48.0 }
                },
                QueuedDrawOp::SetCameraTarget(Vec2 { x: 70.0, y: 80.0 }),
                QueuedDrawOp::DrawText {
                    text: "queued".to_owned(),
                    position: Vec2 { x: 90.0, y: 100.0 },
                    font_size: 18.0,
                    color: Color {
                        r: 0.5,
                        g: 0.6,
                        b: 0.7,
                        a: 1.0
                    }
                },
            ]
        );
        assert_eq!(
            runtime.vm().take_queued_draw_batch_for_test(),
            queued_draw_batch,
            "flush should apply the queue in insertion order"
        );
        assert!(
            runtime.vm().queued_draw_batch_snapshot().is_empty(),
            "flush should clear the queue for the next frame"
        );

        fs::remove_file(script_path).expect("temporary script should be removable");
    }

    #[test]
    fn draw_batch_flush_clears_per_frame() {
        let script = r#"
import pycro

_count = 0

def update(dt):
    global _count
    _count += 1
    pycro.draw_circle((10.0 + _count, 20.0), float(_count), (1.0, 0.0, 0.0, 1.0))
"#;
        let script_path = write_temp_script("draw-batch-clear", script);
        let mut runtime = ScriptRuntime::new(
            RustPythonVm::new(),
            RuntimeConfig {
                entry_script: script_path.to_string_lossy().into_owned(),
            },
        );

        runtime.load_main().expect("load_main should succeed");
        runtime
            .update(0.016)
            .expect("frame 1 update should succeed");
        let after_first_flush = runtime.vm().take_queued_draw_batch_for_test();
        assert_eq!(
            after_first_flush,
            vec![QueuedDrawOp::DrawCircle {
                position: Vec2 { x: 11.0, y: 20.0 },
                radius: 1.0,
                color: Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0
                },
                render_mode: VectorRenderMode::Default,
            }]
        );

        runtime
            .update(0.032)
            .expect("frame 2 update should succeed");
        assert_eq!(
            runtime.vm().queued_draw_batch_snapshot(),
            vec![QueuedDrawOp::DrawCircle {
                position: Vec2 { x: 12.0, y: 20.0 },
                radius: 2.0,
                color: Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0
                },
                render_mode: VectorRenderMode::Default,
            }],
            "frame 1 draw must not replay in frame 2 queue"
        );

        assert_eq!(
            runtime.vm().take_queued_draw_batch_for_test(),
            vec![QueuedDrawOp::DrawCircle {
                position: Vec2 { x: 12.0, y: 20.0 },
                radius: 2.0,
                color: Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0
                },
                render_mode: VectorRenderMode::Default,
            }]
        );

        fs::remove_file(script_path).expect("temporary script should be removable");
    }

    #[test]
    fn submit_render_matches_legacy_draw_path_order_and_payload() {
        let direct_script = r#"
import pycro

def update(dt):
    tex = pycro.load_texture("examples/assets/does-not-exist.png")
    pycro.clear_background((0.1, 0.2, 0.3, 1.0))
    pycro.draw_circle((10.0, 20.0), 5.0, (0.9, 0.8, 0.7, 1.0))
    pycro.draw_texture(tex, (30.0, 40.0), (64.0, 48.0))
    pycro.set_camera_target((70.0, 80.0))
    pycro.draw_text("queued", (90.0, 100.0), 18.0, (0.5, 0.6, 0.7, 1.0))
"#;
        let submit_script = r#"
import pycro

def update(dt):
    tex = pycro.load_texture("examples/assets/does-not-exist.png")
    pycro.submit_render([
        ("clear_background", (0.1, 0.2, 0.3, 1.0)),
        ("draw_circle", (10.0, 20.0), 5.0, (0.9, 0.8, 0.7, 1.0)),
        ("draw_texture", tex, (30.0, 40.0), (64.0, 48.0)),
        ("set_camera_target", (70.0, 80.0)),
        ("draw_text", "queued", (90.0, 100.0), 18.0, (0.5, 0.6, 0.7, 1.0)),
    ])
"#;
        let direct_path = write_temp_script("draw-direct", direct_script);
        let submit_path = write_temp_script("draw-submit", submit_script);

        let mut direct_runtime = ScriptRuntime::new(
            RustPythonVm::new(),
            RuntimeConfig {
                entry_script: direct_path.to_string_lossy().into_owned(),
            },
        );
        let mut submit_runtime = ScriptRuntime::new(
            RustPythonVm::new(),
            RuntimeConfig {
                entry_script: submit_path.to_string_lossy().into_owned(),
            },
        );

        direct_runtime
            .load_main()
            .expect("direct runtime load_main should succeed");
        submit_runtime
            .load_main()
            .expect("submit runtime load_main should succeed");
        direct_runtime
            .update(0.016)
            .expect("direct runtime update should succeed");
        submit_runtime
            .update(0.016)
            .expect("submit runtime update should succeed");

        assert_eq!(
            submit_runtime.vm().queued_draw_batch_snapshot(),
            direct_runtime.vm().queued_draw_batch_snapshot(),
            "submit_render must queue the same draw payload/order as legacy draw_* calls"
        );

        assert_eq!(
            backend_dispatches(&submit_runtime),
            vec![BackendDispatch::LoadTexture(
                "examples/assets/does-not-exist.png".to_owned()
            )],
            "submit_render should preserve direct-return dispatch behavior for load_texture"
        );
        assert_eq!(
            backend_dispatches(&direct_runtime),
            backend_dispatches(&submit_runtime),
            "submit_render and legacy draw_* should keep identical direct backend dispatches before flush"
        );

        fs::remove_file(direct_path).expect("temporary script should be removable");
        fs::remove_file(submit_path).expect("temporary script should be removable");
    }

    #[test]
    fn submit_circle_batch_queues_expected_draw_circles() {
        let script = r#"
import pycro

def update(dt):
    pycro.submit_circle_batch(
        [(10.0, 20.0), (30.0, 40.0)],
        [5.0, 6.0],
        [(0.9, 0.8, 0.7, 1.0), (0.1, 0.2, 0.3, 1.0)],
    )
"#;
        let script_path = write_temp_script("submit-circle-batch", script);
        let mut runtime = ScriptRuntime::new(
            RustPythonVm::new(),
            RuntimeConfig {
                entry_script: script_path.to_string_lossy().into_owned(),
            },
        );

        runtime.load_main().expect("load_main should succeed");
        runtime.update(0.016).expect("update should succeed");

        assert_eq!(
            runtime.vm().queued_draw_batch_snapshot(),
            vec![
                QueuedDrawOp::DrawCircle {
                    position: Vec2 { x: 10.0, y: 20.0 },
                    radius: 5.0,
                    color: Color {
                        r: 0.9,
                        g: 0.8,
                        b: 0.7,
                        a: 1.0
                    },
                    render_mode: VectorRenderMode::Default,
                },
                QueuedDrawOp::DrawCircle {
                    position: Vec2 { x: 30.0, y: 40.0 },
                    radius: 6.0,
                    color: Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0
                    },
                    render_mode: VectorRenderMode::Default,
                }
            ],
            "submit_circle_batch must queue ordered draw_circle operations"
        );

        fs::remove_file(script_path).expect("temporary script should be removable");
    }

    #[test]
    fn direct_bridge_returns_backend_values_for_frame_time_and_texture_handle() {
        let script = r#"
import pycro

_last_dt = None

def update(dt):
    global _last_dt
    handle = pycro.load_texture('examples/assets/does-not-exist.png')
    if handle != 'examples/assets/does-not-exist.png':
        raise RuntimeError(f'unexpected texture handle: {handle}')

    current = pycro.frame_time()
    if abs(current - dt) > 1e-6:
        raise RuntimeError(f'frame_time mismatch: {current} vs {dt}')

    key_state = pycro.is_key_down('UnmappedKey')
    if key_state is not False:
        raise RuntimeError('is_key_down did not return bool')

    if _last_dt is not None and dt <= _last_dt:
        raise RuntimeError('dt did not advance')

    _last_dt = dt
"#;
        let script_path = write_temp_script("bridge-returns", script);
        let mut runtime = ScriptRuntime::new(
            RustPythonVm::new(),
            RuntimeConfig {
                entry_script: script_path.to_string_lossy().into_owned(),
            },
        );

        runtime.load_main().expect("load_main should succeed");
        runtime.update(0.05).expect("first update should succeed");
        assert_eq!(
            backend_dispatches(&runtime),
            vec![BackendDispatch::LoadTexture(
                "examples/assets/does-not-exist.png".to_owned()
            )],
            "load_texture should remain direct and not require draw-batch flush"
        );
        runtime.update(0.09).expect("second update should succeed");
        assert_eq!(
            backend_dispatches(&runtime),
            vec![
                BackendDispatch::LoadTexture("examples/assets/does-not-exist.png".to_owned()),
                BackendDispatch::LoadTexture("examples/assets/does-not-exist.png".to_owned()),
            ],
            "direct-return API semantics must remain unchanged frame-to-frame"
        );

        fs::remove_file(script_path).expect("temporary script should be removable");
    }

    #[test]
    fn direct_bridge_surfaces_python_exceptions_from_api_argument_errors() {
        let script = r#"
import pycro

def update(dt):
    pycro.draw_texture('tex', (1.0,), (32.0, 32.0))
"#;
        let script_path = write_temp_script("bridge-errors", script);
        let mut runtime = ScriptRuntime::new(
            RustPythonVm::new(),
            RuntimeConfig {
                entry_script: script_path.to_string_lossy().into_owned(),
            },
        );

        runtime.load_main().expect("load_main should succeed");
        let error = runtime
            .update(0.016)
            .expect_err("update should propagate python call failure");

        match error {
            RuntimeError::FunctionCall { function, details } => {
                assert_eq!(function, "update");
                assert!(
                    details.contains("ValueError"),
                    "details should preserve python exception type, got: {details}"
                );
                assert!(
                    details.contains("draw_texture position"),
                    "details should preserve helper context, got: {details}"
                );
            }
            _ => panic!("unexpected runtime error variant"),
        }

        fs::remove_file(script_path).expect("temporary script should be removable");
    }

    #[test]
    fn load_main_supports_importing_sidecar_python_modules_from_script_directory() {
        let root = write_temp_project(
            "imports",
            &[
                (
                    "main.py",
                    r#"
import phase03_player

hero = None

def setup():
    global hero
    hero = phase03_player.create_player("Rhea")

def update(dt):
    if hero is None:
        raise RuntimeError("hero should be initialized in setup")
    phase03_player.tick(hero, dt)
"#,
                ),
                (
                    "phase03_player.py",
                    r#"
class Player:
    def __init__(self, name):
        self.name = name
        self.x = 200.0
        self.y = 160.0

def create_player(name):
    return Player(name)

def tick(player, dt):
    player.x = player.x + (60.0 * dt)
"#,
                ),
            ],
        );
        let script_path = root.join("main.py");

        let mut runtime = ScriptRuntime::new(
            RustPythonVm::new(),
            RuntimeConfig {
                entry_script: script_path.to_string_lossy().into_owned(),
            },
        );

        runtime
            .load_main()
            .expect("main with sidecar import should load");
        runtime
            .update(0.016)
            .expect("update should succeed using imported module");

        fs::remove_dir_all(root).expect("temporary project should be removable");
    }

    #[test]
    fn load_main_supports_stdlib_math_and_os_imports() {
        let script = r#"
import math
import os

def update(dt):
    if abs(math.sqrt(81.0) - 9.0) > 1e-9:
        raise RuntimeError("math.sqrt failed")
    if abs(math.cos(0.0) - 1.0) > 1e-9:
        raise RuntimeError("math.cos failed")
    if abs(math.sin(0.0)) > 1e-9:
        raise RuntimeError("math.sin failed")
    if abs(math.hypot(3.0, 4.0) - 5.0) > 1e-9:
        raise RuntimeError("math.hypot failed")
    if math.pi <= 3.0:
        raise RuntimeError("math.pi is unavailable")
    cwd = os.getcwd()
    if not cwd:
        raise RuntimeError("os.getcwd returned empty path")
    if os.path.basename(cwd) is None:
        raise RuntimeError("os.path.basename unavailable")
    if os.getenv("__PYCRO_MISSING_ENV__", "fallback") != "fallback":
        raise RuntimeError("os.getenv default fallback failed")
"#;
        let script_path = write_temp_script("stdlib-imports", script);
        let mut runtime = ScriptRuntime::new(
            RustPythonVm::new(),
            RuntimeConfig {
                entry_script: script_path.to_string_lossy().into_owned(),
            },
        );

        runtime
            .load_main()
            .expect("main with stdlib imports should load");
        runtime
            .update(0.016)
            .expect("update should succeed using stdlib modules");

        fs::remove_file(script_path).expect("temporary script should be removable");
    }

    #[test]
    fn load_main_prefers_sidecar_module_over_stdlib_module_name_collision() {
        let root = write_temp_project(
            "imports-sidecar-overrides-stdlib",
            &[
                (
                    "main.py",
                    r#"
import math

def update(dt):
    if math.SOURCE != "sidecar":
        raise RuntimeError(f"expected sidecar module, got {math.SOURCE}")
"#,
                ),
                (
                    "math.py",
                    r#"
SOURCE = "sidecar"
"#,
                ),
            ],
        );
        let script_path = root.join("main.py");

        let mut runtime = ScriptRuntime::new(
            RustPythonVm::new(),
            RuntimeConfig {
                entry_script: script_path.to_string_lossy().into_owned(),
            },
        );

        runtime
            .load_main()
            .expect("main with sidecar math module should load");
        runtime
            .update(0.016)
            .expect("update should resolve sidecar module first");

        fs::remove_dir_all(root).expect("temporary project should be removable");
    }

    #[test]
    fn load_main_prefers_sidecar_module_for_transitive_import_collision() {
        let root = write_temp_project(
            "imports-sidecar-transitive-overrides-stdlib",
            &[
                (
                    "main.py",
                    r#"
import helper

def update(dt):
    helper.tick(dt)
"#,
                ),
                (
                    "helper.py",
                    r#"
import math

def tick(dt):
    if math.SOURCE != "sidecar-transitive":
        raise RuntimeError(f"expected sidecar module, got {math.SOURCE}")
"#,
                ),
                (
                    "math.py",
                    r#"
SOURCE = "sidecar-transitive"
"#,
                ),
            ],
        );
        let script_path = root.join("main.py");

        let mut runtime = ScriptRuntime::new(
            RustPythonVm::new(),
            RuntimeConfig {
                entry_script: script_path.to_string_lossy().into_owned(),
            },
        );

        runtime
            .load_main()
            .expect("main with transitive sidecar math module should load");
        runtime
            .update(0.016)
            .expect("update should resolve transitive sidecar module first");

        fs::remove_dir_all(root).expect("temporary project should be removable");
    }
}
