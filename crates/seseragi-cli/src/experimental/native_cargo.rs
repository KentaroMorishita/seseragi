use seseragi_project::BuildProfile;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

const RUNTIME_MANIFEST: &str = r#"[package]
name = "seseragi-native-runtime"
version = "0.0.0"
edition = "2021"

[lib]
path = "lib.rs"
"#;

// Keep rustc -O's overflow and assertion settings in development builds.
const PROGRAM_MANIFEST: &str = r#"[package]
name = "seseragi-program"
version = "0.0.0"
edition = "2021"

[workspace]
resolver = "2"

[[bin]]
name = "program"
path = "program.rs"

[profile.dev]
opt-level = 0
debug-assertions = false
overflow-checks = false

[profile.release]
opt-level = 3
"#;

pub(crate) fn build(
    input: &Path,
    entry: &str,
    profile: BuildProfile,
    source: &str,
    executable: &Path,
) -> Result<(), String> {
    let project = Project::open(input, entry)?;
    project.build(
        profile,
        source,
        seseragi_backend_rust::native_runtime::SOURCE,
        executable,
    )
}

struct Project {
    directory: PathBuf,
    target: PathBuf,
    config: Config,
    _lock: File,
}

impl Project {
    fn open(input: &Path, entry: &str) -> Result<Self, String> {
        let input = fs::canonicalize(input).map_err(|error| error.to_string())?;
        let root = crate::local_project::containing_package(&input).unwrap_or_else(|| {
            if input.is_dir() {
                input.clone()
            } else {
                input.parent().unwrap().to_owned()
            }
        });
        let config = Config::read(&root)?;
        let identity = format!(
            "{:x}",
            Sha256::digest(format!("{}\0{entry}", input.display()))
        );
        let mut directory = root;
        for part in [".seseragi", "native", &identity] {
            directory.push(part);
            ensure_directory(&directory)?;
        }
        regular_file(&directory.join("build.lock"))?;
        let lock = File::options()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(directory.join("build.lock"))
            .map_err(|error| error.to_string())?;
        lock.lock().map_err(|error| error.to_string())?;
        let target = match std::env::var_os("CARGO_TARGET_DIR").filter(|value| !value.is_empty()) {
            Some(base) => {
                let base = std::path::absolute(base).map_err(|error| error.to_string())?;
                let key = format!(
                    "{:x}",
                    Sha256::digest(directory.to_string_lossy().as_bytes())
                );
                base.join("seseragi-native").join(key)
            }
            None => directory.join("target"),
        };
        Ok(Self {
            directory,
            target,
            config,
            _lock: lock,
        })
    }

