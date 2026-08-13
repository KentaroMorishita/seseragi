use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn test_directory(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "seseragi-new-{name}-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).unwrap();
    directory
}

#[test]
fn creates_the_canonical_web_starter_and_builds_it() {
    let directory = test_directory("web");
    let project = directory.join("hello-web");
    let created = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["new", "web"])
        .arg(&project)
        .output()
        .unwrap();
    assert_eq!(
        created.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&created.stderr)
    );
    let stdout = String::from_utf8_lossy(&created.stdout);
    assert!(stdout.contains("Created Seseragi Web project"));
    assert!(stdout.contains("seseragi dev --open"));

    let canonical = repository_root().join("examples/samples/web-starter");
    for source in ["app.ssrg", "main.ssrg"] {
        assert_eq!(
            fs::read(project.join("src").join(source)).unwrap(),
            fs::read(canonical.join("src").join(source)).unwrap(),
            "{source} must have one canonical source"
        );
    }
    let manifest = fs::read_to_string(project.join("seseragi.toml")).unwrap();
    assert!(manifest.contains("name = \"hello-web\""));
    assert!(manifest.contains("target = \"web\""));
    assert!(!manifest.contains("samples/web-starter"));

    let built = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["build", "."])
        .current_dir(&project)
        .output()
        .unwrap();
    assert_eq!(
        built.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert!(project.join("dist/index.html").is_file());
    assert!(project.join("dist/assets/app.js").is_file());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejects_invalid_templates_names_and_existing_destinations() {
    let directory = test_directory("safety");
    let existing = directory.join("existing-app");
    fs::create_dir(&existing).unwrap();
    fs::write(existing.join("keep.txt"), "keep").unwrap();

    for arguments in [
        vec!["new", "api", "other-app"],
        vec!["new", "web", "Invalid-App"],
        vec!["new", "web", "existing-app"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(arguments)
            .current_dir(&directory)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
    }
    assert_eq!(
        fs::read_to_string(existing.join("keep.txt")).unwrap(),
        "keep"
    );
    assert!(!directory.join("other-app").exists());
    assert!(!directory.join("Invalid-App").exists());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn documents_the_web_scaffold_in_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("seseragi new web path/to/my-app"));
}
