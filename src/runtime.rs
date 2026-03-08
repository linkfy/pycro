//! Script runtime lifecycle contract.

use crate::api::{
    ENTRYPOINT_SCRIPT, MODULE_NAME, SETUP_FUNCTION, UPDATE_FUNCTION, registration_plan,
};
use crate::backend::{Color, EngineBackend, MacroquadBackendContract, TextureHandle, Vec2};
use rustpython_vm::builtins::PyBaseExceptionRef;
use rustpython_vm::scope::Scope;
use rustpython_vm::{AsObject, Interpreter, PyObjectRef, VirtualMachine};
use std::fs;

const API_OPS_GLOBAL: &str = "__pycro_ops";
const FRAME_TIME_GLOBAL: &str = "__pycro_frame_time";

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
        self.vm
            .call_function(UPDATE_FUNCTION, &[RuntimeValue::Float(dt)])
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
    backend: MacroquadBackendContract,
}

impl std::fmt::Debug for RustPythonVm {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RustPythonVm")
            .field("initialized", &self.scope.is_some())
            .field("backend_dispatches", &self.backend.dispatch_log().len())
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
            backend: MacroquadBackendContract::default(),
        }
    }

    /// Exposes the current backend for smoke evidence.
    #[must_use]
    pub fn backend(&self) -> &MacroquadBackendContract {
        &self.backend
    }

    fn module_bootstrap_source(plan: &ModuleInstallPlan) -> String {
        let mut source = String::from(
            "Color = tuple\nVec2 = tuple\nTextureHandle = str\n\n__pycro_ops = []\n__pycro_frame_time = 0.016\n\n",
        );
        for function_name in &plan.exported_function_names {
            let function_source = match *function_name {
                "clear_background" => {
                    "def clear_background(color):\n    __pycro_ops.append(('clear_background', color))\n\n"
                }
                "draw_circle" => {
                    "def draw_circle(position, radius, color):\n    __pycro_ops.append(('draw_circle', position, radius, color))\n\n"
                }
                "is_key_down" => "def is_key_down(key):\n    return False\n\n",
                "frame_time" => "def frame_time():\n    return __pycro_frame_time\n\n",
                "load_texture" => {
                    "def load_texture(path):\n    __pycro_ops.append(('load_texture', path))\n    return path\n\n"
                }
                "draw_texture" => {
                    "def draw_texture(texture, position, size):\n    __pycro_ops.append(('draw_texture', texture, position, size))\n\n"
                }
                "set_camera_target" => {
                    "def set_camera_target(target):\n    __pycro_ops.append(('set_camera_target', target))\n\n"
                }
                _ => "",
            };
            source.push_str(function_source);
        }
        source
    }

    fn with_scope<T>(
        &self,
        scope: Scope,
        f: impl FnOnce(&VirtualMachine, Scope) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        self.interpreter.enter(|vm| f(vm, scope))
    }

    fn with_scope_and_backend<T>(
        &mut self,
        scope: Scope,
        f: impl FnOnce(&VirtualMachine, Scope, &mut MacroquadBackendContract) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        let interpreter = &self.interpreter;
        let backend = &mut self.backend;
        interpreter.enter(|vm| f(vm, scope, backend))
    }

    fn exception_details(vm: &VirtualMachine, error: &PyBaseExceptionRef) -> String {
        vm.print_exception(error.clone());
        format!("{error:?}")
    }

    fn flush_stdio(vm: &VirtualMachine) {
        if let Ok(stdout) = vm.sys_module.get_attr("stdout", vm) {
            let _ = vm.call_method(stdout.as_object(), "flush", ());
        }
        if let Ok(stderr) = vm.sys_module.get_attr("stderr", vm) {
            let _ = vm.call_method(stderr.as_object(), "flush", ());
        }
    }

    fn parse_vec2(
        vm: &VirtualMachine,
        object: PyObjectRef,
        context: &str,
    ) -> Result<Vec2, RuntimeError> {
        let values: Vec<f64> =
            object
                .try_into_value(vm)
                .map_err(|error| RuntimeError::FunctionCall {
                    function: context.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
        if values.len() != 2 {
            return Err(RuntimeError::FunctionCall {
                function: context.to_owned(),
                details: "expected Vec2 tuple length 2".to_owned(),
            });
        }
        Ok(Vec2 {
            x: values[0] as f32,
            y: values[1] as f32,
        })
    }

    fn parse_color(
        vm: &VirtualMachine,
        object: PyObjectRef,
        context: &str,
    ) -> Result<Color, RuntimeError> {
        let values: Vec<f64> =
            object
                .try_into_value(vm)
                .map_err(|error| RuntimeError::FunctionCall {
                    function: context.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
        if values.len() != 4 {
            return Err(RuntimeError::FunctionCall {
                function: context.to_owned(),
                details: "expected Color tuple length 4".to_owned(),
            });
        }
        Ok(Color {
            r: values[0] as f32,
            g: values[1] as f32,
            b: values[2] as f32,
            a: values[3] as f32,
        })
    }

    fn dispatch_api_operations(
        vm: &VirtualMachine,
        backend: &mut MacroquadBackendContract,
    ) -> Result<(), RuntimeError> {
        let modules =
            vm.sys_module
                .get_attr("modules", vm)
                .map_err(|error| RuntimeError::FunctionCall {
                    function: "sys.modules".to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
        let module =
            modules
                .get_item(MODULE_NAME, vm)
                .map_err(|error| RuntimeError::FunctionCall {
                    function: "sys.modules[pycro]".to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;
        let ops_obj =
            module
                .get_attr(API_OPS_GLOBAL, vm)
                .map_err(|error| RuntimeError::FunctionCall {
                    function: API_OPS_GLOBAL.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;

        let operations: Vec<PyObjectRef> =
            ops_obj
                .as_object()
                .try_to_value(vm)
                .map_err(|error| RuntimeError::FunctionCall {
                    function: API_OPS_GLOBAL.to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;

        module
            .set_attr(API_OPS_GLOBAL, vm.ctx.new_list(vec![]), vm)
            .map_err(|error| RuntimeError::FunctionCall {
                function: API_OPS_GLOBAL.to_owned(),
                details: Self::exception_details(vm, &error),
            })?;

        for operation in operations {
            let fields: Vec<PyObjectRef> =
                operation
                    .try_into_value(vm)
                    .map_err(|error| RuntimeError::FunctionCall {
                        function: "api operation tuple".to_owned(),
                        details: Self::exception_details(vm, &error),
                    })?;
            if fields.is_empty() {
                continue;
            }

            let op_name: String = fields[0].clone().try_into_value(vm).map_err(|error| {
                RuntimeError::FunctionCall {
                    function: "api operation name".to_owned(),
                    details: Self::exception_details(vm, &error),
                }
            })?;

            match op_name.as_str() {
                "clear_background" => {
                    if fields.len() != 2 {
                        return Err(RuntimeError::FunctionCall {
                            function: "clear_background".to_owned(),
                            details: "expected 1 argument".to_owned(),
                        });
                    }
                    let color = Self::parse_color(vm, fields[1].clone(), "clear_background")?;
                    backend.clear_background(color);
                }
                "draw_circle" => {
                    if fields.len() != 4 {
                        return Err(RuntimeError::FunctionCall {
                            function: "draw_circle".to_owned(),
                            details: "expected 3 arguments".to_owned(),
                        });
                    }
                    let position = Self::parse_vec2(vm, fields[1].clone(), "draw_circle position")?;
                    let radius: f64 = fields[2].clone().try_into_value(vm).map_err(|error| {
                        RuntimeError::FunctionCall {
                            function: "draw_circle radius".to_owned(),
                            details: Self::exception_details(vm, &error),
                        }
                    })?;
                    let color = Self::parse_color(vm, fields[3].clone(), "draw_circle color")?;
                    backend.draw_circle(position, radius as f32, color);
                }
                "load_texture" => {
                    if fields.len() != 2 {
                        return Err(RuntimeError::FunctionCall {
                            function: "load_texture".to_owned(),
                            details: "expected 1 argument".to_owned(),
                        });
                    }
                    let path: String = fields[1].clone().try_into_value(vm).map_err(|error| {
                        RuntimeError::FunctionCall {
                            function: "load_texture path".to_owned(),
                            details: Self::exception_details(vm, &error),
                        }
                    })?;
                    backend
                        .load_texture(&path)
                        .map_err(|error| RuntimeError::FunctionCall {
                            function: "load_texture".to_owned(),
                            details: error,
                        })?;
                }
                "draw_texture" => {
                    if fields.len() != 4 {
                        return Err(RuntimeError::FunctionCall {
                            function: "draw_texture".to_owned(),
                            details: "expected 3 arguments".to_owned(),
                        });
                    }
                    let texture: String =
                        fields[1].clone().try_into_value(vm).map_err(|error| {
                            RuntimeError::FunctionCall {
                                function: "draw_texture texture".to_owned(),
                                details: Self::exception_details(vm, &error),
                            }
                        })?;
                    let position =
                        Self::parse_vec2(vm, fields[2].clone(), "draw_texture position")?;
                    let size = Self::parse_vec2(vm, fields[3].clone(), "draw_texture size")?;
                    backend.draw_texture(&TextureHandle(texture), position, size);
                }
                "set_camera_target" => {
                    if fields.len() != 2 {
                        return Err(RuntimeError::FunctionCall {
                            function: "set_camera_target".to_owned(),
                            details: "expected 1 argument".to_owned(),
                        });
                    }
                    let target = Self::parse_vec2(vm, fields[1].clone(), "set_camera_target")?;
                    backend.set_camera_target(target);
                }
                _ => {
                    return Err(RuntimeError::FunctionCall {
                        function: op_name,
                        details: "unknown pycro API operation".to_owned(),
                    });
                }
            }
        }

        Ok(())
    }
}

impl PythonVm for RustPythonVm {
    fn install_module(&mut self, plan: ModuleInstallPlan) -> Result<(), RuntimeError> {
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

            let module_scope = Scope::with_builtins(None, attrs, vm);
            let module_source = Self::module_bootstrap_source(&plan);
            vm.run_code_string(module_scope, &module_source, "<pycro-module>".to_owned())
                .map_err(|error| RuntimeError::FunctionCall {
                    function: "pycro module bootstrap".to_owned(),
                    details: Self::exception_details(vm, &error),
                })?;

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
        self.with_scope_and_backend(scope, |vm, scope, backend| {
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
                backend.set_frame_time(*dt);
                let modules = vm.sys_module.get_attr("modules", vm).map_err(|error| {
                    RuntimeError::FunctionCall {
                        function: "sys.modules".to_owned(),
                        details: Self::exception_details(vm, &error),
                    }
                })?;
                let module = modules.get_item(MODULE_NAME, vm).map_err(|error| {
                    RuntimeError::FunctionCall {
                        function: "sys.modules[pycro]".to_owned(),
                        details: Self::exception_details(vm, &error),
                    }
                })?;
                module
                    .set_attr(FRAME_TIME_GLOBAL, vm.ctx.new_float(f64::from(*dt)), vm)
                    .map_err(|error| RuntimeError::FunctionCall {
                        function: FRAME_TIME_GLOBAL.to_owned(),
                        details: Self::exception_details(vm, &error),
                    })?;
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

            Self::dispatch_api_operations(vm, backend)?;
            Self::flush_stdio(vm);
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ModuleInstallPlan, PythonVm, RuntimeConfig, RuntimeError, RuntimeValue, ScriptRuntime,
    };

    #[derive(Debug, Default)]
    struct FakeVm {
        setup_present: bool,
        update_present: bool,
        calls: Vec<String>,
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
            let mut label = function.to_owned();
            if let [RuntimeValue::Float(dt)] = args {
                label = format!("{label}({dt:.3})");
            }
            self.calls.push(label);
            Ok(())
        }
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
}
