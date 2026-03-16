//! Build-time generator for optional embedded project payload metadata.

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

const ENTRY_SCRIPT_FILE_NAME: &str = "main.py";
const PROJECT_MANIFEST_FILE_NAME: &str = "pycro-project.toml";
const EMBED_PROJECT_ENV: &str = "PYCRO_EMBED_PROJECT_ROOT";

#[derive(Debug, Clone)]
struct EmbeddedFile {
    relative_path: String,
    absolute_path: PathBuf,
}

fn main() {
    println!("cargo:rerun-if-env-changed={EMBED_PROJECT_ENV}");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be set by cargo"));
    let generated_path = out_dir.join("embedded_project_payload.rs");

    let embed_root = env::var_os(EMBED_PROJECT_ENV);
    let generated = match embed_root {
        Some(root) => generate_with_payload(Path::new(&root)),
        None => Ok(generate_without_payload()),
    }
    .unwrap_or_else(|error| panic!("failed to generate embedded payload module: {error}"));

    fs::write(&generated_path, generated).unwrap_or_else(|error| {
        panic!(
            "failed to write generated payload module {}: {error}",
            generated_path.display()
        )
    });
}

fn generate_without_payload() -> String {
    "/// Compile-time embedded project payload, when present.\npub const EMBEDDED_PROJECT_PAYLOAD: Option<EmbeddedProjectPayload> = None;\n".to_owned()
}

fn generate_with_payload(project_root: &Path) -> Result<String, String> {
    let root = fs::canonicalize(project_root).map_err(|error| {
        format!(
            "cannot canonicalize embedded project root {}: {error}",
            project_root.display()
        )
    })?;

    if !root.is_dir() {
        return Err(format!(
            "embedded project root must be a directory: {}",
            root.display()
        ));
    }

    let entry_script = root.join(ENTRY_SCRIPT_FILE_NAME);
    if !entry_script.is_file() {
        return Err(format!(
            "embedded project root must contain `{ENTRY_SCRIPT_FILE_NAME}`: {}",
            root.display()
        ));
    }

    println!("cargo:rerun-if-changed={}", entry_script.display());

    let mut files = vec![EmbeddedFile {
        relative_path: ENTRY_SCRIPT_FILE_NAME.to_owned(),
        absolute_path: entry_script,
    }];

    collect_root_python_sidecars(&root, &mut files)?;
    collect_optional_manifest(&root, &mut files);
    collect_optional_assets(&root, &mut files)?;
    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

    for file in &files {
        println!("cargo:rerun-if-changed={}", file.absolute_path.display());
    }

    let build_id = format!("{:016x}", payload_hash(&files)?);
    let mut generated = String::new();
    writeln!(
        &mut generated,
        "/// Compile-time embedded project payload, when present."
    )
    .expect("write to string should not fail");
    writeln!(
        &mut generated,
        "pub const EMBEDDED_PROJECT_PAYLOAD: Option<EmbeddedProjectPayload> = Some(EmbeddedProjectPayload {{"
    )
    .expect("write to string should not fail");
    writeln!(
        &mut generated,
        "    entry_script: {:?},",
        ENTRY_SCRIPT_FILE_NAME
    )
    .expect("write to string should not fail");
    writeln!(&mut generated, "    build_id: {:?},", build_id)
        .expect("write to string should not fail");
    writeln!(&mut generated, "    files: &[",).expect("write to string should not fail");

    for file in files {
        writeln!(
            &mut generated,
            "        EmbeddedProjectFile {{ relative_path: {:?}, bytes: include_bytes!({:?}) }},",
            file.relative_path, file.absolute_path
        )
        .expect("write to string should not fail");
    }

    writeln!(&mut generated, "    ],").expect("write to string should not fail");
    writeln!(&mut generated, "}});").expect("write to string should not fail");
    Ok(generated)
}

fn collect_root_python_sidecars(root: &Path, files: &mut Vec<EmbeddedFile>) -> Result<(), String> {
    let entries = fs::read_dir(root)
        .map_err(|error| format!("failed to read embedded root {}: {error}", root.display()))?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read entry in embedded root {}: {error}",
                root.display()
            )
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if path
            .file_name()
            .is_some_and(|name| name == ENTRY_SCRIPT_FILE_NAME)
        {
            continue;
        }

        let is_python = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension == "py");
        if !is_python {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("invalid UTF-8 file name: {}", path.display()))?;

        files.push(EmbeddedFile {
            relative_path: file_name.to_owned(),
            absolute_path: path,
        });
    }

    Ok(())
}

fn collect_optional_manifest(root: &Path, files: &mut Vec<EmbeddedFile>) {
    let manifest = root.join(PROJECT_MANIFEST_FILE_NAME);
    if manifest.is_file() {
        files.push(EmbeddedFile {
            relative_path: PROJECT_MANIFEST_FILE_NAME.to_owned(),
            absolute_path: manifest,
        });
    }
}

fn collect_optional_assets(root: &Path, files: &mut Vec<EmbeddedFile>) -> Result<(), String> {
    let assets_dir = root.join("assets");
    if !assets_dir.is_dir() {
        return Ok(());
    }
    collect_assets_recursive(root, assets_dir.as_path(), files)
}

fn collect_assets_recursive(
    root: &Path,
    current: &Path,
    files: &mut Vec<EmbeddedFile>,
) -> Result<(), String> {
    let entries = fs::read_dir(current).map_err(|error| {
        format!(
            "failed to read assets directory {}: {error}",
            current.display()
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read assets entry in {}: {error}",
                current.display()
            )
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_assets_recursive(root, path.as_path(), files)?;
            continue;
        }
        if !path.is_file() {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|error| {
                format!(
                    "failed to strip embedded root prefix {} from {}: {error}",
                    root.display(),
                    path.display()
                )
            })?
            .to_string_lossy()
            .replace('\\', "/");
        files.push(EmbeddedFile {
            relative_path: relative,
            absolute_path: path,
        });
    }
    Ok(())
}

fn payload_hash(files: &[EmbeddedFile]) -> Result<u64, String> {
    let mut hash = 0xcbf29ce484222325_u64;
    for file in files {
        hash = fnv1a_update(hash, file.relative_path.as_bytes());
        hash = fnv1a_update(hash, &[0]);
        let bytes = fs::read(file.absolute_path.as_path()).map_err(|error| {
            format!(
                "failed to read embedded file for hashing {}: {error}",
                file.absolute_path.display()
            )
        })?;
        hash = fnv1a_update(hash, bytes.as_slice());
        hash = fnv1a_update(hash, &[0xff]);
    }
    Ok(hash)
}

fn fnv1a_update(mut hash: u64, bytes: &[u8]) -> u64 {
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
