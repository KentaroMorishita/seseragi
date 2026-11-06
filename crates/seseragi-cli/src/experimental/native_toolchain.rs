use sha2::{Digest, Sha256};
use std::{
    ffi::OsString,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

const RUSTUP_VERSION: &str = "1.28.2";

pub(super) fn version() -> &'static str {
    include_str!("../../../../rust-toolchain.toml")
        .lines()
        .find_map(|line| line.strip_prefix("channel = \"")?.strip_suffix('"'))
        .expect("the workspace pins a Rust version")
}

pub(super) fn host() -> Result<&'static str, String> {
    let host = seseragi_release::BUILD_TARGET;
    match host {
        "aarch64-apple-darwin"
        | "x86_64-apple-darwin"
        | "x86_64-unknown-linux-gnu"
        | "x86_64-pc-windows-msvc" => Ok(host),
        _ => Err(format!("native Rust toolchain is unavailable for {host}")),
    }
}

pub(super) fn cache_root() -> Result<PathBuf, String> {
    let nonempty = |key| std::env::var_os(key).filter(|value| !value.is_empty());
    let root = if let Some(root) = nonempty("SESERAGI_NATIVE_CACHE") {
        PathBuf::from(root)
    } else if cfg!(windows) {
        PathBuf::from(nonempty("LOCALAPPDATA").ok_or(
            "native cache location is unavailable; set SESERAGI_NATIVE_CACHE to a writable directory",
        )?)
        .join("seseragi/cache/native")
    } else if cfg!(target_os = "macos") {
        PathBuf::from(nonempty("HOME").ok_or(
            "native cache location is unavailable; set SESERAGI_NATIVE_CACHE to a writable directory",
        )?)
        .join("Library/Caches/seseragi/native")
    } else if let Some(root) = nonempty("XDG_CACHE_HOME") {
        PathBuf::from(root).join("seseragi/native")
    } else {
        PathBuf::from(nonempty("HOME").ok_or(
            "native cache location is unavailable; set SESERAGI_NATIVE_CACHE to a writable directory",
        )?)
        .join(".cache/seseragi/native")
    };
    fs::create_dir_all(&root)
        .map_err(|error| format!("could not create native cache {}: {error}", root.display()))?;
    fs::canonicalize(root)
        .map(simplified)
        .map_err(|error| error.to_string())
}

/// Turns a Windows verbatim drive path (`\\?\C:\dir`) back into `C:\dir`.
/// rustc and MSVC `link.exe` mishandle verbatim paths in build-script output
/// names (`cannot open input file '.obj'`). Other paths are returned as is.
fn simplified(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy();
    match text.strip_prefix(r"\\?\") {
        Some(rest)
            if cfg!(windows)
                && rest.as_bytes().get(1) == Some(&b':')
                && rest.as_bytes()[0].is_ascii_alphabetic() =>
        {
            PathBuf::from(rest)
        }
        _ => path,
    }
}

pub(super) struct Toolchain {
    directory: PathBuf,
    cargo_home: PathBuf,
    pub host: String,
}

impl Toolchain {
    pub fn obtain() -> Result<Self, String> {
        let host = host()?;
        let cache = cache_root()?;
        if let Some(directory) = std::env::var_os("SESERAGI_NATIVE_TOOLCHAIN") {
            if directory.is_empty() {
                return Err(
                    "SESERAGI_NATIVE_TOOLCHAIN must name a complete Rust toolchain directory"
                        .into(),
                );
            }
            return Self::load(PathBuf::from(directory), cache.join("cargo"), host);
        }
        Self::prepare(&cache, host, install)
    }

    fn prepare(
        cache: &Path,
        host: &str,
        install: impl FnOnce(&Path, &str) -> Result<(), String>,
    ) -> Result<Self, String> {
        let toolchains = cache.join("toolchains");
        fs::create_dir_all(&toolchains).map_err(|error| error.to_string())?;
        let name = format!("{}-{host}", version());
        let lock = File::options()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(toolchains.join(format!("{name}.lock")))
            .map_err(|error| error.to_string())?;
        lock.lock().map_err(|error| error.to_string())?;
        let directory = toolchains.join(&name);
        let root = directory.join("rustup/toolchains").join(&name);
        let cargo_home = cache.join("cargo");
        if directory.try_exists().map_err(|error| error.to_string())? {
            return Self::load(root, cargo_home, host).map_err(|error| {
                format!(
                    "{error}; remove {} and retry while online",
                    directory.display()
                )
            });
        }
        eprintln!(
            "Preparing Rust {} for {host} in {} (internet access required on first use)",
            version(),
            directory.display()
        );
        let staged = tempfile::Builder::new()
            .prefix(".install-")
            .tempdir_in(&toolchains)
            .map_err(|error| error.to_string())?;
        install(staged.path(), host).map_err(|error| {
            format!("could not prepare native Rust {} for {host}: {error}; connect to the internet and retry, or set SESERAGI_NATIVE_TOOLCHAIN to a complete offline toolchain", version())
        })?;
        Self::load(
            staged.path().join("rustup/toolchains").join(&name),
            cargo_home.clone(),
            host,
        )?;
        fs::rename(staged.path(), &directory).map_err(|error| error.to_string())?;
        Self::load(root, cargo_home, host)
    }

