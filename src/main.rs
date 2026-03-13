//! Single-command CLI that delegates script execution to runtime.

use pycro_cli::{
    DesktopFrameLoop, FrameLoopConfig, RuntimeConfig, RustPythonVm, ScriptRuntime, window_conf,
};
use pycro_cli::{module_spec, registration_plan};
use std::time::{Duration, Instant};

#[macroquad::main(window_conf)]
async fn main() {
    let script_path = script_path_from_args(std::env::args().nth(1));
    if let Err(error) = run_script_contract(script_path.as_str()).await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn script_path_from_args(script_arg: Option<String>) -> String {
    script_arg.unwrap_or_else(|| "examples/phase01_basic_main.py".to_owned())
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
    use super::script_path_from_args;

    #[test]
    fn default_script_path_is_basic_example() {
        assert_eq!(
            script_path_from_args(None),
            "examples/phase01_basic_main.py".to_owned()
        );
    }

    #[test]
    fn accepts_positional_script_argument() {
        assert_eq!(
            script_path_from_args(Some("examples/phase01_basic_main.py".to_owned())),
            "examples/phase01_basic_main.py".to_owned()
        );
    }
}
