//! Single-project engine surface: CLI + runtime + API + backend contract.

/// Global allocator tuned for allocation-heavy runtime workloads.
#[cfg(all(not(target_env = "msvc"), not(target_arch = "wasm32")))]
#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod api;
pub mod backend;
pub mod embedded_project;
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
pub use embedded_project::{
    EmbeddedProjectFile, EmbeddedProjectPayload, embedded_project_payload,
    resolve_payload_relative_path,
};
pub use project::{
    PROJECT_MANIFEST_FILE_NAME, ProjectBuildTarget, ProjectBundle, ProjectContract,
    ResourceProviderKind, build_project_bundle,
};
pub use runtime::{
    ModuleInstallPlan, PythonVm, RuntimeConfig, RuntimeError, RuntimeValue, RustPythonVm,
    ScriptRuntime,
};