    pub(super) fn load(
        directory: PathBuf,
        cargo_home: PathBuf,
        host: &str,
    ) -> Result<Self, String> {
        let directory = fs::canonicalize(&directory).map_err(|error| {
            format!(
                "native Rust {} for {host} is missing at {}: {error}",
                version(),
                directory.display()
            )
        })?;
        let toolchain = Self {
            directory,
            cargo_home,
            host: host.to_owned(),
        };
        for (tool, arguments) in [("rustc", &["-vV"][..]), ("cargo", &["--version"][..])] {
            let output = toolchain
                .command(tool)?
                .args(arguments)
                .output()
                .map_err(|error| {
                    format!(
                        "native {tool} {} for {host} is missing or cannot run at {}: {error}",
                        version(),
                        toolchain.binary(tool).display()
                    )
                })?;
            let text = String::from_utf8_lossy(&output.stdout);
            if !output.status.success()
                || text.split_whitespace().take(2).collect::<Vec<_>>() != [tool, version()]
            {
                return Err(format!(
                    "native {tool} version mismatch for {host}: expected {}, found {}{}",
                    version(),
                    text.trim(),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            if tool == "rustc" && !text.lines().any(|line| line == format!("host: {host}")) {
                return Err(format!(
                    "native rustc host mismatch: expected {host}, found {}",
                    text.trim()
                ));
            }
        }
        let libraries = toolchain
            .directory
            .join("lib/rustlib")
            .join(host)
            .join("lib");
        let has_std = fs::read_dir(&libraries).ok().is_some_and(|entries| {
            entries.filter_map(Result::ok).any(|entry| {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                name.starts_with("libstd-") && name.ends_with(".rlib") && entry.path().is_file()
            })
        });
        if !has_std {
            return Err(format!("native Rust {} standard library is missing for {host} at {}; provide rustc, cargo and rust-std from the same Rust release", version(), libraries.display()));
        }
        Ok(toolchain)
    }

    fn binary(&self, tool: &str) -> PathBuf {
        self.directory.join("bin").join(format!(
            "{tool}{}",
            if self.host.contains("windows") {
                ".exe"
            } else {
                ""
            }
        ))
    }

    fn command(&self, tool: &str) -> Result<Command, String> {
        let mut command = Command::new(self.binary(tool));
        let original_path = std::env::var_os("PATH").unwrap_or_default();
        let path = std::env::join_paths(
            std::iter::once(self.directory.join("bin"))
                .chain(std::env::split_paths(&original_path)),
        )
        .map_err(|error| format!("invalid native toolchain PATH: {error}"))?;
        command
            .env("PATH", path)
            .env("CARGO_HOME", &self.cargo_home)
            .env("RUSTC", self.binary("rustc"))
            .env("RUSTDOC", self.binary("rustdoc"))
            .env_remove("RUSTUP_TOOLCHAIN")
            .env("RUSTC_WRAPPER", "")
            .env("RUSTC_WORKSPACE_WRAPPER", "");
        Ok(command)
    }

    pub fn cargo(&self) -> Result<Command, String> {
        self.command("cargo")
    }

    pub fn rustc(&self) -> Result<Command, String> {
        self.command("rustc")
    }

    /// The `CARGO_HOME` that managed cargo runs with.
    pub fn cargo_home(&self) -> &Path {
        &self.cargo_home
    }
}

fn download(url: &str, destination: &Path) -> Result<(), String> {
    let output = Command::new(if cfg!(windows) { "curl.exe" } else { "curl" })
        .args([
            "--disable",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--tlsv1.2",
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--connect-timeout",
            "15",
            "--max-time",
            "300",
            "--output",
        ])
        .arg(destination)
        .arg(url)
        .output()
        .map_err(|error| {
            format!("could not start curl: {error}; install curl for the first toolchain download")
        })?;
    if !output.status.success() {
        return Err(format!(
            "download failed for {url}: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

fn install(directory: &Path, host: &str) -> Result<(), String> {
    install_with(directory, host, download)
}

fn install_with(
    directory: &Path,
    host: &str,
    mut download: impl FnMut(&str, &Path) -> Result<(), String>,
) -> Result<(), String> {
    let name = if host.contains("windows") {
        "rustup-init.exe"
    } else {
        "rustup-init"
    };
    let url = format!("https://static.rust-lang.org/rustup/archive/{RUSTUP_VERSION}/{host}/{name}");
    let installer = directory.join(name);
    let checksum = installer.with_extension("sha256");
    download(&format!("{url}.sha256"), &checksum)?;
    download(&url, &installer)?;
    let expected = fs::read_to_string(checksum).map_err(|error| error.to_string())?;
    let expected = expected.split_whitespace().next().unwrap_or("");
    let mut file = File::open(&installer).map_err(|error| error.to_string())?;
    let mut digest = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    if expected.len() != 64 || expected != format!("{:x}", digest.finalize()) {
        return Err("rustup-init SHA-256 checksum mismatch; the installer was not executed".into());
    }
    drop(file);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&installer, fs::Permissions::from_mode(0o700))
            .map_err(|error| error.to_string())?;
    }
    let output = Command::new(&installer)
        .args([
            "-y",
            "--no-modify-path",
            "--profile",
            "minimal",
            "--default-host",
            host,
            "--default-toolchain",
            version(),
        ])
        .env("RUSTUP_HOME", directory.join("rustup"))
        .env("CARGO_HOME", directory.join("cargo"))
        .env("RUSTUP_INIT_SKIP_PATH_CHECK", "yes")
        .env("RUSTUP_INIT_SKIP_EXISTENCE_CHECKS", "yes")
        .env("RUSTUP_INIT_SKIP_MSVC_CHECK", "yes")
        .env("RUSTUP_DIST_SERVER", "https://static.rust-lang.org")
        .env("RUSTUP_UPDATE_ROOT", "https://static.rust-lang.org/rustup")
        .env_remove("RUSTUP_TOOLCHAIN")
        .stdin(Stdio::null())
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "rustup-init failed: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    fs::remove_file(installer).map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) fn build_settings() -> Vec<(OsString, OsString)> {
    let mut settings = std::env::vars_os()
        .filter(|(key, _)| {
            let key = key.to_string_lossy();
            matches!(
                key.as_ref(),
                "RUSTFLAGS" | "CARGO_ENCODED_RUSTFLAGS" | "CARGO_INCREMENTAL"
            ) || key.starts_with("CARGO_PROFILE_")
                || key.starts_with("CARGO_TARGET_") && key != "CARGO_TARGET_DIR"
                || key.starts_with("CARGO_BUILD_")
        })
        .collect::<Vec<_>>();
    settings.sort();
    settings
}

pub(super) fn compilation_error(host: &str, diagnostics: &str) -> String {
    if [
        "can't find crate for `std`",
        "can't find crate for `core`",
        "invalid metadata files for crate `std`",
        "possibly newer version of crate `std`",
        "crate `std` compiled by an incompatible version of rustc",
        "crate `core` compiled by an incompatible version of rustc",
    ]
    .iter()
    .any(|needle| diagnostics.contains(needle))
    {
        return format!("Rust target compilation failed for {host}:\n{diagnostics}\nThe native standard library is missing or incompatible; provide rustc, cargo and rust-std {} for {host} from the same release, or remove the managed toolchain directory under SESERAGI_NATIVE_CACHE/toolchains and retry while online.", version());
    }
    let hint = if host.contains("apple") {
        "Install the Xcode Command Line Tools with `xcode-select --install`; select a working Xcode/CommandLineTools installation with `xcode-select --switch`, and check its SDK with `xcrun --show-sdk-path`."
    } else if host.contains("windows") {
        "Install Visual Studio Build Tools with the Desktop development with C++ workload and the Windows SDK; run from a Developer Command Prompt if the linker is unavailable."
    } else {
        "Install a C linker and development libraries (Debian/Ubuntu: `apt install build-essential`; Fedora: `dnf install gcc glibc-devel`)."
    };
    let needs_hint = [
        "linker",
        "linking with",
        "link.exe",
        "xcrun",
        "SDK",
        "sdk",
        "crt1.o",
        "crti.o",
        "-lSystem",
        "-lc",
        "kernel32.lib",
        "cannot find -l",
        "library not found",
    ]
    .iter()
    .any(|needle| diagnostics.contains(needle));
    format!(
        "Rust target compilation failed for {host}:\n{diagnostics}{}",
        if needs_hint {
            format!(
                "\n{hint}\nThe managed Rust toolchain does not include the system linker or SDK."
            )
        } else {
            String::new()
        }
    )
}

#[cfg(test)]
mod tests;
