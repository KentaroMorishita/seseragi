use super::local_package::{canonical_output_path, stage_project_modules};
use super::{entry_source, stage_main_program};
use crate::{
    main_contract, project_main_contract, validate_target, ExecutionTarget, TargetMismatch,
};
use serde::{Deserialize, Serialize};
use seseragi_driver::{CompiledLocalProject, CompiledModule};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const BUILD_MARKER_NAME: &str = ".seseragi-build.json";
const BUILD_MARKER: &str = concat!(
    "{\n",
    "  \"schema\": 1,\n",
    "  \"kind\": \"single-file\",\n",
    "  \"entry\": \"entry.ts\",\n",
    "  \"module\": \"main.ts\",\n",
    "  \"metadata\": \"generated-module.json\",\n",
    "  \"runtime\": \"node_modules/@seseragi/runtime\"\n",
    "}\n",
);

static NEXT_BUILD: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub enum BuildError {
    InvalidEntry(String),
    TargetMismatch(TargetMismatch),
    Host(String),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidEntry(message) => write!(formatter, "invalid entry point: {message}"),
            Self::TargetMismatch(mismatch) => mismatch.fmt(formatter),
            Self::Host(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for BuildError {}

/// Writes one compiled source module and its executable process adapter to a
/// persistent directory.
///
/// Existing empty directories and directories marked as Seseragi single-file
/// builds are replaced. Other existing targets are left untouched.
pub fn build_main(compiled: &CompiledModule, output_directory: &Path) -> Result<(), BuildError> {
    let contract = main_contract(compiled).map_err(BuildError::InvalidEntry)?;
    validate_target(&contract, ExecutionTarget::Process).map_err(BuildError::TargetMismatch)?;
    publish_build(output_directory, |staging| {
        stage_main_program(compiled, &contract, &staging)?;
        write_json(
            &staging.join("generated-module.json"),
            "generated module metadata",
            &compiled.generated.metadata,
        )?;
        write_json(
            &staging.join("main.ts.map"),
            "source map",
            &compiled.generated.source_map,
        )?;
        fs::write(staging.join(BUILD_MARKER_NAME), BUILD_MARKER)
            .map_err(|error| format!("failed to write build ownership marker: {error}"))?;
        Ok(())
    })
}

/// Writes every generated module in a compiled local project, preserving the
/// compiler-planned output graph and process entry contract.
pub fn build_local_project(
    project: &CompiledLocalProject,
    output_directory: &Path,
) -> Result<(), BuildError> {
    let entry = project
        .compiled
        .modules
        .get(&project.entry_module)
        .ok_or_else(|| BuildError::InvalidEntry("compiled package omitted its entry".to_owned()))?;
    let contract = project_main_contract(&project.compiled, &project.entry_module)
        .map_err(BuildError::InvalidEntry)?;
    validate_target(&contract, ExecutionTarget::Process).map_err(BuildError::TargetMismatch)?;
    publish_build(output_directory, |staging| {
        stage_project_modules(&project.compiled, staging)?;
        let mut modules = Vec::with_capacity(project.compiled.order.len());
        for module_id in &project.compiled.order {
            let module = project
                .compiled
                .modules
                .get(module_id)
                .ok_or_else(|| format!("compiled package omitted {module_id}"))?;
            let typescript = canonical_output_path(&module.generated.metadata.outputs.typescript)?;
            let source_map =
                canonical_source_map_path(&module.generated.metadata.outputs.source_map)?;
            let metadata = module_metadata_path(&typescript)?;
            write_json(
                &staging.join(&source_map),
                "project module source map",
                &module.generated.source_map,
            )?;
            write_json(
                &staging.join(&metadata),
                "project generated module metadata",
                &module.generated.metadata,
            )?;
            modules.push(ProjectBuildModule {
                module: module_id.clone(),
                typescript: path_string(&typescript),
                source_map: path_string(&source_map),
                metadata: path_string(&metadata),
            });
        }
        crate::stage_typescript_package(staging)?;
        let entry_path = canonical_output_path(&entry.generated.metadata.outputs.typescript)?;
        fs::write(
            staging.join("entry.ts"),
            entry_source(&contract, &format!("./{}", path_string(&entry_path))),
        )
        .map_err(|error| format!("failed to stage runtime entry: {error}"))?;
        write_json(
            &staging.join(BUILD_MARKER_NAME),
            "build ownership marker",
            &ProjectBuildMarker {
                schema: 1,
                kind: "local-project",
                entry: "entry.ts",
                entry_module: &project.entry_module,
                modules,
                runtime: "node_modules/@seseragi/runtime",
            },
        )
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectBuildMarker<'entry> {
    schema: u32,
    kind: &'static str,
    entry: &'static str,
    entry_module: &'entry str,
    modules: Vec<ProjectBuildModule>,
    runtime: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectBuildModule {
    module: String,
    typescript: String,
    source_map: String,
    metadata: String,
}

#[derive(Deserialize)]
struct BuildOwnership {
    schema: u32,
    kind: String,
    entry: String,
    runtime: String,
}

fn publish_build(
    output_directory: &Path,
    stage: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<(), BuildError> {
    let staging = create_staging_directory(output_directory).map_err(BuildError::Host)?;
    let result =
        stage(&staging).and_then(|()| replace_output_directory(output_directory, &staging));
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result.map_err(BuildError::Host)
}

fn write_json(path: &Path, description: &str, value: &impl Serialize) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|error| format!("failed to encode {description}: {error}"))?;
    fs::write(path, format!("{json}\n"))
        .map_err(|error| format!("failed to write {description}: {error}"))
}

fn canonical_source_map_path(value: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || value.contains('\\')
        || value
            .split('/')
            .any(|segment| matches!(segment, "" | "." | ".."))
        || !value.ends_with(".ts.map")
    {
        return Err(format!(
            "generated package source map must be a canonical relative path: {value}"
        ));
    }
    Ok(path.to_owned())
}

fn module_metadata_path(typescript: &Path) -> Result<PathBuf, String> {
    let value = path_string(typescript);
    let stem = value
        .strip_suffix(".ts")
        .ok_or_else(|| format!("generated module is not TypeScript: {value}"))?;
    Ok(PathBuf::from(format!("{stem}.generated-module.json")))
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn create_staging_directory(output_directory: &Path) -> Result<PathBuf, String> {
    let name = output_directory
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| {
            format!(
                "build output must name a dedicated directory: {}",
                output_directory.display()
            )
        })?;
    let parent = output_directory
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create build output parent {}: {error}",
            parent.display()
        )
    })?;
    loop {
        let build = NEXT_BUILD.fetch_add(1, Ordering::Relaxed);
        let staging = parent.join(format!(
            ".{name}.seseragi-staging-{}-{build}",
            std::process::id()
        ));
        match fs::create_dir(&staging) {
            Ok(()) => return Ok(staging),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "failed to create build staging directory {}: {error}",
                    staging.display()
                ));
            }
        }
    }
}

