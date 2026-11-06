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
            let native = project.directory.parent().unwrap();
            assert_eq!(default, native.parent().unwrap().join("native-target"));
            assert!(!default.starts_with(native));
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
        for path in [
            target.join("debug/build/seseragi-program-0123456789abcdef/build_script_build-0123456789abcdef.exe"),
            target.join("x86_64-pc-windows-msvc/debug/deps/program-0123456789abcdef.exe"),
            project.target.join("runtime").join("0".repeat(32)).join(
                "target/x86_64-pc-windows-msvc/debug/deps/libseseragi_native_runtime-0123456789abcdef.rlib",
            ),
        ] {
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
fn rustflags_keep_the_selected_source_and_append_path_flags() {
    let path_flags = [
        "--remap-path-prefix".to_owned(),
        "native cargo 雪 =.".to_owned(),
    ];
    let configured = ["-C".to_owned(), "opt-level=1".to_owned()];
    for (encoded, plain, prefix) in [
        // Without environment flags, the flags Cargo selected from its
        // config (for example target.<host>.rustflags) are kept.
        (None, None, "-C\x1fopt-level=1"),
        (
            None,
            Some("-C opt-level=2\t--cfg caller"),
            "-C\x1fopt-level=2\x1f--cfg\x1fcaller",
        ),
        (
            Some("--cfg\x1fencoded"),
            Some("ignored"),
            "--cfg\x1fencoded",
        ),
        (Some(""), Some("ignored"), ""),
    ] {
        let actual = rustflags_with_paths(
            encoded.map(Into::into),
            plain.map(Into::into),
            &configured,
            &path_flags,
        )
        .unwrap();
        let prefix = if prefix.is_empty() {
            String::new()
        } else {
            format!("{prefix}\x1f")
        };
        assert_eq!(
            actual,
            OsString::from(format!("{prefix}--remap-path-prefix\x1fnative cargo 雪 =."))
        );
    }
    assert_eq!(
        rustflags_with_paths(None, None, &[], &path_flags).unwrap(),
        OsString::from("--remap-path-prefix\x1fnative cargo 雪 =.")
    );
}

#[test]
fn apple_hosts_also_make_linker_object_paths_relative_to_the_target() {
    let directory = Path::new("/work/a,b/.seseragi/native/id");
    let target = Path::new("/work/a,b/.seseragi/native-target/key");
    let remap = ["--remap-path-prefix", "/work/a,b/.seseragi/native/id=."];
    for invocation in [LinkerInvocation::CompilerDriver, LinkerInvocation::Direct] {
        assert_eq!(
            path_flags(directory, target, "x86_64-unknown-linux-gnu", invocation),
            remap
        );
    }
    let link = |arguments: &[&str]| {
        let mut flags = remap.map(String::from).to_vec();
        flags.extend(
            arguments
                .iter()
                .map(|argument| format!("-Clink-arg={argument}")),
        );
        flags
    };
    let prefix = "/work/a,b/.seseragi/native-target/key/";
    assert_eq!(
        path_flags(
            directory,
            target,
            "aarch64-apple-darwin",
            LinkerInvocation::CompilerDriver
        ),
        link(&["-Xlinker", "-oso_prefix", "-Xlinker", prefix])
    );
    // A linker started directly rejects the compiler driver's -Xlinker.
    assert_eq!(
        path_flags(
            directory,
            target,
            "aarch64-apple-darwin",
            LinkerInvocation::Direct
        ),
        link(&["-oso_prefix", prefix])
    );
}

#[test]
fn linker_invocation_follows_rustc_flavor_and_linker_name_rules() {
    use LinkerInvocation::{CompilerDriver, Direct};
    let flags = |text: &str| {
        text.split_whitespace()
            .map(String::from)
            .collect::<Vec<_>>()
    };
    for (rustflags, environment, expected) in [
        ("", None, CompilerDriver),
        ("-C opt-level=1", None, CompilerDriver),
        ("-C linker=rust-lld", None, Direct),
        // rustc gives ld64.lld no hint, so the Apple default driver stays.
        ("-Clinker=/opt/llvm/bin/ld64.lld", None, CompilerDriver),
        // rustc removes a numeric suffix after the last `-`.
        ("-C linker=/opt/bin/ld-21", None, Direct),
        ("-C linker=rust-lld-19", None, Direct),
        ("-C linker=ld.lld-18.exe", None, Direct),
        ("-C linker=clang-18", None, CompilerDriver),
        ("-C linker=gcc-", None, CompilerDriver),
        ("-C linker=ld-x", None, CompilerDriver),
        ("", Some("/opt/bin/ld-21"), Direct),
        ("--codegen=linker=ld", None, Direct),
        ("--codegen linker=x86_64-apple-darwin-ld", None, Direct),
        ("-C linker=/usr/bin/clang", None, CompilerDriver),
        ("-C linker=cc", None, CompilerDriver),
        ("", Some("rust-lld"), Direct),
        ("", Some("/usr/bin/clang"), CompilerDriver),
        ("-C linker=clang", Some("rust-lld"), CompilerDriver),
        ("-C linker=rust-lld -C linker=clang", None, CompilerDriver),
        ("-C linker-flavor=darwin", None, Direct),
        ("-C linker-flavor=darwin-lld", None, Direct),
        ("-C linker-flavor=ld64.lld", None, Direct),
        ("-C linker-flavor=darwin-lld-cc", None, CompilerDriver),
        ("-C linker-flavor=gcc", None, CompilerDriver),
        (
            "-C linker=rust-lld -C linker-flavor=darwin-lld-cc",
            None,
            CompilerDriver,
        ),
        ("-C linker=clang -C linker-flavor=darwin-lld", None, Direct),
    ] {
        assert_eq!(
            linker_invocation(&flags(rustflags), environment),
            expected,
            "{rustflags} / {environment:?}"
        );
    }
}

#[test]
fn linker_settings_read_cargo_config_files_with_cargo_precedence() {
    let host = "aarch64-apple-darwin";
    let root = tempfile::tempdir().unwrap();
    let package = root.path().join("package");
    let directory = package.join(".seseragi/native/id");
    let home = root.path().join("cargo-home");
    for path in [&directory, &home] {
        fs::create_dir_all(path).unwrap();
    }
    let write = |directory: &Path, name: &str, source: &str| {
        fs::create_dir_all(directory).unwrap();
        fs::write(directory.join(name), source).unwrap();
    };
    let none = |_: &str| None;
    let resolve = |environment: &dyn Fn(&str) -> Option<OsString>| {
        cargo_config::linker_settings(&directory, &home, host, Some(&[]), environment)
    };
    assert_eq!(resolve(&none), cargo_config::LinkerSettings::default());

    // A linker set only in the package's Cargo config is the one Cargo uses.
    write(
        &package.join(".cargo"),
        "config.toml",
        "[target.aarch64-apple-darwin]\nlinker = \"/opt/llvm/bin/rust-lld\"\n[target.x86_64-apple-darwin]\nlinker = \"clang\"\n",
    );
    let settings = resolve(&none);
    assert_eq!(settings.linker.as_deref(), Some("/opt/llvm/bin/rust-lld"));
    assert_eq!(
        linker_invocation(&settings.rustflags, settings.linker.as_deref()),
        LinkerInvocation::Direct
    );

    // CARGO_HOME has the lowest precedence; a nearer `config` beats
    // `config.toml` in the same directory and every farther file.
    write(&home, "config.toml", "[target.aarch64-apple-darwin]\nlinker = \"home-ld\"\n[build]\nrustflags = \"-C opt-level=1\"\n");
    write(
        &directory.join(".cargo"),
        "config",
        "[target.aarch64-apple-darwin]\nlinker = \"clang\"\n",
    );
    write(
        &directory.join(".cargo"),
        "config.toml",
        "[target.aarch64-apple-darwin]\nlinker = \"rust-lld\"\n",
    );
    let settings = resolve(&none);
    assert_eq!(settings.linker.as_deref(), Some("clang"));
    assert_eq!(settings.rustflags, ["-C", "opt-level=1"]);

    // Target rustflags replace build rustflags; config arrays append in
    // precedence order, so a nearer -C linker comes last and wins.
    write(
        &package.join(".cargo"),
        "config.toml",
        "[target.aarch64-apple-darwin]\nrustflags = [\"-C\", \"linker=rust-lld\"]\n",
    );
    write(
        &directory.join(".cargo"),
        "config",
        "[target.aarch64-apple-darwin]\nrustflags = [\"-C\", \"linker=clang-18\"]\n",
    );
    let settings = resolve(&none);
    assert_eq!(
        settings.rustflags,
        ["-C", "linker=rust-lld", "-C", "linker=clang-18"]
    );
    assert_eq!(
        linker_invocation(&settings.rustflags, settings.linker.as_deref()),
        LinkerInvocation::CompilerDriver
    );

    // Environment variables override config files, and RUSTFLAGS replaces
    // every configured rustflags value.
    let environment = |key: &str| match key {
        "CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER" => Some(OsString::from("/opt/bin/ld-21")),
        "RUSTFLAGS" => Some(OsString::from("-C opt-level=2")),
        _ => None,
    };
    let settings = resolve(&environment);
    assert_eq!(settings.linker.as_deref(), Some("/opt/bin/ld-21"));
    assert_eq!(settings.rustflags, ["-C", "opt-level=2"]);
    assert_eq!(
        linker_invocation(&settings.rustflags, settings.linker.as_deref()),
        LinkerInvocation::Direct
    );

    // A file Cargo cannot parse is left for Cargo to report.
    write(&directory.join(".cargo"), "config", "[target");
    assert_eq!(resolve(&none).rustflags, ["-C", "linker=rust-lld"]);
}

#[test]
fn rustflags_strings_replace_arrays_join_and_empty_target_flags_use_build() {
    let host = "aarch64-apple-darwin";
    let root = tempfile::tempdir().unwrap();
    let package = root.path().join("package");
    let directory = package.join(".seseragi/native/id");
    let home = root.path().join("cargo-home");
    let write = |directory: &Path, source: &str| {
        fs::create_dir_all(directory).unwrap();
        fs::write(directory.join("config.toml"), source).unwrap();
    };
    let resolve = |far: &str, near: &str| {
        write(&package.join(".cargo"), far);
        write(&directory.join(".cargo"), near);
        cargo_config::linker_settings(&directory, &home, host, Some(&[]), &|_: &str| None).rustflags
    };
    let target = |value: &str| format!("[target.aarch64-apple-darwin]\nrustflags = {value}\n");
    let build = "[build]\nrustflags = [\"-Cdebuginfo=0\"]\n";

    // A nearer string replaces a farther string.
    assert_eq!(
        resolve(
            &target("\"-C linker=rust-lld\""),
            &target("\"-C opt-level=1\"")
        ),
        ["-C", "opt-level=1"]
    );
    // Arrays join with the farther values first.
    assert_eq!(
        resolve(&target("[\"-Ca\"]"), &target("[\"-Cb\"]")),
        ["-Ca", "-Cb"]
    );
    // A string and an array for the same key are a Cargo type error, so no
    // flags come from that key; build.rustflags is the next source.
    for (far, near) in [
        ("\"-Ca\"", "[\"-Cb\"]"),
        ("[\"-Ca\"]", "\"-Cb\""),
        ("[1]", "[\"-Cb\"]"),
    ] {
        assert_eq!(
            resolve(&format!("{build}{}", target(far)), &target(near)),
            ["-Cdebuginfo=0"],
            "{far} / {near}"
        );
    }
    // Empty target flags fall back to build.rustflags, as Cargo does.
    assert_eq!(resolve(build, &target("[]")), ["-Cdebuginfo=0"]);
    assert_eq!(resolve(build, &target("\"\"")), ["-Cdebuginfo=0"]);
}

#[test]
fn cfg_tables_see_cfg_values_from_the_selected_rustflags() {
    let host = "aarch64-apple-darwin";
    let root = tempfile::tempdir().unwrap();
    let directory = root.path().join("package/.seseragi/native/id");
    let home = root.path().join("cargo-home");
    fs::create_dir_all(directory.join(".cargo")).unwrap();
    fs::write(
        directory.join(".cargo/config.toml"),
        "[target.'cfg(use_direct_linker)']\nlinker = \"rust-lld\"\n",
    )
    .unwrap();
    let environment =
        |key: &str| (key == "RUSTFLAGS").then(|| OsString::from("--cfg use_direct_linker"));
    // Stands in for `rustc <rustflags> --print cfg`: a `--cfg` flag adds a
    // name to the printed cfg.
    let queries = std::cell::RefCell::new(Vec::new());
    let query = |rustflags: &[String]| {
        queries.borrow_mut().push(rustflags.to_vec());
        let mut cfg = cargo_config::parse_cfg("target_os=\"macos\"\nunix\n");
        cfg.extend(
            rustflags
                .windows(2)
                .filter(|pair| pair[0] == "--cfg")
                .map(|pair| (pair[1].clone(), None)),
        );
        cfg
    };
    let settings = cargo_config::settle_with_cfg(&query, &|cfg| {
        cargo_config::linker_settings(&directory, &home, host, cfg, &environment)
    });
    assert_eq!(settings.linker.as_deref(), Some("rust-lld"));
    assert_eq!(
        linker_invocation(&settings.rustflags, settings.linker.as_deref()),
        LinkerInvocation::Direct
    );
    assert!(queries
        .borrow()
        .iter()
        .any(|flags| flags == &["--cfg", "use_direct_linker"]));
}

#[test]
fn the_first_cfg_query_uses_rustflags_selected_without_cfg_tables() {
    let host = "aarch64-apple-darwin";
    let root = tempfile::tempdir().unwrap();
    let directory = root.path().join("package/.seseragi/native/id");
    let home = root.path().join("cargo-home");
    fs::create_dir_all(directory.join(".cargo")).unwrap();
    fs::write(
        directory.join(".cargo/config.toml"),
        "[build]\nrustflags = [\"--cfg\", \"use_cc\"]\n[target.'cfg(not(use_cc))']\nrustflags = [\"-C\", \"linker=/nonexistent/rust-lld\"]\n",
    )
    .unwrap();
    // Before the first query, no cfg table applies, not even `not(..)`.
    assert_eq!(
        cargo_config::linker_settings(&directory, &home, host, None, &|_: &str| None).rustflags,
        ["--cfg", "use_cc"]
    );
    assert_eq!(
        cargo_config::linker_settings(&directory, &home, host, Some(&[]), &|_: &str| None)
            .rustflags,
        ["-C", "linker=/nonexistent/rust-lld"]
    );
    let queries = std::cell::RefCell::new(Vec::new());
    let query = |rustflags: &[String]| {
        queries.borrow_mut().push(rustflags.to_vec());
        let mut cfg = cargo_config::parse_cfg("target_os=\"macos\"\nunix\n");
        cfg.extend(
            rustflags
                .windows(2)
                .filter(|pair| pair[0] == "--cfg")
                .map(|pair| (pair[1].clone(), None)),
        );
        cfg
    };
    let settings = cargo_config::settle_with_cfg(&query, &|cfg| {
        cargo_config::linker_settings(&directory, &home, host, cfg, &|_: &str| None)
    });
    // Cargo sees `use_cc`, so `cfg(not(use_cc))` does not apply and the
    // build rustflags stay.
    assert_eq!(queries.borrow()[0], ["--cfg", "use_cc"]);
    assert_eq!(settings.rustflags, ["--cfg", "use_cc"]);
    assert_eq!(settings.linker, None);
}

#[test]
fn build_rustflags_with_a_negated_cfg_table_build_like_cargo() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.root.path().join(".cargo")).unwrap();
    fs::write(
        fixture.root.path().join(".cargo/config.toml"),
        "[build]\nrustflags = [\"--cfg\", \"use_cc\"]\n[target.'cfg(not(use_cc))']\nrustflags = [\"-C\", \"linker=/nonexistent/rust-lld\"]\n",
    )
    .unwrap();
    let project = fixture.project();
    project
        .build(
            BuildProfile::Development,
            "fn main() { println!(\"use_cc={}\", cfg!(use_cc)); }\n",
            RUNTIME,
            &fixture.executable(),
            &fixture.toolchain,
        )
        .unwrap();
    assert_eq!(fixture.stdout(), "use_cc=true\n");
}

