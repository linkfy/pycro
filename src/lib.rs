//! Single-project engine surface: CLI + runtime + API + backend contract.

pub mod api;
pub mod backend;
pub mod runtime;

pub use api::{
    ApiFamily, ENTRYPOINT_SCRIPT, MODULE_NAME, ModuleSpec, PlatformMatrix, PlatformSupportLevel,
    PythonAlias, PythonArg, PythonFunction, RegistrationFunction, SETUP_FUNCTION, UPDATE_FUNCTION,
    module_spec, registration_plan, render_stub,
};
pub use backend::{
    BackendDispatch, Color, DesktopFrameLoop, DesktopLoopReport, EngineBackend, FrameLoopConfig,
    MacroquadBackendContract, TextureHandle, Vec2, window_conf,
};
pub use runtime::{
    ModuleInstallPlan, PythonVm, RuntimeConfig, RuntimeError, RuntimeValue, RustPythonVm,
    ScriptRuntime,
};