fn replace_output_directory(output_directory: &Path, staging: &Path) -> Result<(), String> {
    match fs::symlink_metadata(output_directory) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(format!(
                "refusing to overwrite symlink build output {}",
                output_directory.display()
            ));
        }
        Ok(metadata) if !metadata.is_dir() => {
            return Err(format!(
                "refusing to overwrite non-directory build output {}",
                output_directory.display()
            ));
        }
        Ok(_) => {
            let is_empty = fs::read_dir(output_directory)
                .map_err(|error| {
                    format!(
                        "failed to inspect build output {}: {error}",
                        output_directory.display()
                    )
                })?
                .next()
                .is_none();
            let is_managed = is_managed_build(output_directory);
            if !is_empty && !is_managed {
                return Err(format!(
                    "refusing to clean unmanaged build output {}; choose an empty directory or a previous Seseragi build",
                    output_directory.display()
                ));
            }
            fs::remove_dir_all(output_directory).map_err(|error| {
                format!(
                    "failed to clean build output {}: {error}",
                    output_directory.display()
                )
            })?;
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "failed to inspect build output {}: {error}",
                output_directory.display()
            ));
        }
    }
    fs::rename(staging, output_directory).map_err(|error| {
        format!(
            "failed to publish build output {}: {error}",
            output_directory.display()
        )
    })
}

