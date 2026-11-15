//! `seseragi upgrade`: tells whether a newer release exists and installs it
//! with that release's own install script, so the update checks the archive
//! checksum exactly like a first install.

use std::path::{Path, PathBuf};
use std::process::Command;

const REPOSITORY: &str = "https://github.com/KentaroMorishita/seseragi";

pub(crate) fn upgrade(arguments: &[String]) -> Result<i32, String> {
    let check_only = match arguments {
        [] => false,
        [flag] if flag == "--check" => true,
        _ => return Err("usage: seseragi upgrade [--check]".to_owned()),
    };
    let current = seseragi_release::TOOLCHAIN_VERSION;
    // SESERAGI_VERSION picks a release the same way the install scripts do.
    let requested = std::env::var("SESERAGI_VERSION")
        .ok()
        .filter(|version| !version.is_empty());
    let target = match &requested {
        Some(version) => normalize(version)?,
        None => latest_version()?,
    };
    if target == current {
        println!("seseragi {current} is up to date");
        return Ok(0);
    }
    if requested.is_none() && !is_newer(&target, current) {
        println!("seseragi {current} is up to date (latest release: {target})");
        return Ok(0);
    }
    println!("seseragi {target} is available (installed: {current})");
    if check_only {
        println!("Run `seseragi upgrade` to install it.");
        return Ok(0);
    }
    install(&target)
}

/// Reads the newest release from the redirect of `releases/latest`, like
/// the install scripts, so no API token or rate-limited endpoint is needed.
fn latest_version() -> Result<String, String> {
    let output = Command::new("curl")
        .args(["-fsSLI", "-o", null_device(), "-w", "%{url_effective}"])
        .arg(format!("{REPOSITORY}/releases/latest"))
        .output()
        .map_err(|error| format!("could not run curl to find the latest release: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "could not find the latest release: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let url = String::from_utf8_lossy(&output.stdout);
    normalize(url.trim().rsplit('/').next().unwrap_or_default())
}

fn null_device() -> &'static str {
    if cfg!(windows) {
        "NUL"
    } else {
        "/dev/null"
    }
}

fn normalize(version: &str) -> Result<String, String> {
    let version = version.strip_prefix('v').unwrap_or(version);
    if parse(version).is_none() {
        return Err(format!("invalid release version `{version}`"));
    }
    Ok(version.to_owned())
}

fn parse(version: &str) -> Option<Vec<u64>> {
    version
        .split('.')
        .map(|part| part.parse().ok())
        .collect::<Option<Vec<_>>>()
        .filter(|parts| !parts.is_empty())
}

fn is_newer(candidate: &str, current: &str) -> bool {
    match (parse(candidate), parse(current)) {
        (Some(candidate), Some(current)) => candidate > current,
        _ => false,
    }
}

/// The install root, when this CLI runs from `<root>/bin` next to the LSP,
/// as the install scripts lay it out. `cargo install` puts only `seseragi`
/// in `~/.cargo/bin`, so that layout is left alone.
fn install_root(executable: &Path) -> Option<PathBuf> {
    let bin = executable.parent()?;
    let lsp = bin.join(format!("seseragi-lsp{}", std::env::consts::EXE_SUFFIX));
    (bin.file_name()? == "bin" && lsp.is_file()).then(|| bin.parent().map(Path::to_path_buf))?
}

fn install(version: &str) -> Result<i32, String> {
    let executable = std::env::current_exe()
        .and_then(std::fs::canonicalize)
        .map_err(|error| format!("could not locate this seseragi executable: {error}"))?;
    let script = if cfg!(windows) {
        "install.ps1"
    } else {
        "install.sh"
    };
    let command = if cfg!(windows) {
        format!("powershell -ExecutionPolicy Bypass -c \"irm {REPOSITORY}/releases/latest/download/install.ps1 | iex\"")
    } else {
        format!("curl -fsSL {REPOSITORY}/releases/latest/download/install.sh | sh")
    };
    let Some(root) = install_root(&executable) else {
        return Err(format!(
            "this seseragi ({}) was not installed by the install script, so upgrade cannot replace it; install the release with:\n  {command}",
            executable.display()
        ));
    };
    if cfg!(windows) {
        // Windows locks a running executable, so the script must run after
        // this process exits.
        println!("Close this terminal's seseragi processes, then run:\n  {command}");
        return Ok(0);
    }
    let directory = tempfile::Builder::new()
        .prefix("seseragi-upgrade-")
        .tempdir()
        .map_err(|error| format!("could not create a download directory: {error}"))?;
    let installer = directory.path().join(script);
    let downloaded = Command::new("curl")
        .args(["-fsSL", "-o"])
        .arg(&installer)
        .arg(format!(
            "{REPOSITORY}/releases/download/v{version}/{script}"
        ))
        .status()
        .map_err(|error| format!("could not run curl: {error}"))?;
    if !downloaded.success() {
        return Err(format!("could not download the {version} install script"));
    }
    let status = Command::new("sh")
        .arg(&installer)
        .env("SESERAGI_VERSION", version)
        .env("SESERAGI_INSTALL", &root)
        .status()
        .map_err(|error| format!("could not run the install script: {error}"))?;
    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions_numerically() {
        assert!(is_newer("0.61.172", "0.61.171"));
        assert!(is_newer("0.62.0", "0.61.999"));
        assert!(is_newer("0.61.10", "0.61.9"));
        assert!(!is_newer("0.61.171", "0.61.171"));
        assert!(!is_newer("0.61.170", "0.61.171"));
        assert!(!is_newer("latest", "0.61.171"));
    }

    #[test]
    fn accepts_tags_and_rejects_other_text() {
        assert_eq!(normalize("v0.61.172").unwrap(), "0.61.172");
        assert_eq!(normalize("0.61.172").unwrap(), "0.61.172");
        assert!(normalize("releases").is_err());
        assert!(normalize("").is_err());
    }

    #[test]
    fn finds_the_install_root_only_for_the_script_layout() {
        let root = tempfile::tempdir().unwrap();
        let bin = root.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        let cli = bin.join("seseragi");
        // Only the CLI, as `cargo install` leaves it.
        assert_eq!(install_root(&cli), None);
        std::fs::write(
            bin.join(format!("seseragi-lsp{}", std::env::consts::EXE_SUFFIX)),
            "",
        )
        .unwrap();
        assert_eq!(install_root(&cli), Some(root.path().to_path_buf()));
        assert_eq!(install_root(&root.path().join("seseragi")), None);
    }
}