#[test]
fn host_cfg_passes_the_selected_rustflags_to_rustc() {
    let fixture = Fixture::new();
    let cfg = host_cfg(
        &fixture.toolchain,
        &["--cfg".into(), "use_direct_linker".into()],
    );
    assert!(cfg.contains(&("use_direct_linker".into(), None)), "{cfg:?}");
    assert!(
        cfg.contains(&("unix".into(), None)) || cfg!(windows),
        "{cfg:?}"
    );
    assert!(!host_cfg(&fixture.toolchain, &[]).contains(&("use_direct_linker".into(), None)));
}

#[test]
fn linker_settings_apply_matching_cfg_tables_after_the_target_triple() {
    let host = "aarch64-apple-darwin";
    let root = tempfile::tempdir().unwrap();
    let directory = root.path().join("package/.seseragi/native/id");
    let home = root.path().join("cargo-home");
    fs::create_dir_all(directory.join(".cargo")).unwrap();
    let cfg = cargo_config::parse_cfg(
        "debug_assertions\npanic=\"unwind\"\ntarget_arch=\"aarch64\"\ntarget_os=\"macos\"\nunix\n",
    );
    let resolve = |source: &str| {
        fs::write(directory.join(".cargo/config.toml"), source).unwrap();
        let settings =
            cargo_config::linker_settings(&directory, &home, host, Some(&cfg), &|_: &str| None);
        let invocation = linker_invocation(&settings.rustflags, settings.linker.as_deref());
        (settings, invocation)
    };

    // A matching cfg table supplies the linker Cargo uses.
    let (settings, invocation) =
        resolve("[target.'cfg(target_os = \"macos\")']\nlinker = \"rust-lld\"\n");
    assert_eq!(settings.linker.as_deref(), Some("rust-lld"));
    assert_eq!(invocation, LinkerInvocation::Direct);

    // The target triple wins over a matching cfg table, and a cfg table that
    // does not match is ignored.
    let (settings, invocation) = resolve(
        "[target.aarch64-apple-darwin]\nlinker = \"clang\"\n[target.'cfg(unix)']\nlinker = \"rust-lld\"\n",
    );
    assert_eq!(settings.linker.as_deref(), Some("clang"));
    assert_eq!(invocation, LinkerInvocation::CompilerDriver);
    let (settings, _) = resolve(
        "[target.'cfg(windows)']\nlinker = \"rust-lld\"\n[target.'cfg(not(unix))']\nlinker = \"ld\"\n",
    );
    assert_eq!(settings.linker, None);

    // Rustflags join the triple and every matching cfg table, and replace
    // build.rustflags.
    let (settings, invocation) = resolve(
        "[build]\nrustflags = [\"-Copt-level=3\"]\n\
         [target.aarch64-apple-darwin]\nrustflags = [\"-Copt-level=1\"]\n\
         [target.'cfg(all(unix, target_arch = \"aarch64\", not(target_os = \"linux\"),))']\n\
         rustflags = \"-C linker=rust-lld\"\n\
         [target.'cfg(any(windows, panic = \"abort\"))']\nrustflags = \"-C linker=clang\"\n",
    );
    assert_eq!(
        settings.rustflags,
        ["-Copt-level=1", "-C", "linker=rust-lld"]
    );
    assert_eq!(invocation, LinkerInvocation::Direct);

    // Malformed cfg expressions never match.
    for key in [
        "cfg(unix",
        "cfg(all(unix)",
        "cfg(not(unix, unix))",
        "cfg(target_os =)",
        "cfg()",
    ] {
        let (settings, _) = resolve(&format!("[target.'{key}']\nlinker = \"rust-lld\"\n"));
        assert_eq!(settings.linker, None, "{key}");
    }
}

