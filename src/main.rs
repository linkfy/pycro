//! Single-command CLI that delegates script execution to runtime.

use macroquad::Window;
use pycro_cli::{
    DesktopFrameLoop, FrameLoopConfig, RuntimeConfig, RustPythonVm, ScriptRuntime, window_conf,
};
use pycro_cli::{module_spec, registration_plan, render_stub};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};

const MAIN_FILE_NAME: &str = "main.py";
const STUB_FILE_NAME: &str = "pycro.pyi";
const DEFAULT_STUB_OUTPUT_PATH: &str = "pycro.pyi";
const PYTHON_STUB_TEMPLATE: &str = include_str!("../python/pycro/__init__.pyi");

#[cfg(target_os = "windows")]
const LOCAL_RUNNER_FILE_NAME: &str = "pycro.exe";
#[cfg(not(target_os = "windows"))]
const LOCAL_RUNNER_FILE_NAME: &str = "pycro";

fn main() {
    let cli_args: Vec<String> = std::env::args().skip(1).collect();
    let command = match parse_cli_command(cli_args) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    match command {
        CliCommand::GenerateStubs(command) => {
            if let Err(error) = run_generate_stubs_contract(command) {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
        CliCommand::InitProject(project_name) => {
            if let Err(error) = write_project_scaffold(project_name.as_str()) {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
        CliCommand::RunScript(script_path) => {
            Window::from_config(window_conf(), async move {
                if let Err(error) = run_script_contract(script_path.as_str()).await {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            });
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CliCommand {
    GenerateStubs(GenerateStubsCommand),
    InitProject(String),
    RunScript(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GenerateStubsCommand {
    Write(PathBuf),
    Check(PathBuf),
}

fn parse_cli_command(args: Vec<String>) -> Result<CliCommand, String> {
    if args.is_empty() {
        return Ok(CliCommand::RunScript(MAIN_FILE_NAME.to_owned()));
    }

    match args[0].as_str() {
        "generate_stubs" => {
            parse_generate_stubs_command(args[1..].to_vec()).map(CliCommand::GenerateStubs)
        }
        "init" => parse_init_command(args[1..].to_vec()),
        _ => Ok(CliCommand::RunScript(args[0].clone())),
    }
}

fn parse_init_command(args: Vec<String>) -> Result<CliCommand, String> {
    if args.len() != 1 {
        return Err(
            "usage: pycro init <project_name>\nexample: pycro init my_game_project".to_owned(),
        );
    }

    Ok(CliCommand::InitProject(args[0].clone()))
}

fn parse_generate_stubs_command(args: Vec<String>) -> Result<GenerateStubsCommand, String> {
    let default_path = PathBuf::from(DEFAULT_STUB_OUTPUT_PATH);
    match args.as_slice() {
        [] => Ok(GenerateStubsCommand::Write(default_path)),
        [path] if path == "--write" => Ok(GenerateStubsCommand::Write(default_path)),
        [path] if path == "--check" => Ok(GenerateStubsCommand::Check(default_path)),
        [path] if path.starts_with("--") => Err(generate_stubs_usage()),
        [path] => Ok(GenerateStubsCommand::Write(PathBuf::from(path))),
        [flag, path] if flag == "--write" => Ok(GenerateStubsCommand::Write(PathBuf::from(path))),
        [flag, path] if flag == "--check" => Ok(GenerateStubsCommand::Check(PathBuf::from(path))),
        _ => Err(generate_stubs_usage()),
    }
}

fn generate_stubs_usage() -> String {
    "usage: pycro generate_stubs [--write|--check] [path]\nexample: pycro generate_stubs --check pycro.pyi".to_owned()
}

fn run_generate_stubs_contract(command: GenerateStubsCommand) -> Result<(), String> {
    let rendered = render_stub(module_spec());
    match command {
        GenerateStubsCommand::Write(path) => write_stub(path.as_path(), rendered.as_str()),
        GenerateStubsCommand::Check(path) => check_stub(path.as_path(), rendered.as_str()),
    }
}

fn write_stub(path: &Path, rendered: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }

    fs::write(path, rendered)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn check_stub(path: &Path, rendered: &str) -> Result<(), String> {
    let current = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;

    if current == rendered {
        Ok(())
    } else {
        Err(format!(
            "stub drift detected for {}. Regenerate with `pycro generate_stubs {}`",
            path.display(),
            path.display()
        ))
    }
}

fn write_project_scaffold(project_name: &str) -> Result<(), String> {
    let cwd = std::env::current_dir()
        .map_err(|error| format!("cannot resolve current directory: {error}"))?;
    create_project_scaffold(cwd.as_path(), project_name)?;
    Ok(())
}

fn create_project_scaffold(base_dir: &Path, project_name: &str) -> Result<PathBuf, String> {
    validate_project_name(project_name)?;

    let project_dir = base_dir.join(project_name);
    if project_dir.exists() {
        return Err(format!(
            "project directory already exists: {}",
            project_dir.display()
        ));
    }

    fs::create_dir_all(project_dir.as_path())
        .map_err(|error| format!("failed to create project directory: {error}"))?;

    let main_py = render_main_py_template(project_name);
    fs::write(project_dir.join(MAIN_FILE_NAME), main_py)
        .map_err(|error| format!("failed to write {MAIN_FILE_NAME}: {error}"))?;
    fs::write(project_dir.join(STUB_FILE_NAME), PYTHON_STUB_TEMPLATE)
        .map_err(|error| format!("failed to write {STUB_FILE_NAME}: {error}"))?;
    copy_current_executable_to_project(project_dir.as_path())?;

    println!("initialized pycro project at {}", project_dir.display());
    Ok(project_dir)
}

fn copy_current_executable_to_project(project_dir: &Path) -> Result<(), String> {
    let source = std::env::current_exe()
        .map_err(|error| format!("failed to resolve current pycro executable: {error}"))?;
    let destination = project_dir.join(LOCAL_RUNNER_FILE_NAME);
    fs::copy(source.as_path(), destination.as_path()).map_err(|error| {
        format!(
            "failed to copy local pycro executable to {}: {error}",
            destination.display()
        )
    })?;
    Ok(())
}

fn validate_project_name(project_name: &str) -> Result<(), String> {
    if project_name.trim().is_empty() {
        return Err("project name must not be empty".to_owned());
    }

    let mut components = Path::new(project_name).components();
    let first = components.next();
    let second = components.next();
    match (first, second) {
        (Some(Component::Normal(_)), None) => Ok(()),
        _ => Err("project name must be a single folder name (no path separators)".to_owned()),
    }
}

fn render_main_py_template(project_name: &str) -> String {
    format!(
        r#"import pycro

BG_COLOR = (0.07, 0.07, 0.09, 1.0)
text = "Welcome to {project_name}"

def update(dt: float) -> None:
    pycro.clear_background(BG_COLOR)
    pycro.draw_text(text, (24.0, 48.0), 32.0, (0.92, 0.94, 0.98, 1.0))
"#
    )
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
    let perf_enabled = std::env::var("PYCRO_PERF").is_ok_and(|value| value == "1");
    let mut perf = PerfWindow::default();

    let loop_owner = DesktopFrameLoop::new(FrameLoopConfig::from_env());
    let report = loop_owner
        .run(|dt| {
            let frame_start = Instant::now();
            let update_start = Instant::now();
            runtime
                .update(dt)
                .map_err(|error| format!("runtime update error: {error}"))?;
            let update_elapsed = update_start.elapsed();

            let flush_start = Instant::now();
            runtime
                .flush_draw_batch()
                .map_err(|error| format!("runtime draw flush error: {error}"))?;
            let flush_elapsed = flush_start.elapsed();

            if perf_enabled {
                let total_dispatches = runtime.vm().backend().dispatch_count();
                perf.record(
                    dt,
                    frame_start.elapsed(),
                    update_elapsed,
                    flush_elapsed,
                    total_dispatches,
                );
            }

            Ok(())
        })
        .await?;

    runtime
        .flush_io()
        .map_err(|error| format!("runtime io flush error: {error}"))?;

    println!("frames executed: {}", report.frames_executed);
    println!(
        "backend api dispatches: {}",
        runtime.vm().backend().dispatch_count()
    );
    Ok(())
}

#[derive(Debug)]
struct PerfWindow {
    dt_seconds_accum: f32,
    frame_wall_accum: Duration,
    update_accum: Duration,
    flush_accum: Duration,
    dispatches_accum: usize,
    frames: usize,
    last_dispatch_total: usize,
}

impl Default for PerfWindow {
    fn default() -> Self {
        Self {
            dt_seconds_accum: 0.0,
            frame_wall_accum: Duration::ZERO,
            update_accum: Duration::ZERO,
            flush_accum: Duration::ZERO,
            dispatches_accum: 0,
            frames: 0,
            last_dispatch_total: 0,
        }
    }
}

impl PerfWindow {
    fn record(
        &mut self,
        dt: f32,
        frame_elapsed: Duration,
        update_elapsed: Duration,
        flush_elapsed: Duration,
        dispatch_total: usize,
    ) {
        self.frames += 1;
        self.dt_seconds_accum += dt.max(0.0);
        self.frame_wall_accum += frame_elapsed;
        self.update_accum += update_elapsed;
        self.flush_accum += flush_elapsed;

        let dispatch_delta = dispatch_total.saturating_sub(self.last_dispatch_total);
        self.dispatches_accum += dispatch_delta;
        self.last_dispatch_total = dispatch_total;

        if self.dt_seconds_accum < 1.0 {
            return;
        }

        let frames = self.frames.max(1) as f64;
        let dt_fps = frames / f64::from(self.dt_seconds_accum.max(1e-6));
        let wall_seconds = self.frame_wall_accum.as_secs_f64().max(1e-9);
        let wall_fps = frames / wall_seconds;
        let avg_update_ms = (self.update_accum.as_secs_f64() * 1000.0) / frames;
        let avg_flush_ms = (self.flush_accum.as_secs_f64() * 1000.0) / frames;
        let avg_dispatches = (self.dispatches_accum as f64) / frames;

        println!(
            "[pycro-perf] frames={} dt_fps={:.2} wall_fps={:.2} avg_update_ms={:.3} avg_flush_ms={:.3} avg_dispatches_per_frame={:.1}",
            self.frames, dt_fps, wall_fps, avg_update_ms, avg_flush_ms, avg_dispatches
        );

        self.dt_seconds_accum = 0.0;
        self.frame_wall_accum = Duration::ZERO;
        self.update_accum = Duration::ZERO;
        self.flush_accum = Duration::ZERO;
        self.dispatches_accum = 0;
        self.frames = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CliCommand, DEFAULT_STUB_OUTPUT_PATH, GenerateStubsCommand, LOCAL_RUNNER_FILE_NAME,
        MAIN_FILE_NAME, STUB_FILE_NAME, check_stub, create_project_scaffold, parse_cli_command,
        render_main_py_template, run_generate_stubs_contract,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_test_dir(test_name: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        dir.push(format!("pycro-{test_name}-{nanos}"));
        fs::create_dir_all(dir.as_path()).expect("failed to create temp test dir");
        dir
    }

    #[test]
    fn parse_cli_defaults_to_script_mode() {
        assert_eq!(
            parse_cli_command(Vec::new()).expect("parse should succeed"),
            CliCommand::RunScript(MAIN_FILE_NAME.to_owned())
        );
    }

    #[test]
    fn parse_cli_accepts_script_argument() {
        assert_eq!(
            parse_cli_command(vec!["examples/phase01_basic_main.py".to_owned()])
                .expect("parse should succeed"),
            CliCommand::RunScript("examples/phase01_basic_main.py".to_owned())
        );
    }

    #[test]
    fn parse_cli_supports_init_mode() {
        assert_eq!(
            parse_cli_command(vec!["init".to_owned(), "my_game".to_owned()])
                .expect("parse should succeed"),
            CliCommand::InitProject("my_game".to_owned())
        );
    }

    #[test]
    fn parse_cli_rejects_invalid_init_usage() {
        let error = parse_cli_command(vec!["init".to_owned()]).expect_err("parse should fail");
        assert!(error.contains("usage: pycro init <project_name>"));
    }

    #[test]
    fn parse_cli_supports_generate_stubs_default_mode() {
        assert_eq!(
            parse_cli_command(vec!["generate_stubs".to_owned()]).expect("parse should succeed"),
            CliCommand::GenerateStubs(GenerateStubsCommand::Write(PathBuf::from(
                DEFAULT_STUB_OUTPUT_PATH
            )))
        );
    }

    #[test]
    fn parse_cli_supports_generate_stubs_check_mode() {
        assert_eq!(
            parse_cli_command(vec!["generate_stubs".to_owned(), "--check".to_owned()])
                .expect("parse should succeed"),
            CliCommand::GenerateStubs(GenerateStubsCommand::Check(PathBuf::from(
                DEFAULT_STUB_OUTPUT_PATH
            )))
        );
    }

    #[test]
    fn parse_cli_supports_generate_stubs_custom_path() {
        assert_eq!(
            parse_cli_command(vec![
                "generate_stubs".to_owned(),
                "python/custom_stub.pyi".to_owned()
            ])
            .expect("parse should succeed"),
            CliCommand::GenerateStubs(GenerateStubsCommand::Write(PathBuf::from(
                "python/custom_stub.pyi"
            )))
        );
    }

    #[test]
    fn parse_cli_rejects_invalid_generate_stubs_usage() {
        let error = parse_cli_command(vec![
            "generate_stubs".to_owned(),
            "--invalid".to_owned(),
            "foo".to_owned(),
        ])
        .expect_err("parse should fail");
        assert!(error.contains("usage: pycro generate_stubs"));
        assert!(error.contains("--check pycro.pyi"));
    }

    #[test]
    fn scaffold_writes_expected_files() {
        let base_dir = temp_test_dir("scaffold-writes-expected-files");
        let project_dir = create_project_scaffold(base_dir.as_path(), "starter")
            .expect("scaffold should be created");

        assert!(project_dir.join(MAIN_FILE_NAME).exists());
        assert!(project_dir.join(STUB_FILE_NAME).exists());
        assert!(project_dir.join(LOCAL_RUNNER_FILE_NAME).exists());

        let main_content = fs::read_to_string(project_dir.join(MAIN_FILE_NAME))
            .expect("main.py should be readable");
        assert!(main_content.contains("import pycro"));
        assert!(main_content.contains("BG_COLOR = (0.07, 0.07, 0.09, 1.0)"));
        assert!(main_content.contains("def update(dt: float) -> None:"));
        assert!(main_content.contains("pycro.clear_background(BG_COLOR)"));
        assert!(main_content.contains("pycro.draw_text(text"));

        fs::remove_dir_all(base_dir).expect("cleanup should succeed");
    }

    #[test]
    fn scaffold_rejects_existing_project_dir() {
        let base_dir = temp_test_dir("scaffold-rejects-existing-dir");
        let first = create_project_scaffold(base_dir.as_path(), "starter");
        assert!(first.is_ok());

        let second = create_project_scaffold(base_dir.as_path(), "starter");
        let error = second.expect_err("second creation should fail");
        assert!(error.contains("project directory already exists"));

        fs::remove_dir_all(base_dir).expect("cleanup should succeed");
    }

    #[test]
    fn main_template_contains_project_name_text() {
        let main_py = render_main_py_template("demo_app");
        assert!(main_py.contains("Welcome to demo_app"));
        assert!(main_py.contains("text = \"Welcome to demo_app\""));
    }

    #[test]
    fn scaffold_rejects_project_name_with_path_separator() {
        let base_dir = temp_test_dir("scaffold-rejects-separators");
        let result = create_project_scaffold(base_dir.as_path(), "bad/name");
        let error = result.expect_err("path separator should be rejected");
        assert!(error.contains("single folder name"));

        fs::remove_dir_all(base_dir).expect("cleanup should succeed");
    }

    #[test]
    fn scaffold_rejects_empty_project_name() {
        let base_dir = temp_test_dir("scaffold-rejects-empty");
        let result = create_project_scaffold(base_dir.as_path(), "   ");
        let error = result.expect_err("empty name should be rejected");
        assert!(error.contains("must not be empty"));

        fs::remove_dir_all(base_dir).expect("cleanup should succeed");
    }

    #[test]
    fn scaffold_writes_stub_with_export_list() {
        let base_dir = temp_test_dir("scaffold-writes-stub-content");
        let project_dir = create_project_scaffold(base_dir.as_path(), "starter")
            .expect("scaffold should be created");
        let stub_content =
            fs::read_to_string(project_dir.join(STUB_FILE_NAME)).expect("stub should be readable");
        assert!(stub_content.contains("__all__"));
        assert!(stub_content.contains("def draw_text"));

        fs::remove_dir_all(base_dir).expect("cleanup should succeed");
    }

    #[test]
    fn parse_cli_treats_unknown_command_as_script_path() {
        assert_eq!(
            parse_cli_command(vec!["custom_script.py".to_owned()]).expect("parse should succeed"),
            CliCommand::RunScript("custom_script.py".to_owned())
        );
    }

    #[test]
    fn generate_stubs_command_writes_file() {
        let base_dir = temp_test_dir("generate-stubs-command-writes-file");
        let output = base_dir.join("pycro.pyi");

        run_generate_stubs_contract(GenerateStubsCommand::Write(output.clone()))
            .expect("stub generation should succeed");
        let contents = fs::read_to_string(output).expect("stub file should exist");
        assert!(contents.contains("__all__"));

        fs::remove_dir_all(base_dir).expect("cleanup should succeed");
    }

    #[test]
    fn generate_stubs_check_succeeds_for_matching_file() {
        let base_dir = temp_test_dir("generate-stubs-check-success");
        let output = base_dir.join("pycro.pyi");

        run_generate_stubs_contract(GenerateStubsCommand::Write(output.clone()))
            .expect("stub generation should succeed");
        run_generate_stubs_contract(GenerateStubsCommand::Check(output))
            .expect("stub check should pass");

        fs::remove_dir_all(base_dir).expect("cleanup should succeed");
    }

    #[test]
    fn generate_stubs_check_fails_for_drifted_file() {
        let base_dir = temp_test_dir("generate-stubs-check-fail");
        let output = base_dir.join("pycro.pyi");

        run_generate_stubs_contract(GenerateStubsCommand::Write(output.clone()))
            .expect("stub generation should succeed");
        fs::write(output.as_path(), "drift").expect("drift write should succeed");

        let err = check_stub(output.as_path(), "expected").expect_err("drift should fail");
        assert!(err.contains("stub drift detected"));

        fs::remove_dir_all(base_dir).expect("cleanup should succeed");
    }
}
