#![allow(dead_code)]

#[path = "temporary.rs"]
mod temporary;

use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex, Weak},
};

struct Cache(PathBuf, Option<PathBuf>);

impl Drop for Cache {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
        if let Some(target) = &self.1 {
            if target.exists() {
                fs::remove_dir_all(target).unwrap();
            }
        }
    }
}

fn shared_cache() -> Arc<Cache> {
    static ACTIVE: Mutex<Weak<Cache>> = Mutex::new(Weak::new());
    let mut active = ACTIVE.lock().unwrap();
    if let Some(cache) = active.upgrade() {
        return cache;
    }
    let directory = temporary::temporary_directory("seseragi-native-test-cache");
    let target = std::env::var_os("CARGO_TARGET_DIR").map(|base| {
        PathBuf::from(base)
            .join("seseragi-native-tests")
            .join(directory.file_name().unwrap())
    });
    let cache = Arc::new(Cache(directory, target));
    *active = Arc::downgrade(&cache);
    cache
}

thread_local! {
    // Test threads share compilation work and release the cache when they exit.
    static CACHE: Arc<Cache> = shared_cache();
}

pub fn configure(command: &mut Command) {
    CACHE.with(|cache| {
        if !command
            .get_envs()
            .any(|(key, _)| key == "SESERAGI_NATIVE_CACHE")
        {
            command.env("SESERAGI_NATIVE_CACHE", &cache.0);
        }
        if !command
            .get_envs()
            .any(|(key, _)| key == "SESERAGI_NATIVE_TOOLCHAIN")
        {
            command.env("SESERAGI_NATIVE_TOOLCHAIN", toolchain());
        }
    });
    if !command.get_envs().any(|(key, _)| key == "CARGO_TARGET_DIR") {
        CACHE.with(|cache| {
            if let Some(target) = &cache.1 {
                command.env("CARGO_TARGET_DIR", target);
            }
        });
    }
}

pub fn command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut command = Command::new(program);
    configure(&mut command);
    command
}

// Cargo stores its generated project beside the input. Copy repository fixtures
// so those files stay in the test's temporary directory as well.
pub fn copy_input(source: &std::path::Path, scratch: &std::path::Path) -> PathBuf {
    if source.starts_with(scratch)
        || source
            .canonicalize()
            .unwrap()
            .starts_with(std::env::temp_dir().canonicalize().unwrap())
    {
        return source.to_owned();
    }
    let destination = scratch.join("source").join(source.file_name().unwrap());
    copy_tree(source, &destination);
    destination
}

fn copy_tree(source: &std::path::Path, destination: &std::path::Path) {
    if source.is_dir() {
        fs::create_dir_all(destination).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            copy_tree(&entry.path(), &destination.join(entry.file_name()));
        }
    } else {
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(source, destination).unwrap();
    }
}

pub fn toolchain() -> &'static std::path::Path {
    static SYSROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    SYSROOT.get_or_init(|| {
        let pinned = include_str!("../../rust-toolchain.toml")
            .lines()
            .find_map(|line| line.strip_prefix("channel = \"")?.strip_suffix('"'))
            .unwrap();
        let output = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
            .args(["--print", "sysroot"])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let root = PathBuf::from(String::from_utf8(output.stdout).unwrap().trim());
        let version = Command::new(root.join(format!("bin/rustc{}", std::env::consts::EXE_SUFFIX)))
            .arg("--version")
            .output()
            .unwrap();
        if version.status.success()
            && String::from_utf8_lossy(&version.stdout)
                .split_whitespace()
                .nth(1)
                == Some(pinned)
        {
            return root;
        }
        let output = Command::new("rustup")
            .args(["run", pinned, "rustc", "--print", "sysroot"])
            .output()
            .expect("tests need the pinned Rust toolchain installed with rustup");
        assert!(
            output.status.success(),
            "tests require Rust {pinned}: {output:?}"
        );
        PathBuf::from(String::from_utf8(output.stdout).unwrap().trim())
    })
}

#[cfg(unix)]
pub fn mock_toolchain(directory: &std::path::Path) -> PathBuf {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let bin = directory.join("bin");
    fs::create_dir(&bin).unwrap();
    let cargo = directory.join("cargo");
    let script = fs::read_to_string(&cargo).unwrap();
    let output = Command::new(toolchain().join("bin/cargo"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(output.status.success());
    let version = String::from_utf8(output.stdout).unwrap();
    fs::write(&cargo, format!("#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then printf '%s\\n' '{}'; exit 0; fi\n{script}", version.trim())).unwrap();
    fs::set_permissions(&cargo, fs::Permissions::from_mode(0o700)).unwrap();
    symlink(&cargo, bin.join("cargo")).unwrap();
    symlink(toolchain().join("bin/rustc"), bin.join("rustc")).unwrap();
    symlink(toolchain().join("lib"), directory.join("lib")).unwrap();
    directory.to_owned()
}
