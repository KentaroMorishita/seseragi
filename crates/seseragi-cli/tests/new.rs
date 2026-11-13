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
    let ignore = fs::read_to_string(project.join(".gitignore")).unwrap();
    assert_eq!(
        ignore,
        fs::read_to_string(canonical.join(".gitignore")).unwrap()
    );
    assert!(ignore.lines().any(|line| line == ".seseragi/"));
    for source in ["app.ssrg", "main.ssrg"] {
        assert_eq!(
            fs::read(project.join("src").join(source)).unwrap(),
            fs::read(canonical.join("src").join(source)).unwrap(),
            "{source} must have one canonical source"
        );
    }
    assert_eq!(
        fs::read(project.join("tests/app.ssrg")).unwrap(),
        fs::read(canonical.join("tests/app.ssrg")).unwrap(),
        "the starter test must have one canonical source"
    );
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

    // The starter test runs without picking a target, although the app targets the web.
    let tested = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["test", "."])
        .current_dir(&project)
        .output()
        .unwrap();
    assert_eq!(
        tested.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&tested.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&tested.stdout),
        "PASS app::counter::Increment adds one\n1 passed; 0 failed; 0 skipped\n"
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn creates_the_cli_template_and_runs_it_on_both_targets() {
    let directory = test_directory("cli");
    let project = directory.join("hello-cli");
    let seseragi = |arguments: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(arguments)
            .current_dir(&project)
            .output()
            .unwrap()
    };
    let created = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["new", "cli"])
        .arg(&project)
        .output()
        .unwrap();
    assert_eq!(
        created.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&created.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&created.stdout),
        format!(
            "Created Seseragi CLI project at {}\nNext:\n  cd {}\n  seseragi run .\n  seseragi test\n  seseragi build . --target native\n",
            project.display(),
            project.display()
        )
    );

    let canonical = repository_root().join("examples/templates/cli");
    for file in [".gitignore", "src/main.ssrg", "tests/main.ssrg"] {
        assert_eq!(
            fs::read(project.join(file)).unwrap(),
            fs::read(canonical.join(file)).unwrap(),
            "{file} must have one canonical source"
        );
    }
    let manifest = fs::read_to_string(project.join("seseragi.toml")).unwrap();
    assert!(manifest.contains("name = \"hello-cli\""));
    assert!(manifest.contains("target = \"process\""));
    assert!(project.join("seseragi.lock").is_file());

    let run = seseragi(&["run", "."]);
    assert_eq!(
        run.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "Hello, World!\n");

    for target in ["node", "native"] {
        let tested = seseragi(&["test", "--target", target]);
        assert_eq!(
            tested.status.code(),
            Some(0),
            "{target}: {}",
            String::from_utf8_lossy(&tested.stderr)
        );
        assert!(
            String::from_utf8_lossy(&tested.stdout).ends_with("2 passed; 0 failed; 0 skipped\n")
        );
    }

    let built = seseragi(&["build", ".", "--target", "native"]);
    assert_eq!(
        built.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let program = project.join("dist").join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    let greeted = Command::new(&program).arg("Alice").output().unwrap();
    assert_eq!(greeted.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&greeted.stdout), "Hello, Alice!\n");
    let failed = Command::new(&program).arg("").output().unwrap();
    assert_eq!(failed.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&failed.stderr).contains("name must not be empty"));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn lists_templates_when_new_has_no_arguments() {
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("new")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Templates:"), "{stderr}");
    assert!(stderr.contains("  cli "), "{stderr}");
    assert!(stderr.contains("  web "), "{stderr}");
}

#[test]
fn builds_and_documents_the_web_starter_in_any_order() {
    let directory = test_directory("doc-build");
    let project = directory.join("hello-web");
    let seseragi = |arguments: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(arguments)
            .current_dir(&project)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(0),
            "seseragi {}: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    };
    let created = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["new", "web"])
        .arg(&project)
        .output()
        .unwrap();
    assert_eq!(created.status.code(), Some(0));

    // `doc` first leaves only `dist/doc`, which the build may replace.
    seseragi(&["doc", "."]);
    seseragi(&["build", "."]);
    assert!(project.join("dist/index.html").is_file());
    assert!(!project.join("dist/doc").exists());
    seseragi(&["doc", "."]);
    assert!(project.join("dist/doc/search-index.json").is_file());
    seseragi(&["build", "."]);
    seseragi(&["build", "."]);
    seseragi(&["doc", "."]);
    assert!(project.join("dist/assets/app.js").is_file());
    assert!(project.join("dist/doc/index.html").is_file());
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
        .args(["new", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("seseragi new web path/to/my-app"));
}
