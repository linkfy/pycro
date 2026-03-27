//! Library entrypoints and public runtime modules for pycro.

use macroquad::Window;
use std::fs;
use std::path::PathBuf;

pub mod api;
pub mod backend;
pub mod embedded_project;
pub mod project;
pub mod runtime;

pub use api::{module_spec, registration_plan, render_stub};
pub use backend::{DesktopFrameLoop, FrameLoopConfig, draw_runtime_error_overlay, window_conf};
pub use embedded_project::{embedded_project_payload, resolve_payload_relative_path};
pub use project::{ProjectBuildTarget, build_project_bundle};
pub use runtime::{
    PythonSourceReloadMonitor, RuntimeConfig, RuntimeExecutionMode, RustPythonVm, ScriptRuntime,
};

#[derive(Debug)]
struct EmbeddedRuntimeLaunch {
    script_path: PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_runtime_entry_script_path(script_path: &str) -> Result<String, String> {
    let requested = PathBuf::from(script_path);
    if requested.is_absolute() || requested.is_file() {
        return Ok(script_path.to_owned());
    }

    let executable = std::env::current_exe()
        .map_err(|error| format!("failed to resolve current executable path: {error}"))?;
    let Some(executable_dir) = executable.parent() else {
        return Ok(script_path.to_owned());
    };
    let candidate = executable_dir.join(script_path);
    if candidate.is_file() {
        std::env::set_current_dir(executable_dir).map_err(|error| {
            format!(
                "failed to set current directory to executable location {}: {error}",
                executable_dir.display()
            )
        })?;
        return Ok(candidate.to_string_lossy().to_string());
    }

    Ok(script_path.to_owned())
}

#[cfg(target_arch = "wasm32")]
fn resolve_runtime_entry_script_path(script_path: &str) -> Result<String, String> {
    Ok(script_path.to_owned())
}

fn load_runtime_instance(
    config: &RuntimeConfig,
) -> Result<ScriptRuntime<RustPythonVm>, crate::runtime::RuntimeError> {
    let mut runtime = ScriptRuntime::new(RustPythonVm::new(), config.clone());
    runtime.load_main()?;
    Ok(runtime)
}

fn is_keyboard_interrupt_runtime_error(message: &str) -> bool {
    message.contains("KeyboardInterrupt")
}

async fn run_script_contract(script_path: &str, mode: RuntimeExecutionMode) -> Result<(), String> {
    let script_path = resolve_runtime_entry_script_path(script_path)?;
    let config = RuntimeConfig {
        entry_script: script_path,
    };
    let reload_monitor = PythonSourceReloadMonitor::new(mode, config.entry_script.as_str())
        .map_err(|error| format!("hot reload init error: {error}"))?;
    let mut runtime: Option<ScriptRuntime<RustPythonVm>> = None;
    let mut overlay_error: Option<String> = None;
    match load_runtime_instance(&config) {
        Ok(loaded) => runtime = Some(loaded),
        Err(error) => {
            let details = format!("runtime load error: {error}");
            eprintln!("{details}");
            overlay_error = Some(details);
        }
    }

    let loop_owner = DesktopFrameLoop::new(FrameLoopConfig::from_env());
    loop_owner
        .run(|dt| {
            let should_reload = reload_monitor.take_reload_signal();

            if should_reload {
                eprintln!("pycro hot reload: change detected, reloading...");
                match load_runtime_instance(&config) {
                    Ok(loaded) => {
                        eprintln!("pycro hot reload: reload complete");
                        runtime = Some(loaded);
                        overlay_error = None;
                    }
                    Err(error) => {
                        let details = format!("runtime reload error: {error}");
                        eprintln!("pycro hot reload: reload failed: {error}");
                        runtime = None;
                        overlay_error = Some(details);
                    }
                }
            }

            if overlay_error.is_none()
                && let Some(active_runtime) = runtime.as_mut()
            {
                if let Err(error) = active_runtime.update(dt) {
                    let details = format!("runtime update error: {error}");
                    if is_keyboard_interrupt_runtime_error(details.as_str()) {
                        return Err("runtime interrupted by keyboard interrupt (Ctrl+C)".to_owned());
                    }
                    eprintln!("{details}");
                    runtime = None;
                    overlay_error = Some(details);
                } else if let Err(error) = active_runtime.flush_draw_batch() {
                    let details = format!("runtime draw flush error: {error}");
                    eprintln!("{details}");
                    runtime = None;
                    overlay_error = Some(details);
                }
            }

            if let Some(message) = overlay_error.as_deref() {
                draw_runtime_error_overlay(message);
            }
            Ok(())
        })
        .await?;

    if let Some(active_runtime) = runtime.as_mut() {
        active_runtime
            .flush_io()
            .map_err(|error| format!("runtime io flush error: {error}"))?;
    }
    Ok(())
}

async fn run_embedded_project_contract() -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        return run_script_contract("main.py", RuntimeExecutionMode::EmbeddedPayload).await;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let launch = materialize_embedded_project_runtime()?;
        return run_script_contract(
            launch.script_path.to_string_lossy().as_ref(),
            RuntimeExecutionMode::EmbeddedPayload,
        )
        .await;
    }
}

fn materialize_embedded_project_runtime() -> Result<EmbeddedRuntimeLaunch, String> {
    let payload = embedded_project_payload()
        .ok_or_else(|| "embedded runtime requested but payload is not present".to_owned())?;
    let staging_root = make_embedded_staging_root(payload.build_id)?;

    for file in payload.files {
        let relative_path = resolve_payload_relative_path(file.relative_path)?;
        let output_path = staging_root.join(relative_path);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
        fs::write(output_path.as_path(), file.bytes)
            .map_err(|error| format!("failed to write {}: {error}", output_path.display()))?;
    }

    std::env::set_current_dir(staging_root.as_path()).map_err(|error| {
        format!("failed to set current directory to embedded payload root: {error}")
    })?;

    let entry_relative = resolve_payload_relative_path(payload.entry_script)?;
    let entry_script = staging_root.join(entry_relative);
    if !entry_script.is_file() {
        return Err(format!(
            "embedded payload entry script is missing after extraction: {}",
            entry_script.display()
        ));
    }

    Ok(EmbeddedRuntimeLaunch {
        script_path: entry_script,
    })
}

fn make_embedded_staging_root(build_id: &str) -> Result<PathBuf, String> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| format!("failed to compute embedded staging timestamp: {error}"))?
        .as_nanos();
    let staging_root = std::env::temp_dir()
        .join("pycro-embedded-runtime")
        .join(build_id)
        .join(format!("{stamp}-{}", std::process::id()));
    fs::create_dir_all(staging_root.as_path())
        .map_err(|error| format!("failed to create {}: {error}", staging_root.display()))?;
    Ok(staging_root)
}

/// Desktop/runtime entrypoint used by generated launchers.
pub fn main() {
    Window::from_config(window_conf(), async move {
        let result = if embedded_project_payload().is_some() {
            run_embedded_project_contract().await
        } else {
            run_script_contract("main.py", RuntimeExecutionMode::Source).await
        };
        if let Err(error) = result {
            eprintln!("{error}");
        }
    });
}

/// iOS C ABI entrypoint invoked by the generated Apple host app.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
#[inline(never)]
#[cfg(target_os = "ios")]
pub extern "C" fn start_app() {
    Window::from_config(window_conf(), async move {
        if let Err(error) = run_embedded_project_contract().await {
            eprintln!("{error}");
        }
    });
}
