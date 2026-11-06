use super::native_toolchain::{self, Toolchain};
use seseragi_project::BuildProfile;
use sha2::{Digest, Sha256};
use std::{
    ffi::OsString,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

#[cfg(test)]
use std::process::Command;

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

const WINDOWS_MAIN_STACK: &str = r#"    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
            println!("cargo:rustc-link-arg-bins=/STACK:8388608");
        } else {
            println!("cargo:rustc-link-arg-bins=-Wl,--stack,8388608");
        }
    }
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
        &Toolchain::obtain()?,
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
        let target = target_root(&directory, std::env::var_os("CARGO_TARGET_DIR"))?;
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
        toolchain: &Toolchain,
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
        let target = self.target_for(profile, runtime)?;
        let mut command = toolchain.cargo()?;
        let encoded = std::env::var_os("CARGO_ENCODED_RUSTFLAGS");
        let plain = std::env::var_os("RUSTFLAGS");
        let linker = linker_settings(
            &self.directory,
            toolchain.cargo_home(),
            &toolchain.host,
            &|key| std::env::var_os(key),
        );
        let invocation = linker_invocation(&linker.rustflags, linker.linker.as_deref());
        let path_flags = path_flags(&self.directory, &target, &toolchain.host, invocation);
        let flags = toml::Value::Array(path_flags.iter().cloned().map(Into::into).collect());
        command
            .arg("--config")
            .arg(format!("build.rustflags={flags}"));
        if let Some(flags) = remapped_environment_flags(encoded, plain, &path_flags)? {
            command.env("CARGO_ENCODED_RUSTFLAGS", flags);
        }
        let output = command
            .current_dir(&self.directory)
            .args([
                "rustc",
                "--offline",
                "--bin",
                "program",
                "--message-format=json-render-diagnostics",
                "--target",
                &toolchain.host,
                "--profile",
            ])
            .arg(match profile {
                BuildProfile::Development => "dev",
                BuildProfile::Release => "release",
            })
            .arg("--target-dir")
            .arg(&target)
            .output()
            .map_err(|error| format!("could not start managed cargo: {error}"))?;
        if !output.status.success() {
            let mut diagnostics = String::from_utf8_lossy(&output.stderr).into_owned();
            for line in output.stdout.split(|byte| *byte == b'\n') {
                if let Ok(message) = serde_json::from_slice::<serde_json::Value>(line) {
                    if let Some(rendered) = message["message"]["rendered"].as_str() {
                        diagnostics.push_str(rendered);
                    }
                }
            }
            return Err(native_toolchain::compilation_error(
                &toolchain.host,
                &diagnostics,
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
        let staged = tempfile::NamedTempFile::new_in(
            executable
                .parent()
                .ok_or("native executable must have a parent directory")?,
        )
        .map_err(|error| error.to_string())?;
        fs::copy(built, staged.path()).map_err(|error| error.to_string())?;
        staged
            .persist(executable)
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn target_for(&self, profile: BuildProfile, runtime: &str) -> Result<PathBuf, String> {
        let identity = format!(
            "{:?}",
            (
                &self.directory,
                env!("CARGO_PKG_VERSION"),
                Sha256::digest(runtime.as_bytes()),
                native_toolchain::version(),
                native_toolchain::host()?,
                profile,
                self.config.manifest()?,
                self.config.build_script(),
                native_toolchain::build_settings(),
            )
        );
        Ok(self
            .target
            .join(format!("{:x}", Sha256::digest(identity.as_bytes()))))
    }
}

fn target_root(directory: &Path, requested: Option<OsString>) -> Result<PathBuf, String> {
    match requested.filter(|value| !value.is_empty()) {
        Some(base) => Ok(std::path::absolute(base)
            .map_err(|error| error.to_string())?
            .join("seseragi-native")),
        // MSVC build-script outputs must fit even when the source project is
        // deep in a checkout. Keep targets outside that tree on Windows.
        None if cfg!(windows) => Ok(native_toolchain::cache_root()?.join("build")),
        // Projects share one target beside, not inside, `.seseragi/native/`,
        // which holds only per-input project directories.
        None => Ok(directory
            .parent()
            .and_then(Path::parent)
            .ok_or("native project must be under .seseragi/native")?
            .join("native-target")),
    }
}

/// How rustc starts the linker on this build.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LinkerInvocation {
    /// Through a C compiler driver such as `cc` or `clang` (the Apple default).
    CompilerDriver,
    /// The linker itself, such as `ld`, `ld64.lld` or `rust-lld`.
    Direct,
}

/// Classifies the linker the same way rustc 1.97 does. An explicit
/// `-C linker-flavor` wins. Otherwise the last `-C linker` in rustflags, or
/// the linker Cargo configures, gives a hint from its file stem; a stem
/// without a hint keeps the Apple default, which uses a compiler driver.
fn linker_invocation(flags: &[String], configured_linker: Option<&str>) -> LinkerInvocation {
    let mut flavor = None;
    let mut linker = None;
    let mut arguments = flags.iter().map(String::as_str);
    while let Some(argument) = arguments.next() {
        let option = match argument {
            "-C" | "--codegen" => arguments.next(),
            _ => argument
                .strip_prefix("--codegen=")
                .or_else(|| argument.strip_prefix("-C")),
        };
        if let Some(value) = option.and_then(|option| option.strip_prefix("linker-flavor=")) {
            flavor = Some(value);
        } else if let Some(value) = option.and_then(|option| option.strip_prefix("linker=")) {
            linker = Some(value);
        }
    }
    if let Some(flavor) = flavor {
        // rustc's `-C linker-flavor` names: the `-cc` forms and the legacy
        // `gcc` and `em` use a compiler driver; MSVC names give no hint.
        let driver = flavor.ends_with("-cc") || ["gcc", "em", "msvc", "msvc-lld"].contains(&flavor);
        return if driver {
            LinkerInvocation::CompilerDriver
        } else {
            LinkerInvocation::Direct
        };
    }
    let Some(stem) = linker
        .or(configured_linker)
        .and_then(|linker| Path::new(linker).file_stem())
        .and_then(|stem| stem.to_str())
    else {
        return LinkerInvocation::CompilerDriver;
    };
    // rustc removes a version suffix such as `-21` before matching.
    let stem = stem
        .rsplit_once('-')
        .and_then(|(name, version)| version.chars().all(char::is_numeric).then_some(name))
        .unwrap_or(stem);
    let direct = [
        "wasm-ld",
        "ld.lld",
        "lld",
        "rust-lld",
        "lld-link",
        "ld",
        "link",
        "llvm-bitcode-linker",
    ]
    .contains(&stem)
        || stem.ends_with("-wasm-ld")
        || stem.ends_with("-ld");
    if direct {
        LinkerInvocation::Direct
    } else {
        LinkerInvocation::CompilerDriver
    }
}

/// The linker and rustflags Cargo uses for the host target.
#[derive(Debug, Default, Eq, PartialEq)]
struct LinkerSettings {
    /// `target.<host>.linker`, from config files or `CARGO_TARGET_<HOST>_LINKER`.
    linker: Option<String>,
    rustflags: Vec<String>,
}

/// Resolves the settings with Cargo's rules. Config files are read from the
/// build directory's `.cargo/` up to the file system root, then `CARGO_HOME`;
/// a nearer file wins, and in one directory `config` wins over `config.toml`.
/// Environment variables override config files. Rustflags come from
/// `CARGO_ENCODED_RUSTFLAGS`, `RUSTFLAGS`, `target.<host>.rustflags` or
/// `build.rustflags`, the first that is set. A file Cargo cannot parse is
/// skipped here; Cargo reports it when the build starts.
fn linker_settings(
    directory: &Path,
    cargo_home: &Path,
    host: &str,
    environment: &dyn Fn(&str) -> Option<OsString>,
) -> LinkerSettings {
    let mut directories = directory
        .ancestors()
        .map(|ancestor| ancestor.join(".cargo"))
        .collect::<Vec<_>>();
    if !directories.iter().any(|candidate| candidate == cargo_home) {
        directories.push(cargo_home.to_owned());
    }
    let key = host.to_uppercase().replace(['-', '.'], "_");
    let mut linker = None;
    let mut target_flags: Option<Vec<String>> = None;
    let mut build_flags: Option<Vec<String>> = None;
    let extend = |slot: &mut Option<Vec<String>>, flags: Vec<String>| {
        slot.get_or_insert_with(Vec::new).extend(flags);
    };
    // Lowest precedence first, so later values override or append.
    for directory in directories.iter().rev() {
        let file = [directory.join("config"), directory.join("config.toml")]
            .into_iter()
            .find(|file| file.is_file());
        let Some(config) = file
            .and_then(|file| fs::read_to_string(file).ok())
            .and_then(|source| source.parse::<toml::Table>().ok())
        else {
            continue;
        };
        let target = config.get("target").and_then(|targets| targets.get(host));
        if let Some(value) = target
            .and_then(|target| target.get("linker"))
            .and_then(toml::Value::as_str)
        {
            linker = Some(value.to_owned());
        }
        if let Some(flags) = target
            .and_then(|target| target.get("rustflags"))
            .and_then(config_flags)
        {
            extend(&mut target_flags, flags);
        }
        if let Some(flags) = config
            .get("build")
            .and_then(|build| build.get("rustflags"))
            .and_then(config_flags)
        {
            extend(&mut build_flags, flags);
        }
    }
    let text = |name: &str| environment(name).map(|value| value.to_string_lossy().into_owned());
    if let Some(value) = text(&format!("CARGO_TARGET_{key}_LINKER")) {
        linker = Some(value);
    }
    let words = |value: String| value.split_whitespace().map(Into::into).collect::<Vec<_>>();
    if let Some(value) = text(&format!("CARGO_TARGET_{key}_RUSTFLAGS")) {
        extend(&mut target_flags, words(value));
    }
    if let Some(value) = text("CARGO_BUILD_RUSTFLAGS") {
        extend(&mut build_flags, words(value));
    }
    let encoded = environment("CARGO_ENCODED_RUSTFLAGS");
    let plain = environment("RUSTFLAGS");
    let rustflags = if encoded.is_some() || plain.is_some() {
        environment_flags(encoded.as_deref(), plain.as_deref())
    } else {
        target_flags.or(build_flags).unwrap_or_default()
    };
    LinkerSettings { linker, rustflags }
}

/// A Cargo config rustflags value: a whitespace-separated string or an array.
fn config_flags(value: &toml::Value) -> Option<Vec<String>> {
    match value {
        toml::Value::String(flags) => Some(flags.split_whitespace().map(Into::into).collect()),
        toml::Value::Array(flags) => flags
            .iter()
            .map(|flag| flag.as_str().map(Into::into))
            .collect(),
        _ => None,
    }
}

/// Splits the rustflags Cargo will use, with Cargo's precedence.
fn environment_flags(
    encoded: Option<&std::ffi::OsStr>,
    plain: Option<&std::ffi::OsStr>,
) -> Vec<String> {
    match (encoded, plain) {
        (Some(flags), _) => flags
            .to_string_lossy()
            .split('\x1f')
            .filter(|flag| !flag.is_empty())
            .map(Into::into)
            .collect(),
        (None, Some(flags)) => flags
            .to_string_lossy()
            .split_whitespace()
            .map(Into::into)
            .collect(),
        (None, None) => Vec::new(),
    }
}

/// Compiler flags that keep the package's absolute location out of the
/// executable. `--remap-path-prefix` covers paths that rustc writes. On Apple
/// hosts the linker also records each object file's absolute path (the debug
/// map read by `dsymutil`); those objects live under the Cargo target
/// directory, which can be inside the package, so `-oso_prefix` makes them
/// relative to it. Both ld64 and ld64.lld accept `-oso_prefix`. A compiler
/// driver needs `-Xlinker` before each linker argument, which also keeps a
/// path containing commas as one argument; a direct linker takes them as is.
fn path_flags(
    directory: &Path,
    target: &Path,
    host: &str,
    invocation: LinkerInvocation,
) -> Vec<String> {
    let mut flags = vec![
        "--remap-path-prefix".to_owned(),
        format!("{}=.", directory.display()),
    ];
    if host.ends_with("-apple-darwin") {
        let mut prefix = target.as_os_str().to_owned();
        prefix.push("/");
        let prefix = prefix.to_string_lossy().into_owned();
        let arguments = match invocation {
            LinkerInvocation::CompilerDriver => {
                vec!["-Xlinker", "-oso_prefix", "-Xlinker", &prefix]
            }
            LinkerInvocation::Direct => vec!["-oso_prefix", &prefix],
        };
        flags.extend(
            arguments
                .into_iter()
                .map(|argument| format!("-Clink-arg={argument}")),
        );
    }
    flags
}

fn remapped_environment_flags(
    encoded: Option<OsString>,
    plain: Option<OsString>,
    path_flags: &[String],
) -> Result<Option<OsString>, String> {
    let mut flags = match (encoded, plain) {
        (Some(flags), _) => flags,
        (None, Some(flags)) => flags
            .to_str()
            .ok_or("RUSTFLAGS must contain valid Unicode")?
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("\x1f")
            .into(),
        (None, None) => return Ok(None),
    };
    if !flags.is_empty() {
        flags.push("\x1f");
    }
    flags.push(path_flags.join("\x1f"));
    Ok(Some(flags))
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
        // Windows reserves only 1 MiB for the main thread. Unoptimized
        // development builds need the 8 MiB Linux and macOS give by default
        // to reach managed limits such as the JSON codec nesting bound.
        source.push_str(WINDOWS_MAIN_STACK);
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
