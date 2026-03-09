//! Script runtime lifecycle contract.

use crate::api::{
    ENTRYPOINT_SCRIPT, MODULE_NAME, SETUP_FUNCTION, UPDATE_FUNCTION, registration_plan,
};
use crate::backend::{Color, EngineBackend, MacroquadBackendContract, TextureHandle, Vec2};
use rustpython_vm::builtins::{PyBaseExceptionRef, PyDictRef};
use rustpython_vm::scope::Scope;
use rustpython_vm::{AsObject, Interpreter, PyObjectRef, PyResult, VirtualMachine};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

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
    backend: Arc<Mutex<MacroquadBackendContract>>,
    draw_batch: Arc<Mutex<Vec<QueuedDrawOp>>>,
}

#[derive(Clone, Debug, PartialEq)]
enum QueuedDrawOp {
    ClearBackground(Color),
    DrawCircle {
        position: Vec2,
        radius: f32,
        color: Color,
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

impl std::fmt::Debug for RustPythonVm {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let backend_dispatches = self
            .backend
            .lock()
            .map(|backend| backend.dispatch_log().len())
            .unwrap_or_default();
        let queued_draw_ops = self
            .draw_batch
            .lock()
            .map(|batch| batch.len())
            .unwrap_or_default();
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
    /// Creates a VM backed by a persistent RustPython interpreter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::without_stdlib(Default::default()),
            scope: None,
            backend: Arc::new(Mutex::new(MacroquadBackendContract::default())),
            draw_batch: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Exposes the current backend for smoke evidence.
    pub fn backend(&self) -> MutexGuard<'_, MacroquadBackendContract> {
        self.backend
            .lock()
            .expect("runtime backend mutex lock must not be poisoned")
    }

    #[cfg(test)]
    fn queued_draw_batch_snapshot(&self) -> Vec<QueuedDrawOp> {
        self.draw_batch
            .lock()
            .expect("runtime draw batch mutex lock must not be poisoned")
            .clone()
    }

    #[cfg(test)]
    fn take_queued_draw_batch_for_test(&self) -> Vec<QueuedDrawOp> {
        let mut draw_batch = self
            .draw_batch
            .lock()
            .expect("runtime draw batch mutex lock must not be poisoned");
        std::mem::take(&mut *draw_batch)
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

    fn parse_vec2_py(vm: &VirtualMachine, object: PyObjectRef, context: &str) -> PyResult<Vec2> {
        let values: Vec<f64> = object
            .try_into_value(vm)
            .map_err(|_| vm.new_value_error(format!("{context}: expected Vec2 tuple")))?;
        if values.len() != 2 {
            return Err(vm.new_value_error(format!("{context}: expected Vec2 tuple length 2")));
        }
        Ok(Vec2 {
            x: values[0] as f32,
            y: values[1] as f32,
        })
    }

    fn parse_color_py(vm: &VirtualMachine, object: PyObjectRef, context: &str) -> PyResult<Color> {
        let values: Vec<f64> = object
            .try_into_value(vm)
            .map_err(|_| vm.new_value_error(format!("{context}: expected Color tuple")))?;
        if values.len() != 4 {
            return Err(vm.new_value_error(format!("{context}: expected Color tuple length 4")));
        }
        Ok(Color {
            r: values[0] as f32,
            g: values[1] as f32,
            b: values[2] as f32,
            a: values[3] as f32,
        })
    }

    fn with_backend_py<T>(
        vm: &VirtualMachine,
        backend: &Arc<Mutex<MacroquadBackendContract>>,
        f: impl FnOnce(&mut MacroquadBackendContract) -> PyResult<T>,
    ) -> PyResult<T> {
        let mut backend = backend
            .lock()
            .map_err(|_| vm.new_runtime_error("backend mutex lock failed".to_owned()))?;
        f(&mut backend)
    }

    fn queue_draw_op_py(
        vm: &VirtualMachine,
        draw_batch: &Arc<Mutex<Vec<QueuedDrawOp>>>,
        op: QueuedDrawOp,
    ) -> PyResult<()> {
        let mut draw_batch = draw_batch
            .lock()
            .map_err(|_| vm.new_runtime_error("draw batch mutex lock failed".to_owned()))?;
        draw_batch.push(op);
        Ok(())
    }

    fn queue_draw_ops_py(
        vm: &VirtualMachine,
        draw_batch: &Arc<Mutex<Vec<QueuedDrawOp>>>,
        ops: Vec<QueuedDrawOp>,
    ) -> PyResult<()> {
        let mut draw_batch = draw_batch
            .lock()
            .map_err(|_| vm.new_runtime_error("draw batch mutex lock failed".to_owned()))?;
        draw_batch.extend(ops);
        Ok(())
    }

    fn parse_submit_render_command_py(
        vm: &VirtualMachine,
        command: PyObjectRef,
        index: usize,
    ) -> PyResult<QueuedDrawOp> {
        let fields: Vec<PyObjectRef> = command.try_into_value(vm).map_err(|_| {
            vm.new_value_error(format!(
                "submit_render commands[{index}]: expected command tuple/list"
            ))
        })?;
        if fields.is_empty() {
            return Err(vm.new_value_error(format!(
                "submit_render commands[{index}]: command must not be empty"
            )));
        }

        let command_name: String = fields[0].clone().try_into_value(vm).map_err(|_| {
            vm.new_value_error(format!(
                "submit_render commands[{index}][0]: expected command name string"
            ))
        })?;

        match command_name.as_str() {
            "clear_background" => {
                if fields.len() != 2 {
                    return Err(vm.new_value_error(format!(
                        "submit_render commands[{index}]: clear_background expects 1 argument"
                    )));
                }
                let color = Self::parse_color_py(
                    vm,
                    fields[1].clone(),
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
                let position = Self::parse_vec2_py(
                    vm,
                    fields[1].clone(),
                    &format!("submit_render commands[{index}] draw_circle position"),
                )?;
                let radius: f64 = fields[2].clone().try_into_value(vm).map_err(|_| {
                    vm.new_value_error(format!(
                        "submit_render commands[{index}] draw_circle radius: expected float"
                    ))
                })?;
                let color = Self::parse_color_py(
                    vm,
                    fields[3].clone(),
                    &format!("submit_render commands[{index}] draw_circle color"),
                )?;
                Ok(QueuedDrawOp::DrawCircle {
                    position,
                    radius: radius as f32,
                    color,
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
                let position = Self::parse_vec2_py(
                    vm,
                    fields[2].clone(),
                    &format!("submit_render commands[{index}] draw_texture position"),
                )?;
                let size = Self::parse_vec2_py(
                    vm,
                    fields[3].clone(),
                    &format!("submit_render commands[{index}] draw_texture size"),
                )?;
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
                let target = Self::parse_vec2_py(
                    vm,
                    fields[1].clone(),
                    &format!("submit_render commands[{index}] set_camera_target"),
                )?;
                Ok(QueuedDrawOp::SetCameraTarget(target))
            }
            "draw_text" => {
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
                let position = Self::parse_vec2_py(
                    vm,
                    fields[2].clone(),
                    &format!("submit_render commands[{index}] draw_text position"),
                )?;
                let font_size: f64 = fields[3].clone().try_into_value(vm).map_err(|_| {
                    vm.new_value_error(format!(
                        "submit_render commands[{index}] draw_text font_size: expected float"
                    ))
                })?;
                let color = Self::parse_color_py(
                    vm,
                    fields[4].clone(),
                    &format!("submit_render commands[{index}] draw_text color"),
                )?;
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
    }

    fn flush_draw_batch_ops(&mut self) -> Result<(), RuntimeError> {
        let pending_ops = {
            let mut draw_batch =
                self.draw_batch
                    .lock()
                    .map_err(|_| RuntimeError::FunctionCall {
                        function: "draw_batch_flush".to_owned(),
                        details: "draw batch mutex lock failed".to_owned(),
                    })?;
            std::mem::take(&mut *draw_batch)
        };

        if pending_ops.is_empty() {
            return Ok(());
        }

        let mut backend = self
            .backend
            .lock()
            .map_err(|_| RuntimeError::FunctionCall {
                function: "draw_batch_flush".to_owned(),
                details: "backend mutex lock failed".to_owned(),
            })?;

        for op in pending_ops {
            match op {
                QueuedDrawOp::ClearBackground(color) => backend.clear_background(color),
                QueuedDrawOp::DrawCircle {
                    position,
                    radius,
                    color,
                } => backend.draw_circle(position, radius, color),
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

        Ok(())
    }

    fn install_direct_api_functions(
        vm: &VirtualMachine,
        module_dict: &PyDictRef,
        plan: &ModuleInstallPlan,
        backend: Arc<Mutex<MacroquadBackendContract>>,
        draw_batch: Arc<Mutex<Vec<QueuedDrawOp>>>,
    ) -> Result<(), RuntimeError> {
        for function_name in &plan.exported_function_names {
            let function_obj = match *function_name {
                "clear_background" => {
                    let draw_batch = Arc::clone(&draw_batch);
                    vm.new_function(
                        "clear_background",
                        move |color: PyObjectRef, vm: &VirtualMachine| {
                            let color = Self::parse_color_py(vm, color, "clear_background")?;
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
                                Self::parse_vec2_py(vm, position, "draw_circle position")?;
                            let color = Self::parse_color_py(vm, color, "draw_circle color")?;
                            Self::queue_draw_op_py(
                                vm,
                                &draw_batch,
                                QueuedDrawOp::DrawCircle {
                                    position,
                                    radius: radius as f32,
                                    color,
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
                    let backend = Arc::clone(&backend);
                    vm.new_function("frame_time", move |vm: &VirtualMachine| {
                        Self::with_backend_py(vm, &backend, |backend| {
                            Ok(f64::from(backend.frame_time()))
                        })
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
                                Self::parse_vec2_py(vm, position, "draw_texture position")?;
                            let size = Self::parse_vec2_py(vm, size, "draw_texture size")?;
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
                            let target = Self::parse_vec2_py(vm, target, "set_camera_target")?;
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
                            let position = Self::parse_vec2_py(vm, position, "draw_text position")?;
                            let color = Self::parse_color_py(vm, color, "draw_text color")?;
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
                    vm.new_function(
                        "submit_render",
                        move |commands: Vec<PyObjectRef>, vm: &VirtualMachine| {
                            let mut ops = Vec::with_capacity(commands.len());
                            for (index, command) in commands.into_iter().enumerate() {
                                ops.push(Self::parse_submit_render_command_py(vm, command, index)?);
                            }
                            Self::queue_draw_ops_py(vm, &draw_batch, ops)
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
}

impl PythonVm for RustPythonVm {
    fn install_module(&mut self, plan: ModuleInstallPlan) -> Result<(), RuntimeError> {
        let backend = Arc::clone(&self.backend);
        let draw_batch = Arc::clone(&self.draw_batch);
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

            Self::install_direct_api_functions(vm, &attrs, &plan, backend, draw_batch)?;

            Ok(scope)
        })?;

        self.scope = Some(scope);
        Ok(())
    }

    fn load_script(&mut self, path: &str) -> Result<(), RuntimeError> {
        let source = fs::read_to_string(path).map_err(|error| RuntimeError::ScriptLoad {
            path: path.to_owned(),
            details: error.to_string(),
        })?;

        let scope = self.scope.clone().ok_or(RuntimeError::NotLoaded)?;
        self.with_scope(scope, |vm, scope| {
            Self::configure_import_path_for_script(vm, path)?;
            Self::install_stdlib_compat_modules(vm, path)?;
            Self::preload_sidecar_modules_for_script(vm, path, &source)?;
            scope
                .globals
                .set_item("__file__", vm.ctx.new_str(path).into(), vm)
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            vm.run_code_string(scope, &source, path.to_owned())
                .map(|_| ())
                .map_err(|error| RuntimeError::ScriptLoad {
                    path: path.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
            Self::flush_stdio(vm);
            Ok(())
        })
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
        let backend = Arc::clone(&self.backend);
        self.with_scope(scope, |vm, scope| {
            let callable = scope
                .globals
                .get_item_opt(function, vm)
                .map_err(|error| RuntimeError::FunctionCall {
                    function: function.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?
                .ok_or_else(|| RuntimeError::FunctionCall {
                    function: function.to_owned(),
                    details: "function not found in loaded script".to_owned(),
                })?;

            if let [RuntimeValue::Float(dt)] = args {
                let mut backend = backend.lock().map_err(|_| RuntimeError::FunctionCall {
                    function: function.to_owned(),
                    details: "backend mutex lock failed".to_owned(),
                })?;
                backend.set_frame_time(*dt);
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

            Self::flush_stdio(vm);
            Ok(())
        })
    }

    fn flush_draw_batch(&mut self) -> Result<(), RuntimeError> {
        self.flush_draw_batch_ops()
    }

    fn discard_draw_batch(&mut self) -> Result<(), RuntimeError> {
        let mut draw_batch = self
            .draw_batch
            .lock()
            .map_err(|_| RuntimeError::FunctionCall {
                function: "draw_batch_discard".to_owned(),
                details: "draw batch mutex lock failed".to_owned(),
            })?;
        draw_batch.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ModuleInstallPlan, PythonVm, QueuedDrawOp, RuntimeConfig, RuntimeError, RuntimeValue,
        RustPythonVm, ScriptRuntime,
    };
    use crate::backend::{BackendDispatch, Color, Vec2};
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
                    }
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
                }
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
                }
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
                }
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
import player

hero = None

def setup():
    global hero
    hero = player.create_player("Rhea")

def update(dt):
    if hero is None:
        raise RuntimeError("hero should be initialized in setup")
    player.tick(hero, dt)
"#,
                ),
                (
                    "player.py",
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
