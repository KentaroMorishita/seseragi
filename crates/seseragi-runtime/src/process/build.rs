use super::stage_main_program;
use crate::main_contract;
use serde::Serialize;
use seseragi_driver::CompiledModule;
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
    Host(String),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidEntry(message) => write!(formatter, "invalid entry point: {message}"),
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
    let staging = create_staging_directory(output_directory).map_err(BuildError::Host)?;
    let result = (|| {
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
        replace_output_directory(output_directory, &staging)
    })();
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
            let is_managed = fs::read_to_string(output_directory.join(BUILD_MARKER_NAME))
                .map(|marker| marker == BUILD_MARKER)
                .unwrap_or(false);
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

#[cfg(test)]
mod tests {
    use super::{replace_output_directory, BUILD_MARKER, BUILD_MARKER_NAME};
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
}
