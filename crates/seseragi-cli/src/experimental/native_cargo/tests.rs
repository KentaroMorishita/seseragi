use super::*;

const RUNTIME: &str = "pub fn answer() -> i64 { 42 }\n";
const SOURCE: &str = "fn main() { println!(\"{}\", seseragi_native_runtime::answer()); }\n";

struct Fixture {
    root: tempfile::TempDir,
    target: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::Builder::new()
            .prefix("native cargo 雪 ")
            .tempdir()
            .unwrap();
        fs::write(root.path().join("main.ssrg"), "pub fn main -> Int = 42\n").unwrap();
        let target = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.path().to_owned())
            .join("seseragi-native-tests")
            .join(root.path().file_name().unwrap());
        Self { root, target }
    }

    fn project(&self) -> Project {
        let mut project = Project::open(&self.root.path().join("main.ssrg"), "main").unwrap();
        project.target = self.target.clone();
        project
    }

    fn executable(&self) -> PathBuf {
        self.root.path().join(super::super::output::EXECUTABLE)
    }

    fn stdout(&self) -> String {
        let output = Command::new(self.executable()).output().unwrap();
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        String::from_utf8(output.stdout).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if self.target.exists() {
            fs::remove_dir_all(&self.target).unwrap();
        }
    }
}

fn modified(path: &Path) -> std::time::SystemTime {
    fs::metadata(path).unwrap().modified().unwrap()
}

fn runtime_library(target: &Path) -> PathBuf {
    let libraries = fs::read_dir(target.join("debug/deps"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("libseseragi_native_runtime-")
                && path
                    .extension()
                    .is_some_and(|extension| extension == "rlib")
        })
        .collect::<Vec<_>>();
    assert_eq!(libraries.len(), 1, "{libraries:?}");
    libraries.into_iter().next().unwrap()
}

#[test]
fn cargo_reuses_files_and_distinguishes_program_and_runtime_changes() {
    let fixture = Fixture::new();
    let project = fixture.project();
    let build = |source: &str, runtime: &str| {
        project
            .build(
                BuildProfile::Development,
                source,
                runtime,
                &fixture.executable(),
            )
            .unwrap();
    };
    build(SOURCE, RUNTIME);
    assert_eq!(fixture.stdout(), "42\n");
    let files = [
        "Cargo.toml",
        "Cargo.lock",
        "build.rs",
        "program.rs",
        "runtime/Cargo.toml",
        "runtime/lib.rs",
    ]
    .map(|file| project.directory.join(file));
    let times = files.each_ref().map(|file| modified(file));
    let library = runtime_library(&project.target);
    let library_time = modified(&library);
    let binary = project
        .target
        .join(format!("debug/{}", super::super::output::EXECUTABLE));
    let binary_time = modified(&binary);
    build(SOURCE, RUNTIME);
    assert_eq!(files.each_ref().map(|file| modified(file)), times);
    assert_eq!(modified(&library), library_time);
    assert_eq!(modified(&binary), binary_time);

    let changed = SOURCE.replace("{}", "changed {}");
    build(&changed, RUNTIME);
    assert_eq!(fixture.stdout(), "changed 42\n");
    assert_ne!(modified(&files[3]), times[3]);
    for index in [0, 1, 2, 4, 5] {
        assert_eq!(modified(&files[index]), times[index]);
    }
    assert_eq!(modified(&library), library_time);
    assert_ne!(modified(&binary), binary_time);

    let program_time = modified(&files[3]);
    build(&changed, &RUNTIME.replace("42", "43"));
    assert_eq!(fixture.stdout(), "changed 43\n");
    assert_ne!(modified(&library), library_time);
    assert_ne!(modified(&files[5]), times[5]);
    assert_eq!(modified(&files[3]), program_time);
    assert_eq!(modified(&files[1]), times[1]);
    let directory = project.directory.clone();
    let target = project.target.clone();
    drop(project);
    let reopened = fixture.project();
    assert_eq!(reopened.directory, directory);
    assert_eq!(reopened.target, target);
    assert!(!fixture.root.path().join("Cargo.toml").exists());
    assert!(!fixture.root.path().join("target").exists());
}

