use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const SOURCE: &str = "pub fn answer unit: Unit -> Int = 42\n";
const EXECUTABLE: &str = if cfg!(windows) {
    "program.exe"
} else {
    "program"
};

fn fixture() -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("main.ssrg");
    fs::write(&source, SOURCE).unwrap();
    (root, source)
}

fn command(operation: &str, source: &Path, destination: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_seseragi"));
    command
        .args(["experimental", operation])
        .arg(source)
        .args([
            "--entry",
            "answer",
            if operation == "emit-rust" {
                "--output"
            } else {
                "--out-dir"
            },
        ])
        .arg(destination);
    command
}

fn emit(source: &Path, destination: &Path) -> Output {
    command("emit-rust", source, destination)
        .env("PATH", "")
        .output()
        .unwrap()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_no_staging(directory: &Path) {
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        assert!(!entry
            .file_name()
            .to_string_lossy()
            .starts_with(".seseragi-rust-"));
        if entry.file_type().unwrap().is_dir() {
            assert_no_staging(&entry.path());
        }
    }
}

#[cfg(unix)]
#[test]
fn emit_rejects_output_and_metadata_symlinks_without_changing_siblings() {
    use std::os::unix::fs::symlink;
    for metadata in [false, true] {
        let (root, source) = fixture();
        let rust = root.path().join("program.rs");
        let report = rust.with_extension("rust-backend.json");
        fs::write(&rust, "old Rust output").unwrap();
        fs::write(&report, "old metadata").unwrap();
        let victim = root.path().join("victim");
        fs::write(&victim, "keep victim").unwrap();
        let link = if metadata { &report } else { &rust };
        fs::remove_file(link).unwrap();
        symlink(&victim, link).unwrap();
        let output = emit(&source, &rust);
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("symlink"));
        assert_eq!(fs::read_to_string(&victim).unwrap(), "keep victim");
        assert_eq!(
            fs::read_to_string(if metadata { &rust } else { &report }).unwrap(),
            if metadata {
                "old Rust output"
            } else {
                "old metadata"
            }
        );
        assert_eq!(fs::read_to_string(source).unwrap(), SOURCE);
        assert_no_staging(root.path());
    }
}

#[test]
fn metadata_directory_rejection_keeps_previous_rust_output() {
    let (root, source) = fixture();
    let rust = root.path().join("program.rs");
    fs::write(&rust, "old Rust output").unwrap();
    fs::create_dir(rust.with_extension("rust-backend.json")).unwrap();
    let output = emit(&source, &rust);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fs::read_to_string(rust).unwrap(), "old Rust output");
    assert_no_staging(root.path());
}

#[test]
fn direct_input_collision_is_rejected_before_any_sibling_output() {
    let (root, source) = fixture();
    let output = emit(&source, &source);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("output must differ from the source file")
    );
    assert_eq!(fs::read_to_string(&source).unwrap(), SOURCE);
    assert!(!source.with_extension("rust-backend.json").exists());
    assert_no_staging(root.path());
}

#[cfg(unix)]
#[test]
fn source_symlink_alias_is_rejected_and_dangling_output_is_not_created() {
    use std::os::unix::fs::symlink;
    for dangling in [false, true] {
        let (root, source) = fixture();
        let rust = root.path().join("program.rs");
        let target = if dangling {
            root.path().join("missing")
        } else {
            source.clone()
        };
        symlink(&target, &rust).unwrap();
        let output = emit(&source, &rust);
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(fs::read_to_string(&source).unwrap(), SOURCE);
        assert!(!rust.with_extension("rust-backend.json").exists());
        if dangling {
            assert!(!target.exists());
        }
        assert_no_staging(root.path());
    }
}

#[test]
fn output_hardlinks_do_not_modify_input_or_the_other_output() {
    for link_metadata in [false, true] {
        let (root, source) = fixture();
        let rust = root.path().join("program.rs");
        let report = rust.with_extension("rust-backend.json");
        fs::hard_link(&source, if link_metadata { &report } else { &rust }).unwrap();
        let output = emit(&source, &rust);
        assert_success(&output);
        assert_eq!(fs::read_to_string(&source).unwrap(), SOURCE);
        assert!(fs::read_to_string(&rust).unwrap().contains("fn main("));
        let metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(&report).unwrap()).unwrap();
        assert_eq!(metadata["entry"], "experimental/main::answer");
        assert_no_staging(root.path());
    }
    let (root, source) = fixture();
    let original = root.path().join("unrelated");
    fs::write(&original, "keep shared inode").unwrap();
    let rust = root.path().join("program.rs");
    let report = rust.with_extension("rust-backend.json");
    fs::hard_link(&original, &rust).unwrap();
    fs::hard_link(&original, &report).unwrap();
    assert_success(&emit(&source, &rust));
    assert_eq!(fs::read_to_string(original).unwrap(), "keep shared inode");
    assert!(fs::read_to_string(rust).unwrap().contains("fn main("));
    serde_json::from_slice::<serde_json::Value>(&fs::read(report).unwrap()).unwrap();
    assert_no_staging(root.path());
}

