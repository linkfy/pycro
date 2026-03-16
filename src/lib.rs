//! Single-project engine surface: CLI + runtime + API + backend contract.

/// Global allocator tuned for allocation-heavy runtime workloads.
#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod api;
pub mod backend;
pub mod project;
pub mod runtime;

pub use api::{
    ApiFamily, ENTRYPOINT_SCRIPT, MODULE_NAME, ModuleSpec, PlatformMatrix, PlatformSupportLevel,
    PythonAlias, PythonArg, PythonFunction, RegistrationFunction, UPDATE_FUNCTION, module_spec,
    registration_plan, render_stub,
};
pub use backend::{
    BackendDispatch, Color, DesktopFrameLoop, DesktopLoopReport, EngineBackend, FrameLoopConfig,
    MacroquadBackendContract, TextureHandle, Vec2, window_conf,
};
pub use project::{
    PROJECT_MANIFEST_FILE_NAME, ProjectBuildTarget, ProjectBundle, ProjectContract,
    ResourceProviderKind, build_project_bundle,
};
pub use runtime::{
    ModuleInstallPlan, PythonVm, RuntimeConfig, RuntimeError, RuntimeValue, RustPythonVm,
    ScriptRuntime,
};
