//! Single-command CLI that delegates script execution to runtime.

use pycro_cli::{
    DesktopFrameLoop, FrameLoopConfig, RuntimeConfig, RustPythonVm, ScriptRuntime, window_conf,
};
use pycro_cli::{module_spec, registration_plan};

#[macroquad::main(window_conf)]
async fn main() {
    let script_path = script_path_from_args(std::env::args().nth(1));
    if let Err(error) = run_script_contract(script_path.as_str()).await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn script_path_from_args(script_arg: Option<String>) -> String {
    script_arg.unwrap_or_else(|| "examples/basic_main.py".to_owned())
}

async fn run_script_contract(script_path: &str) -> Result<(), String> {
    let config = RuntimeConfig {
        entry_script: script_path.to_owned(),
    };

    println!("run contract");
    println!("entry script: {}", config.entry_script);
    println!("python module: {}", module_spec().module_name);
    println!("registered api functions: {}", registration_plan().len());

    let mut runtime = ScriptRuntime::new(RustPythonVm::new(), config.clone());
    runtime
        .load_main()
        .map_err(|error| format!("runtime load error: {error}"))?;

    let loop_owner = DesktopFrameLoop::new(FrameLoopConfig::from_env());
    let report = loop_owner
        .run(|dt| {
            runtime
                .update(dt)
                .map_err(|error| format!("runtime update error: {error}"))
        })
        .await?;

    println!("frames executed: {}", report.frames_executed);
    println!(
        "backend api dispatches: {}",
        runtime.vm().backend().dispatch_log().len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::script_path_from_args;

    #[test]
    fn default_script_path_is_basic_example() {
        assert_eq!(
            script_path_from_args(None),
            "examples/basic_main.py".to_owned()
        );
    }

    #[test]
    fn accepts_positional_script_argument() {
        assert_eq!(
            script_path_from_args(Some("examples/basic_main.py".to_owned())),
            "examples/basic_main.py".to_owned()
        );
    }
}