#[cfg(unix)]
#[test]
fn native_rejects_symlink_directory_even_with_a_trailing_separator() {
    use std::os::unix::fs::symlink;
    for suffix in ["", "/"] {
        let (root, source) = fixture();
        let victim = root.path().join("victim");
        fs::create_dir(&victim).unwrap();
        fs::write(victim.join("program.rs"), "keep victim").unwrap();
        let link = root.path().join("output");
        symlink(&victim, &link).unwrap();
        let destination = PathBuf::from(format!("{}{suffix}", link.display()));
        let output = command("build-native", &source, &destination)
            .env("PATH", "")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("symlink"));
        assert_eq!(
            fs::read_to_string(victim.join("program.rs")).unwrap(),
            "keep victim"
        );
        assert_eq!(fs::read_dir(victim).unwrap().count(), 1);
        assert_no_staging(root.path());
    }
}

#[cfg(unix)]
#[test]
fn native_rejects_each_symlink_leaf_before_changing_any_output() {
    use std::os::unix::fs::symlink;
    for selected in ["program.rs", "rust-backend.json", EXECUTABLE] {
        let (root, source) = fixture();
        let directory = root.path().join("native");
        fs::create_dir(&directory).unwrap();
        for name in ["program.rs", "rust-backend.json", EXECUTABLE] {
            fs::write(directory.join(name), format!("old {name}")).unwrap();
        }
        let victim = root.path().join("victim");
        fs::write(&victim, "keep victim").unwrap();
        fs::remove_file(directory.join(selected)).unwrap();
        symlink(&victim, directory.join(selected)).unwrap();
        let output = command("build-native", &source, &directory)
            .env("PATH", "")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        for name in ["program.rs", "rust-backend.json", EXECUTABLE] {
            if name != selected {
                assert_eq!(
                    fs::read_to_string(directory.join(name)).unwrap(),
                    format!("old {name}")
                );
            }
        }
        assert_eq!(fs::read_to_string(victim).unwrap(), "keep victim");
        assert_no_staging(root.path());
    }
}

#[test]
fn regular_outputs_are_updated_and_native_unrelated_files_are_preserved() {
    let (root, source) = fixture();
    let rust = root.path().join("program.rs");
    fs::write(&rust, "old Rust output").unwrap();
    let report = rust.with_extension("rust-backend.json");
    fs::write(&report, "old metadata").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for path in [&rust, &report] {
            fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
        }
    }
    assert_success(&emit(&source, &rust));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for path in [&rust, &report] {
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
    let directory = root.path().join("native");
    fs::create_dir(&directory).unwrap();
    for name in ["program.rs", "rust-backend.json", EXECUTABLE] {
        fs::write(directory.join(name), format!("old {name}")).unwrap();
    }
    fs::write(directory.join("keep.txt"), "unrelated output").unwrap();
    assert_success(
        &command("build-native", &source, &directory)
            .output()
            .unwrap(),
    );
    let executed = Command::new(directory.join(EXECUTABLE)).output().unwrap();
    assert_success(&executed);
    assert_eq!(executed.stdout, b"42\n");
    assert_eq!(
        fs::read_to_string(directory.join("keep.txt")).unwrap(),
        "unrelated output"
    );
    assert_eq!(fs::read_to_string(source).unwrap(), SOURCE);
    assert_no_staging(root.path());
}

