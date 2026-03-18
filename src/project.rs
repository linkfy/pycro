//! Canonical external project contract and normalized build bundle for `pycro project`.

use std::fs;
use std::path::{Path, PathBuf};

const ENTRY_SCRIPT_FILE_NAME: &str = "main.py";

/// Reserved manifest file name for future project-level settings.
pub const PROJECT_MANIFEST_FILE_NAME: &str = "pycro-project.toml";

/// Canonical build targets accepted by `pycro project build`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectBuildTarget {
    /// Desktop host packaging flow (implemented in later phase).
    Desktop,
    /// Web packaging flow (implemented in later phase).
    Web,
    /// Android packaging flow (implemented in later phase).
    Android,
    /// iOS packaging flow (implemented in later phase).
    Ios,
}

impl ProjectBuildTarget {
    /// Parses a build target from CLI text.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "desktop" => Ok(Self::Desktop),
            "web" => Ok(Self::Web),
            "android" => Ok(Self::Android),
            "ios" => Ok(Self::Ios),
            _ => Err(format!(
                "invalid target `{value}`; expected one of: desktop, web, android, ios"
            )),
        }
    }

    /// Returns a stable CLI-facing target name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Web => "web",
            Self::Android => "android",
            Self::Ios => "ios",
        }
    }
}

/// Canonical external project contract validated before any target build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectContract {
    /// Project root directory provided by `--project`.
    pub root: PathBuf,
    /// Required entry script path (`main.py`).
    pub entry_script: PathBuf,
    /// Supported sidecar python files in project root (excluding `main.py`).
    pub local_python_modules: Vec<PathBuf>,
    /// Optional assets directory when present.
    pub assets_dir: Option<PathBuf>,
    /// Optional reserved project manifest path when present.
    pub manifest_file: Option<PathBuf>,
}

/// Resource provider strategy for runtime/platform loading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceProviderKind {
    /// Loads directly from local filesystem paths.
    FileSystem,
    /// Loads from a packaged archive representation.
    PackagedBundle,
    /// Loads from platform-native asset containers.
    PlatformAssetContainer,
}

/// Normalized bundle that downstream target phases consume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectBundle {
    /// Validated external project contract.
    pub contract: ProjectContract,
    /// Selected build target for downstream packaging.
    pub target: ProjectBuildTarget,
    /// Ordered provider options for runtime/resource loading.
    pub resource_provider_plan: Vec<ResourceProviderKind>,
}

/// Validates the external project contract and returns a normalized bundle.
pub fn build_project_bundle(
    project_root: &Path,
    target: ProjectBuildTarget,
) -> Result<ProjectBundle, String> {
    let contract = validate_project_contract(project_root)?;
    Ok(ProjectBundle {
        contract,
        target,
        resource_provider_plan: vec![
            ResourceProviderKind::FileSystem,
            ResourceProviderKind::PackagedBundle,
            ResourceProviderKind::PlatformAssetContainer,
        ],
    })
}

fn validate_project_contract(project_root: &Path) -> Result<ProjectContract, String> {
    if !project_root.exists() {
        return Err(format!(
            "project path does not exist: {}",
            project_root.display()
        ));
    }

    if !project_root.is_dir() {
        return Err(format!(
            "project path must be a directory: {}",
            project_root.display()
        ));
    }

    let entry_script = project_root.join(ENTRY_SCRIPT_FILE_NAME);
    if !entry_script.exists() || !entry_script.is_file() {
        return Err(format!(
            "project contract violation: required `{ENTRY_SCRIPT_FILE_NAME}` not found at {}",
            project_root.display()
        ));
    }

    let mut local_python_modules = Vec::new();
    let entries = fs::read_dir(project_root).map_err(|error| {
        format!(
            "failed to read project directory {}: {error}",
            project_root.display()
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "failed to inspect project directory {}: {error}",
                project_root.display()
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

        if is_python {
            local_python_modules.push(path);
        }
    }

    local_python_modules.sort();

    let assets_dir_candidate = project_root.join("assets");
    let assets_dir = if assets_dir_candidate.is_dir() {
        Some(assets_dir_candidate)
    } else {
        None
    };

    let manifest_candidate = project_root.join(PROJECT_MANIFEST_FILE_NAME);
    let manifest_file = if manifest_candidate.exists() {
        if !manifest_candidate.is_file() {
            return Err(format!(
                "project contract violation: `{PROJECT_MANIFEST_FILE_NAME}` must be a file"
            ));
        }
        Some(manifest_candidate)
    } else {
        None
    };

    Ok(ProjectContract {
        root: project_root.to_path_buf(),
        entry_script,
        local_python_modules,
        assets_dir,
        manifest_file,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        PROJECT_MANIFEST_FILE_NAME, ProjectBuildTarget, ResourceProviderKind, build_project_bundle,
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
        dir.push(format!("pycro-project-contract-{test_name}-{nanos}"));
        fs::create_dir_all(dir.as_path()).expect("failed to create temp test dir");
        dir
    }

    #[test]
    fn build_project_bundle_requires_main_py() {
        let root = temp_test_dir("requires-main");
        let result = build_project_bundle(root.as_path(), ProjectBuildTarget::Desktop);
        let error = result.expect_err("missing main.py should fail");
        assert!(error.contains("required `main.py`"));
        fs::remove_dir_all(root).expect("cleanup should succeed");
    }

    #[test]
    fn build_project_bundle_collects_contract_fields() {
        let root = temp_test_dir("collects-contract");
        fs::write(root.join("main.py"), "def update(dt):\n    pass\n")
            .expect("main.py should be writable");
        fs::write(root.join("enemy.py"), "HP = 10\n").expect("enemy.py should be writable");
        fs::create_dir_all(root.join("assets")).expect("assets dir should be creatable");
        fs::write(root.join(PROJECT_MANIFEST_FILE_NAME), "name = \"demo\"\n")
            .expect("manifest should be writable");

        let bundle = build_project_bundle(root.as_path(), ProjectBuildTarget::Web)
            .expect("contract should validate");

        assert_eq!(bundle.target, ProjectBuildTarget::Web);
        assert_eq!(
            bundle.resource_provider_plan,
            vec![
                ResourceProviderKind::FileSystem,
                ResourceProviderKind::PackagedBundle,
                ResourceProviderKind::PlatformAssetContainer
            ]
        );
        assert_eq!(bundle.contract.local_python_modules.len(), 1);
        assert!(bundle.contract.assets_dir.is_some());
        assert!(bundle.contract.manifest_file.is_some());

        fs::remove_dir_all(root).expect("cleanup should succeed");
    }

    #[test]
    fn parse_project_target_rejects_unknown_values() {
        let err = ProjectBuildTarget::parse("tv").expect_err("unknown target should fail");
        assert!(err.contains("expected one of"));
    }
}