fn is_managed_build(output_directory: &Path) -> bool {
    fs::read_to_string(output_directory.join(BUILD_MARKER_NAME))
        .ok()
        .and_then(|marker| serde_json::from_str::<BuildOwnership>(&marker).ok())
        .is_some_and(|ownership| {
            ownership.schema == 1
                && ownership.entry == "entry.ts"
                && ownership.runtime == "node_modules/@seseragi/runtime"
                && matches!(ownership.kind.as_str(), "single-file" | "local-project")
        })
}

#[cfg(test)]
mod tests {
    use super::{is_managed_build, replace_output_directory, BUILD_MARKER, BUILD_MARKER_NAME};
    use std::fs;
    use std::path::Path;

    #[test]
    fn refuses_to_replace_an_unmanaged_directory() {
        let root = test_directory("unmanaged");
        let output = root.join("dist");
        let staging = root.join("staging");
        fs::create_dir_all(&output).unwrap();
        fs::write(output.join("keep.txt"), "keep").unwrap();
        fs::create_dir(&staging).unwrap();

        let error = replace_output_directory(&output, &staging).unwrap_err();

        assert!(error.contains("refusing to clean unmanaged build output"));
        assert_eq!(fs::read_to_string(output.join("keep.txt")).unwrap(), "keep");
        assert!(staging.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn replaces_a_managed_directory_and_removes_stale_files() {
        let root = test_directory("managed");
        let output = root.join("dist");
        let staging = root.join("staging");
        fs::create_dir_all(&output).unwrap();
        fs::write(output.join(BUILD_MARKER_NAME), BUILD_MARKER).unwrap();
        fs::write(output.join("stale.txt"), "stale").unwrap();
        fs::create_dir(&staging).unwrap();
        fs::write(staging.join("main.ts"), "export {};\n").unwrap();

        replace_output_directory(&output, &staging).unwrap();

        assert_eq!(
            fs::read_to_string(output.join("main.ts")).unwrap(),
            "export {};\n"
        );
        assert!(!output.join("stale.txt").exists());
        fs::remove_dir_all(root).unwrap();
    }

    fn test_directory(name: &str) -> std::path::PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "seseragi-build-runtime-{}-{name}",
            std::process::id()
        ));
        if directory.exists() {
            fs::remove_dir_all(&directory).unwrap();
        }
        fs::create_dir_all(&directory).unwrap();
        directory
    }

    #[test]
    fn marker_path_is_relative_to_the_build_root() {
        assert_eq!(
            Path::new(BUILD_MARKER_NAME),
            Path::new(".seseragi-build.json")
        );
    }

    #[test]
    fn recognizes_only_supported_build_ownership_markers() {
        let root = test_directory("ownership");
        fs::write(root.join(BUILD_MARKER_NAME), BUILD_MARKER).unwrap();
        assert!(is_managed_build(&root));

        fs::write(
            root.join(BUILD_MARKER_NAME),
            "{\"schema\":1,\"kind\":\"local-project\",\"entry\":\"entry.ts\",\"runtime\":\"node_modules/@seseragi/runtime\"}\n",
        )
        .unwrap();
        assert!(is_managed_build(&root));

        fs::write(
            root.join(BUILD_MARKER_NAME),
            "{\"schema\":2,\"kind\":\"local-project\"}\n",
        )
        .unwrap();
        assert!(!is_managed_build(&root));
        fs::remove_dir_all(root).unwrap();
    }
}