#[test]
fn compiler_failure_preserves_the_executable_and_recovery_uses_both_profiles() {
    let fixture = Fixture::new();
    let project = fixture.project();
    project
        .build(
            BuildProfile::Development,
            SOURCE,
            RUNTIME,
            &fixture.executable(),
        )
        .unwrap();
    let previous = fs::read(fixture.executable()).unwrap();
    let error = project
        .build(
            BuildProfile::Development,
            "fn main() { missing(); }",
            RUNTIME,
            &fixture.executable(),
        )
        .unwrap_err();
    assert!(error.contains("Rust target compilation failed"), "{error}");
    assert!(error.contains("missing"), "{error}");
    assert_eq!(fs::read(fixture.executable()).unwrap(), previous);
    assert_eq!(fixture.stdout(), "42\n");
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        project
            .build(profile, SOURCE, RUNTIME, &fixture.executable())
            .unwrap();
        assert_eq!(fixture.stdout(), "42\n");
    }
    assert!(project
        .target
        .join(format!("release/{}", super::super::output::EXECUTABLE))
        .is_file());
    let manifest = fs::read_to_string(project.directory.join("Cargo.toml"))
        .unwrap()
        .parse::<toml::Table>()
        .unwrap();
    assert_eq!(
        manifest["profile"]["dev"]["opt-level"].as_integer(),
        Some(0)
    );
    assert_eq!(
        manifest["profile"]["release"]["opt-level"].as_integer(),
        Some(3)
    );
    assert!(!manifest["profile"]["dev"]
        .as_table()
        .unwrap()
        .contains_key("debug"));
    assert!(!manifest["profile"]["dev"]
        .as_table()
        .unwrap()
        .contains_key("incremental"));
}

#[cfg(unix)]
#[test]
fn external_crate_and_system_c_library_link_offline_and_bad_link_preserves_binary() {
    let fixture = Fixture::new();
    let root = fixture.root.path();
    fs::create_dir(root.join("adapter 雪")).unwrap();
    fs::write(root.join("adapter 雪/Cargo.toml"), "[package]\nname = \"adapter\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[lib]\npath = \"lib.rs\"\n[features]\nselected = []\n").unwrap();
    fs::write(
        root.join("adapter 雪/lib.rs"),
        "#[cfg(feature = \"selected\")] pub fn input() -> f64 { std::hint::black_box(0.0) }\n",
    )
    .unwrap();
    let config = "[tool.native]\nlibraries = [\"m\"]\nsearch-paths = [\"adapter 雪\"]\n[tool.native.dependencies]\nrenamed = { package = \"adapter\", path = \"adapter 雪\", features = [\"selected\"], default-features = false }\n";
    fs::write(root.join("seseragi.toml"), config).unwrap();
    let project = fixture.project();
    let source = "unsafe extern \"C\" { fn cos(value: f64) -> f64; }\nfn main() { println!(\"{:.0}\", unsafe { cos(renamed::input()) }); }\n";
    project
        .build(
            BuildProfile::Development,
            source,
            RUNTIME,
            &fixture.executable(),
        )
        .unwrap();
    assert_eq!(fixture.stdout(), "1\n");
    let lock = fs::read_to_string(project.directory.join("Cargo.lock")).unwrap();
    assert!(lock.contains("name = \"adapter\""), "{lock}");
    let runtime = runtime_library(&project.target);
    let runtime_time = modified(&runtime);
    fs::write(root.join("adapter 雪/lib.rs"), "#[cfg(feature = \"selected\")] pub fn input() -> f64 { std::hint::black_box(std::f64::consts::PI) }\n").unwrap();
    project
        .build(
            BuildProfile::Development,
            source,
            RUNTIME,
            &fixture.executable(),
        )
        .unwrap();
    assert_eq!(fixture.stdout(), "-1\n");
    assert_eq!(modified(&runtime), runtime_time);
    let previous = fs::read(fixture.executable()).unwrap();
    drop(project);
    fs::write(
        root.join("seseragi.toml"),
        config.replace("[\"m\"]", "[\"seseragi_nonexistent_library_1206\"]"),
    )
    .unwrap();
    let project = fixture.project();
    let error = project
        .build(
            BuildProfile::Development,
            source,
            RUNTIME,
            &fixture.executable(),
        )
        .unwrap_err();
    assert!(
        error.contains("seseragi_nonexistent_library_1206"),
        "{error}"
    );
    assert_eq!(fs::read(fixture.executable()).unwrap(), previous);
    assert_eq!(fixture.stdout(), "-1\n");
}