#[test]
fn environment_flags_use_cargo_precedence() {
    let encoded = OsString::from("-C\x1flinker=rust-lld\x1f");
    let plain = OsString::from("-C  linker=clang");
    assert_eq!(
        cargo_config::environment_flags(Some(&encoded), Some(&plain)),
        ["-C", "linker=rust-lld"]
    );
    assert_eq!(
        cargo_config::environment_flags(None, Some(&plain)),
        ["-C", "linker=clang"]
    );
    assert!(cargo_config::environment_flags(None, None).is_empty());
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

/// The development runtime libraries built under a target root.
fn runtime_libraries(target: &Path) -> Vec<PathBuf> {
    let Ok(builds) = fs::read_dir(target.join("runtime")) else {
        return Vec::new();
    };
    let mut libraries = builds
        .flat_map(|build| {
            let deps = build
                .unwrap()
                .path()
                .join("target")
                .join(native_toolchain::host().unwrap())
                .join("debug/deps");
            fs::read_dir(deps)
                .into_iter()
                .flatten()
                .map(|entry| entry.unwrap().path())
        })
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
    libraries.sort();
    libraries
}

fn runtime_library(target: &Path) -> PathBuf {
    let libraries = runtime_libraries(target);
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
    let library = runtime_library(&project.target);
    let runtime = library.ancestors().nth(5).unwrap().to_owned();
    assert_eq!(
        runtime.parent(),
        Some(project.target.join("runtime").as_path())
    );
    assert!(!project.directory.join("runtime").exists());
    let files = [
        project.directory.join("Cargo.toml"),
        project.directory.join("Cargo.lock"),
        project.directory.join("build.rs"),
        project.directory.join("program.rs"),
        runtime.join("Cargo.toml"),
        runtime.join("lib.rs"),
    ];
    let times = files.each_ref().map(|file| modified(file));
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
    // A changed runtime gets its own build; the earlier one stays usable.
    let libraries = runtime_libraries(&project.target);
    assert_eq!(libraries.len(), 2, "{libraries:?}");
    assert!(libraries.contains(&library));
    assert_eq!(modified(&library), library_time);
    assert_eq!(modified(&files[5]), times[5]);
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
fn programs_under_one_target_root_share_one_runtime_build() {
    let fixture = Fixture::new();
    fs::write(fixture.root.path().join("other.ssrg"), "").unwrap();
    let first = fixture.project();
    let mut second = Project::open(&fixture.root.path().join("other.ssrg"), "main").unwrap();
    second.target = fixture.target.clone();
    assert_ne!(first.directory, second.directory);
    let other = fixture
        .root
        .path()
        .join(format!("other{}", std::env::consts::EXE_SUFFIX));
    first
        .build(
            BuildProfile::Development,
            SOURCE,
            RUNTIME,
            &fixture.executable(),
            &fixture.toolchain,
        )
        .unwrap();
    let library = runtime_library(&fixture.target);
    let library_time = modified(&library);
    second
        .build(
            BuildProfile::Development,
            &SOURCE.replace("{}", "other {}"),
            RUNTIME,
            &other,
            &fixture.toolchain,
        )
        .unwrap();
    assert_eq!(runtime_libraries(&fixture.target), [library.clone()]);
    assert_eq!(modified(&library), library_time);
    assert_ne!(
        first
            .target_for(BuildProfile::Development, RUNTIME)
            .unwrap(),
        second
            .target_for(BuildProfile::Development, RUNTIME)
            .unwrap()
    );
    assert_eq!(fixture.stdout(), "42\n");
    let output = run_published(&other);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"other 42\n");
    // The release runtime is a separate build.
    second
        .build(
            BuildProfile::Release,
            SOURCE,
            RUNTIME,
            &other,
            &fixture.toolchain,
        )
        .unwrap();
    assert_eq!(
        fs::read_dir(fixture.target.join("runtime"))
            .unwrap()
            .count(),
        2
    );
    assert_eq!(modified(&library), library_time);
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
    let runtime = runtime_library(&project.target);
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
    project
        .build(
            BuildProfile::Development,
            SOURCE,
            RUNTIME,
            &fixture.executable(),
            &fixture.toolchain,
        )
        .unwrap();
    let runtime = runtime_library(&project.target)
        .ancestors()
        .nth(5)
        .unwrap()
        .join("lib.rs");
    fs::remove_file(&runtime).unwrap();
    symlink(&victim, &runtime).unwrap();
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
