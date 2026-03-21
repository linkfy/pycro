//! CLI entrypoint for deterministic stub generation.

use std::{env, fs, path::Path};

use pycro::{module_spec, render_stub};

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let rendered = render_stub(module_spec());

    match args.as_slice() {
        [path] => write_stub(Path::new(path), &rendered),
        [flag, path] if flag == "--write" => write_stub(Path::new(path), &rendered),
        [flag, path] if flag == "--check" => check_stub(Path::new(path), &rendered),
        _ => Err("usage: cargo run --bin generate_stubs -- [--write|--check] <path>".to_owned()),
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
            "stub drift detected for {}. Regenerate with `cargo run --bin generate_stubs -- {}`",
            path.display(),
            path.display()
        ))
    }
}
