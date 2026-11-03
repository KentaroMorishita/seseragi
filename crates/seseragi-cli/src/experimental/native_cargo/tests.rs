use super::*;

#[path = "../../../../test-support/native_cache.rs"]
mod native_cache;

const RUNTIME: &str = "pub fn answer() -> i64 { 42 }\n";
const SOURCE: &str = "fn main() { println!(\"{}\", seseragi_native_runtime::answer()); }\n";

#[test]
fn cargo_targets_use_one_full_key_and_separate_inputs_and_entry_points() {
    let root = tempfile::tempdir().unwrap();
    let source = root
        .path()
        .join("workspace/packages/examples/nested/native-project");
    fs::create_dir_all(&source).unwrap();
    for name in ["main.ssrg", "other.ssrg"] {
        fs::write(source.join(name), "").unwrap();
    }
    let mut targets = Vec::new();
    for (name, entry) in [
        ("main.ssrg", "main"),
        ("main.ssrg", "other"),
        ("other.ssrg", "main"),
        ("main.ssrg", "main"),
    ] {
        let mut project = Project::open(&source.join(name), entry).unwrap();
        let default = target_root(&project.directory, None).unwrap();
        if cfg!(windows) {
            assert_eq!(
                default,
                native_toolchain::cache_root().unwrap().join("build")
            );
        } else {
            assert_eq!(default, project.directory.parent().unwrap().join("target"));
        }
        assert!(!default.starts_with(&project.directory));
        let selected = root.path().join("cargo-output");
        assert_eq!(
            target_root(&project.directory, Some(selected.clone().into())).unwrap(),
            selected.join("seseragi-native")
        );
        let relative = PathBuf::from("native cargo output 雪");
        assert_eq!(
            target_root(&project.directory, Some(relative.clone().into())).unwrap(),
            std::path::absolute(relative)
                .unwrap()
                .join("seseragi-native")
        );
        assert_eq!(
            target_root(&project.directory, Some("".into())).unwrap(),
            default
        );

        // Model the default Windows cache independently of the checkout depth.
        project.target = "C:/Users/developer/AppData/Local/seseragi/cache/native/build".into();
        let target = project
            .target_for(BuildProfile::Development, RUNTIME)
            .unwrap();
        assert_eq!(target.parent(), Some(project.target.as_path()));
        let key = target.file_name().unwrap().to_str().unwrap();
        assert_eq!(key.len(), 64);
        assert!(key.bytes().all(|byte| byte.is_ascii_hexdigit()));
        for artifact in [
            "debug/build/seseragi-program-0123456789abcdef/build_script_build-0123456789abcdef.exe",
            "x86_64-pc-windows-msvc/debug/deps/libseseragi_native_runtime-0123456789abcdef.rlib",
        ] {
            let path = target.join(artifact);
            assert!(
                path.as_os_str().to_string_lossy().encode_utf16().count() < 260,
                "{}",
                path.display()
            );
        }
        targets.push(target);
    }
    assert_ne!(targets[0], targets[1]);
    assert_ne!(targets[0], targets[2]);
    assert_eq!(targets[0], targets[3]);
}

#[test]
fn path_remapping_preserves_environment_flag_precedence_and_argument_boundaries() {
    let remapping = "native cargo 雪 =.";
    for (encoded, plain, prefix) in [
        (None, None, None),
        (
            None,
            Some("-C opt-level=1\t--cfg caller"),
            Some("-C\x1fopt-level=1\x1f--cfg\x1fcaller"),
        ),
        (
            Some("--cfg\x1fencoded"),
            Some("ignored"),
            Some("--cfg\x1fencoded"),
        ),
        (Some(""), Some("ignored"), Some("")),
    ] {
        let actual =
            remapped_environment_flags(encoded.map(Into::into), plain.map(Into::into), remapping)
                .unwrap();
        let expected = prefix.map(|prefix| {
            let prefix = if prefix.is_empty() {
                String::new()
            } else {
                format!("{prefix}\x1f")
            };
            OsString::from(format!("{prefix}--remap-path-prefix\x1f{remapping}"))
        });
        assert_eq!(actual, expected);
    }
}