    fn build(
        &self,
        profile: BuildProfile,
        source: &str,
        runtime: &str,
        executable: &Path,
    ) -> Result<(), String> {
        let runtime_directory = self.directory.join("runtime");
        ensure_directory(&runtime_directory)?;
        write_if_changed(
            &runtime_directory.join("Cargo.toml"),
            RUNTIME_MANIFEST.as_bytes(),
        )?;
        write_if_changed(
            &runtime_directory.join("lib.rs"),
            format!("#![allow(dead_code)]\n{runtime}").as_bytes(),
        )?;
        write_if_changed(&self.directory.join("program.rs"), source.as_bytes())?;
        write_if_changed(
            &self.directory.join("Cargo.toml"),
            self.config.manifest()?.as_bytes(),
        )?;
        write_if_changed(
            &self.directory.join("build.rs"),
            self.config.build_script().as_bytes(),
        )?;
        regular_file(&self.directory.join("Cargo.lock"))?;
        let output = Command::new("cargo")
            .current_dir(&self.directory)
            .args([
                "build",
                "--offline",
                "--bin",
                "program",
                "--message-format=json-render-diagnostics",
                "--profile",
            ])
            .arg(match profile {
                BuildProfile::Development => "dev",
                BuildProfile::Release => "release",
            })
            .arg("--target-dir")
            .arg(&self.target)
            .output()
            .map_err(|error| format!("could not start cargo: {error}; install a Rust toolchain"))?;
        if !output.status.success() {
            return Err(format!(
                "Rust target compilation failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let built = output
            .stdout
            .split(|byte| *byte == b'\n')
            .filter_map(|line| serde_json::from_slice::<serde_json::Value>(line).ok())
            .find(|message| {
                message["reason"] == "compiler-artifact"
                    && message["target"]["name"] == "program"
                    && message["executable"].is_string()
            })
            .and_then(|message| message["executable"].as_str().map(PathBuf::from))
            .ok_or("Cargo did not report a native executable")?;
        fs::copy(built, executable).map_err(|error| error.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;

#[derive(Default)]
struct Config {
    dependencies: toml::Table,
    libraries: Vec<String>,
    search_paths: Vec<String>,
}

impl Config {
    fn read(root: &Path) -> Result<Self, String> {
        let manifest = root.join("seseragi.toml");
        let source = match fs::read_to_string(&manifest) {
            Ok(source) => source,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error.to_string()),
        };
        let manifest = source
            .parse::<toml::Table>()
            .map_err(|error| error.to_string())?;
        let Some(native) = manifest.get("tool").and_then(|tool| tool.get("native")) else {
            return Ok(Self::default());
        };
        let native = native.as_table().ok_or("tool.native must be a table")?;
        for key in native.keys() {
            if !["dependencies", "libraries", "search-paths"].contains(&key.as_str()) {
                return Err(format!("unknown tool.native option `{key}`"));
            }
        }
        let mut dependencies = match native.get("dependencies") {
            Some(value) => value
                .as_table()
                .ok_or("tool.native.dependencies must be a table")?
                .clone(),
            None => toml::Table::new(),
        };
        for (name, dependency) in &mut dependencies {
            if name.replace('-', "_") == "seseragi_native_runtime" {
                return Err(
                    "tool.native.dependencies cannot replace seseragi-native-runtime".into(),
                );
            }
            if !dependency.is_str() && !dependency.is_table() {
                return Err(format!(
                    "tool.native.dependencies.{name} requires a version or Cargo dependency table"
                ));
            }
            if let Some(path) = dependency
                .as_table_mut()
                .and_then(|table| table.get_mut("path"))
            {
                let value = path
                    .as_str()
                    .ok_or("native dependency path must be a string")?;
                *path = root.join(value).to_string_lossy().into_owned().into();
            }
        }
        let strings = |key: &str| -> Result<Vec<String>, String> {
            let Some(value) = native.get(key) else {
                return Ok(Vec::new());
            };
            value.as_array().ok_or_else(|| format!("tool.native.{key} must be an array"))?
                .iter().map(|value| {
                    value.as_str().filter(|value| !value.is_empty() && !value.chars().any(char::is_control))
                        .map(str::to_owned).ok_or_else(|| format!("tool.native.{key} requires nonempty strings without control characters"))
                }).collect()
        };
        Ok(Self {
            dependencies,
            libraries: strings("libraries")?,
            search_paths: strings("search-paths")?
                .into_iter()
                .map(|path| root.join(path).to_string_lossy().into_owned())
                .collect(),
        })
    }

    fn manifest(&self) -> Result<String, String> {
        let mut dependencies = self.dependencies.clone();
        dependencies.insert(
            "seseragi-native-runtime".into(),
            toml::Value::Table(toml::Table::from_iter([("path".into(), "runtime".into())])),
        );
        let mut source = PROGRAM_MANIFEST.to_owned();
        source.push_str(
            &toml::to_string(&toml::Table::from_iter([(
                "dependencies".into(),
                toml::Value::Table(dependencies),
            )]))
            .map_err(|error| error.to_string())?,
        );
        Ok(source)
    }

    fn build_script(&self) -> String {
        let mut source =
            "fn main() {\n    println!(\"cargo:rerun-if-changed=build.rs\");\n".to_owned();
        for library in &self.libraries {
            source.push_str(&format!(
                "    println!(\"cargo:rustc-link-lib={{}}\", {library:?});\n"
            ));
        }
        for path in &self.search_paths {
            source.push_str(&format!(
                "    println!(\"cargo:rustc-link-search=native={{}}\", {path:?});\n"
            ));
        }
        source.push_str("}\n");
        source
    }
}

fn ensure_directory(path: &Path) -> Result<(), String> {
    match fs::create_dir(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
            if metadata.is_dir() && !metadata.file_type().is_symlink() {
                Ok(())
            } else {
                Err(format!(
                    "native build directory must not be a file or symlink: {}",
                    path.display()
                ))
            }
        }
        Err(error) => Err(error.to_string()),
    }
}

fn regular_file(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.is_file() || metadata.file_type().is_symlink() => Err(format!(
            "native build file must be a regular file: {}",
            path.display()
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn write_if_changed(path: &Path, source: &[u8]) -> Result<(), String> {
    regular_file(path)?;
    match fs::read(path) {
        Ok(previous) if previous == source => return Ok(()),
        Ok(_) => (),
        Err(error) if error.kind() == io::ErrorKind::NotFound => (),
        Err(error) => return Err(error.to_string()),
    }
    let mut staged = tempfile::NamedTempFile::new_in(path.parent().unwrap())
        .map_err(|error| error.to_string())?;
    staged
        .write_all(source)
        .map_err(|error| error.to_string())?;
    staged.persist(path).map_err(|error| error.to_string())?;
    Ok(())
}
