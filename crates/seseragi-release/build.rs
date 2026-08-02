use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

fn git(root: &Path, arguments: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    Some(value.trim().to_owned())
}

fn main() {
    let manifest_directory = env::var("CARGO_MANIFEST_DIR").expect("manifest directory");
    let repository_root = Path::new(&manifest_directory).join("../..");
    println!(
        "cargo:rerun-if-changed={}",
        repository_root.join("Cargo.toml").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        repository_root.join(".git").display()
    );
    if let Some(git_directory) = git(&repository_root, &["rev-parse", "--git-dir"]) {
        let git_directory = PathBuf::from(git_directory);
        let git_directory = if git_directory.is_absolute() {
            git_directory
        } else {
            repository_root.join(git_directory)
        };
        for file in ["HEAD", "index"] {
            println!(
                "cargo:rerun-if-changed={}",
                git_directory.join(file).display()
            );
        }
    }

    let version = env::var("CARGO_PKG_VERSION").expect("package version");
    let commit = git(&repository_root, &["rev-parse", "--short=12", "HEAD"])
        .unwrap_or_else(|| "unknown".to_owned());
    let dirty = git(
        &repository_root,
        &["status", "--porcelain", "--untracked-files=normal"],
    )
    .map(|status| !status.is_empty())
    .unwrap_or(true);
    let tag = git(
        &repository_root,
        &["describe", "--exact-match", "--tags", "HEAD"],
    );
    let expected_tag = format!("v{version}");
    let channel = if !dirty && tag.as_deref() == Some(expected_tag.as_str()) {
        "release"
    } else {
        "development"
    };
    let release_tag = if channel == "release" {
        expected_tag
    } else {
        String::new()
    };
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_owned());

    println!("cargo:rustc-env=SESERAGI_COMMIT_SHA={commit}");
    println!("cargo:rustc-env=SESERAGI_BUILD_CHANNEL={channel}");
    println!("cargo:rustc-env=SESERAGI_BUILD_DIRTY={dirty}");
    println!("cargo:rustc-env=SESERAGI_BUILD_TARGET={target}");
    println!("cargo:rustc-env=SESERAGI_RELEASE_TAG={release_tag}");
}