struct Fixture {
    root: tempfile::TempDir,
    target: PathBuf,
    toolchain: Toolchain,
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
        let toolchain = Toolchain::load(
            native_cache::toolchain().to_owned(),
            root.path().join("cargo-home"),
            native_toolchain::host().unwrap(),
        )
        .unwrap();
        Self {
            root,
            target,
            toolchain,
        }
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
        let output = run_published(&self.executable());
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
    let libraries = fs::read_dir(
        target
            .join(native_toolchain::host().unwrap())
            .join("debug/deps"),
    )
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
                &fixture.toolchain,
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
    let library = runtime_library(
        &project
            .target_for(BuildProfile::Development, RUNTIME)
            .unwrap(),
    );
    let library_time = modified(&library);
    let binary = project
        .target_for(BuildProfile::Development, RUNTIME)
        .unwrap()
        .join(native_toolchain::host().unwrap())
        .join(format!("debug/{}", super::super::output::EXECUTABLE));
    let binary_time = modified(&binary);
    build(SOURCE, RUNTIME);
    assert_eq!(fixture.stdout(), "42\n");
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
    let changed_runtime = RUNTIME.replace("42", "43");
    build(&changed, &changed_runtime);
    assert_eq!(fixture.stdout(), "changed 43\n");
    let rebuilt = runtime_library(
        &project
            .target_for(BuildProfile::Development, &changed_runtime)
            .unwrap(),
    );
    assert_ne!(rebuilt, library);
    assert_ne!(modified(&rebuilt), library_time);
    assert_eq!(modified(&library), library_time);
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
            &fixture.toolchain,
        )
        .unwrap();
    let previous = fs::read(fixture.executable()).unwrap();
    let error = project
        .build(
            BuildProfile::Development,
            "fn main() { missing(); }",
            RUNTIME,
            &fixture.executable(),
            &fixture.toolchain,
        )
        .unwrap_err();
    assert!(error.contains("Rust target compilation failed"), "{error}");
    assert!(error.contains("missing"), "{error}");
    assert_eq!(fs::read(fixture.executable()).unwrap(), previous);
    assert_eq!(fixture.stdout(), "42\n");
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        project
            .build(
                profile,
                SOURCE,
                RUNTIME,
                &fixture.executable(),
                &fixture.toolchain,
            )
            .unwrap();
        assert_eq!(fixture.stdout(), "42\n");
    }
    assert!(project
        .target_for(BuildProfile::Release, RUNTIME)
        .unwrap()
        .join(native_toolchain::host().unwrap())
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

#[test]
fn build_cache_separates_runtime_profiles_features_and_linker_settings() {
    let fixture = Fixture::new();
    let mut project = fixture.project();
    let development = project
        .target_for(BuildProfile::Development, RUNTIME)
        .unwrap();
    let release = project.target_for(BuildProfile::Release, RUNTIME).unwrap();
    assert_ne!(development, release);
    assert_ne!(
        development,
        project
            .target_for(BuildProfile::Development, "changed runtime")
            .unwrap()
    );
    project.config.dependencies.insert(
        "adapter".into(),
        toml::Value::Table(toml::Table::from_iter([
            ("version".into(), "1.0".into()),
            (
                "features".into(),
                toml::Value::Array(vec!["selected".into()]),
            ),
        ])),
    );
    let features = project
        .target_for(BuildProfile::Development, RUNTIME)
        .unwrap();
    assert_ne!(development, features);
    project.config.libraries.push("m".into());
    let linking = project
        .target_for(BuildProfile::Development, RUNTIME)
        .unwrap();
    assert_ne!(features, linking);
    project.config.search_paths.push("native/lib".into());
    assert_ne!(
        linking,
        project
            .target_for(BuildProfile::Development, RUNTIME)
            .unwrap()
    );
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
            &fixture.toolchain,
        )
        .unwrap();
    assert_eq!(fixture.stdout(), "1\n");
    let lock = fs::read_to_string(project.directory.join("Cargo.lock")).unwrap();
    assert!(lock.contains("name = \"adapter\""), "{lock}");
    let runtime = runtime_library(
        &project
            .target_for(BuildProfile::Development, RUNTIME)
            .unwrap(),
    );
    let runtime_time = modified(&runtime);
    fs::write(root.join("adapter 雪/lib.rs"), "#[cfg(feature = \"selected\")] pub fn input() -> f64 { std::hint::black_box(std::f64::consts::PI) }\n").unwrap();
    project
        .build(
            BuildProfile::Development,
            source,
            RUNTIME,
            &fixture.executable(),
            &fixture.toolchain,
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
            &fixture.toolchain,
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
            &fixture.executable(),
            &fixture.toolchain
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
            &fixture.executable(),
            &fixture.toolchain
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

/// Runs a published program. A sibling test thread may fork while this program was
/// still being copied, leaving the new executable briefly open for writing in
/// that child until it execs. Linux reports that as ETXTBSY, so retry briefly.
fn run_published(path: &Path) -> std::process::Output {
    let mut attempts = 0;
    loop {
        match Command::new(path).output() {
            Err(error) if error.kind() == io::ErrorKind::ExecutableFileBusy && attempts < 100 => {
                attempts += 1;
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            result => return result.unwrap(),
        }
    }
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
                        &fixture.toolchain,
                    )
                    .unwrap();
                let result = run_published(&output);
                assert!(result.status.success(), "{result:?}");
                assert_eq!(result.stdout, format!("{value}\n").as_bytes());
            });
        }
    });
}