#[cfg(unix)]
#[test]
fn rustc_failure_preserves_all_previous_outputs_and_cleans_owned_staging() {
    use std::os::unix::fs::PermissionsExt;
    let (root, source) = fixture();
    let directory = root.path().join("native");
    fs::create_dir(&directory).unwrap();
    for name in ["program.rs", "rust-backend.json", EXECUTABLE, "keep.txt"] {
        fs::write(directory.join(name), format!("old {name}")).unwrap();
    }
    let compiler_directory = root.path().join("compiler");
    fs::create_dir(&compiler_directory).unwrap();
    let compiler = compiler_directory.join("rustc");
    fs::write(&compiler, "#!/bin/sh\nprintf '%s\\n' \"$5\" > \"$SESERAGI_TEST_STAGE_LOG\"\nprintf 'forced rustc failure\\n' >&2\nexit 17\n").unwrap();
    fs::set_permissions(compiler, fs::Permissions::from_mode(0o700)).unwrap();
    let stage_log = root.path().join("stage-path");
    let output = command("build-native", &source, &directory)
        .env("PATH", &compiler_directory)
        .env("SESERAGI_TEST_STAGE_LOG", &stage_log)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("forced rustc failure"));
    for name in ["program.rs", "rust-backend.json", EXECUTABLE, "keep.txt"] {
        assert_eq!(
            fs::read_to_string(directory.join(name)).unwrap(),
            format!("old {name}")
        );
    }
    let staged = PathBuf::from(fs::read_to_string(stage_log).unwrap().trim());
    assert!(staged.to_string_lossy().contains(".seseragi-rust-stage-"));
    assert!(!staged.parent().unwrap().exists());
    assert_no_staging(root.path());
}

#[cfg(target_os = "macos")]
#[test]
fn publish_io_failure_rolls_back_and_removes_owned_staging() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    struct ClearImmutable(PathBuf);
    impl Drop for ClearImmutable {
        fn drop(&mut self) {
            // This guard runs before the owning TempDir, including on assertion
            // failure, so an inherited immutable flag cannot strand a fixture.
            let _ = Command::new("/usr/bin/chflags")
                .args(["-R", "nouchg"])
                .arg(&self.0)
                .output();
        }
    }

    for blocked_index in [1, 2] {
        for missing_first in [false, true] {
            let (root, source) = fixture();
            let _clear_flags = ClearImmutable(root.path().to_owned());
            let directory = root.path().join("native");
            fs::create_dir(&directory).unwrap();
            let names = ["program.rs", "rust-backend.json", EXECUTABLE];
            let mut previous_inodes = Vec::new();
            for (index, name) in names.iter().enumerate() {
                if missing_first && index == 0 {
                    previous_inodes.push(None);
                    continue;
                }
                let path = directory.join(name);
                fs::write(&path, format!("old {name}")).unwrap();
                fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
                previous_inodes.push(Some(fs::metadata(path).unwrap().ino()));
            }
            fs::write(directory.join("keep.txt"), "unrelated output").unwrap();
            let blocked = directory.join(names[blocked_index]);
            assert_success(
                &Command::new("/usr/bin/chflags")
                    .arg("uchg")
                    .arg(&blocked)
                    .output()
                    .unwrap(),
            );
            assert!(!fs::metadata(&blocked).unwrap().permissions().readonly());

            let compiler_directory = root.path().join("compiler");
            fs::create_dir(&compiler_directory).unwrap();
            let compiler = compiler_directory.join("rustc");
            fs::write(
                &compiler,
                "#!/bin/sh\nset -eu\nprintf 'staged executable\\n' > \"$7\"\n",
            )
            .unwrap();
            fs::set_permissions(compiler, fs::Permissions::from_mode(0o700)).unwrap();
            let output = command("build-native", &source, &directory)
                .env("PATH", compiler_directory)
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(2));
            let diagnostic = String::from_utf8_lossy(&output.stderr);
            assert!(
                diagnostic.contains(&format!(
                    "failed to publish Rust output {}",
                    fs::canonicalize(&blocked).unwrap().display()
                )),
                "{diagnostic}"
            );
            assert!(!diagnostic.contains("rollback failed"), "{diagnostic}");
            for (index, name) in names.iter().enumerate() {
                let path = directory.join(name);
                if let Some(previous) = previous_inodes[index] {
                    assert_eq!(fs::read_to_string(&path).unwrap(), format!("old {name}"));
                    let metadata = fs::metadata(&path).unwrap();
                    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
                    if index < blocked_index {
                        assert_ne!(metadata.ino(), previous);
                    }
                } else {
                    assert!(!path.exists());
                }
            }
            assert_eq!(fs::read_to_string(&source).unwrap(), SOURCE);
            assert_eq!(
                fs::read_to_string(directory.join("keep.txt")).unwrap(),
                "unrelated output"
            );
            assert_no_staging(root.path());
            drop(_clear_flags);
            let fixture_path = root.path().to_owned();
            root.close().unwrap();
            assert!(!fixture_path.exists());
        }
    }
}
