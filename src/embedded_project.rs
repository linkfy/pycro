//! Compile-time embedded project payload metadata.

use std::path::{Component, Path, PathBuf};

/// One embedded file entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmbeddedProjectFile {
    /// Relative path inside the embedded project payload.
    pub relative_path: &'static str,
    /// File bytes.
    pub bytes: &'static [u8],
}

/// Embedded project payload metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmbeddedProjectPayload {
    /// Relative entry script path (currently `main.py`).
    pub entry_script: &'static str,
    /// Stable build identifier derived from payload contents.
    pub build_id: &'static str,
    /// Embedded files included in the payload.
    pub files: &'static [EmbeddedProjectFile],
}

include!(concat!(env!("OUT_DIR"), "/embedded_project_payload.rs"));

/// Returns the embedded payload generated at compile time, when available.
#[must_use]
pub fn embedded_project_payload() -> Option<EmbeddedProjectPayload> {
    EMBEDDED_PROJECT_PAYLOAD
}

/// Resolves a payload-relative path into a safe filesystem path.
pub fn resolve_payload_relative_path(relative_path: &str) -> Result<PathBuf, String> {
    if relative_path.is_empty() {
        return Err("embedded payload file path must not be empty".to_owned());
    }

    let path = Path::new(relative_path);
    if path.is_absolute() {
        return Err(format!(
            "embedded payload file path must be relative: {relative_path}"
        ));
    }

    let mut cleaned = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(segment) => cleaned.push(segment),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(format!(
                    "embedded payload file path must not contain parent traversal: {relative_path}"
                ));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "embedded payload file path has unsupported component: {relative_path}"
                ));
            }
        }
    }

    if cleaned.as_os_str().is_empty() {
        return Err(format!(
            "embedded payload file path has no normal segments: {relative_path}"
        ));
    }

    Ok(cleaned)
}

#[cfg(test)]
mod tests {
    use super::{embedded_project_payload, resolve_payload_relative_path};
    use std::path::PathBuf;

    #[test]
    fn resolve_payload_relative_path_accepts_normal_relative_paths() {
        assert_eq!(
            resolve_payload_relative_path("main.py").expect("path should resolve"),
            PathBuf::from("main.py")
        );
        assert_eq!(
            resolve_payload_relative_path("assets/ui/hud.png").expect("path should resolve"),
            PathBuf::from("assets/ui/hud.png")
        );
    }

    #[test]
    fn resolve_payload_relative_path_rejects_parent_traversal() {
        let err = resolve_payload_relative_path("../main.py").expect_err("must reject traversal");
        assert!(err.contains("parent traversal"));
    }

    #[test]
    fn resolve_payload_relative_path_rejects_absolute_paths() {
        let err = resolve_payload_relative_path("/tmp/main.py").expect_err("must reject absolute");
        assert!(err.contains("must be relative"));
    }

    #[test]
    fn embedded_payload_entry_script_is_present_when_payload_exists() {
        if let Some(payload) = embedded_project_payload() {
            let has_entry = payload
                .files
                .iter()
                .any(|file| file.relative_path == payload.entry_script);
            assert!(
                has_entry,
                "embedded payload must include declared entry script `{}`",
                payload.entry_script
            );
        }
    }
}