#[test]
fn native_config_rejects_invalid_settings_and_runtime_replacement() {
    let fixture = Fixture::new();
    for invalid in [
        "native = 1",
        "[native]\nunknown = 1",
        "[native]\nlibraries = \"m\"",
        "[native]\nlibraries = [\"\"]",
        "[native]\nsearch-paths = [1]",
        "[native]\nlibraries = [\"m\\ninvalid\"]",
        "[native]\ndependencies = []",
        "[native.dependencies]\nfoo = 1",
        "[native.dependencies]\nfoo = { path = 1 }",
        "[native.dependencies]\nseseragi_native_runtime = \"1\"",
        "[native.dependencies]\nseseragi-native-runtime = \"1\"",
    ] {
        fs::write(
            fixture.root.path().join("seseragi.toml"),
            format!("[tool]\n{}", invalid.replace("[native", "[tool.native")),
        )
        .unwrap();
        assert!(Config::read(fixture.root.path()).is_err(), "{invalid}");
    }
    assert!(!fixture.root.path().join(".seseragi").exists());
}

#[cfg(unix)]
#[test]
fn native_build_rejects_symlinked_build_inputs() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let victim = fixture.root.path().join("victim");
    fs::write(&victim, "keep victim").unwrap();
    let project = fixture.project();
    symlink(&victim, project.directory.join("Cargo.lock")).unwrap();
    assert!(project
        .build(
            BuildProfile::Development,
            SOURCE,
            RUNTIME,
            &fixture.executable()
        )
        .unwrap_err()
        .contains("regular file"));
    assert_eq!(fs::read_to_string(&victim).unwrap(), "keep victim");
    fs::remove_file(project.directory.join("Cargo.lock")).unwrap();
    fs::remove_file(project.directory.join("runtime/lib.rs")).unwrap();
    symlink(&victim, project.directory.join("runtime/lib.rs")).unwrap();
    assert!(project
        .build(
            BuildProfile::Development,
            SOURCE,
            RUNTIME,
            &fixture.executable()
        )
        .unwrap_err()
        .contains("regular file"));
    assert_eq!(fs::read_to_string(&victim).unwrap(), "keep victim");
    drop(project);
    let other = tempfile::tempdir().unwrap();
    symlink(fixture.root.path(), other.path().join(".seseragi")).unwrap();
    fs::write(other.path().join("main.ssrg"), "").unwrap();
    assert!(Project::open(&other.path().join("main.ssrg"), "main")
        .err()
        .unwrap()
        .contains("symlink"));
}

#[test]
fn concurrent_builds_of_the_same_project_keep_each_published_program() {
    let fixture = Fixture::new();
    std::thread::scope(|scope| {
        for value in [41, 42, 43] {
            let fixture = &fixture;
            scope.spawn(move || {
                let project = fixture.project();
                let output = fixture
                    .root
                    .path()
                    .join(format!("program-{value}{}", std::env::consts::EXE_SUFFIX));
                project
                    .build(
                        BuildProfile::Development,
                        &format!("fn main() {{ println!(\"{value}\"); }}"),
                        RUNTIME,
                        &output,
                    )
                    .unwrap();
                let result = Command::new(output).output().unwrap();
                assert!(result.status.success(), "{result:?}");
                assert_eq!(result.stdout, format!("{value}\n").as_bytes());
            });
        }
    });
}
