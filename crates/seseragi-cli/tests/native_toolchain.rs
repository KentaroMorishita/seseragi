#![cfg(unix)]

#[path = "../../test-support/native_cache.rs"]
mod native_cache;
#[path = "support/generic_impl_process.rs"]
#[allow(dead_code)]
mod process;

use process::{bounded, successful};
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::{symlink, PermissionsExt},
    path::Path,
    process::Command,
};

fn script(path: &Path, text: &str) {
    fs::write(path, text).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn first_use_without_rust_on_path_and_offline_rebuild_match_ts_results_and_errors() {
    let scratch = tempfile::Builder::new()
        .prefix("native toolchain 雪 ")
        .tempdir()
        .unwrap();
    let root = scratch.path();
    let tools = root.join("tools");
    fs::create_dir(&tools).unwrap();
    for name in ["cc", "ld", "ar", "as", "xcrun", "clang"] {
        let program = Path::new("/usr/bin").join(name);
        if program.exists() {
            symlink(program, tools.join(name)).unwrap();
        }
    }
    for name in ["rustc", "cargo", "rustup"] {
        script(
            &tools.join(name),
            "#!/bin/sh\nprintf 'unexpected PATH Rust invocation\\n' >&2\nexit 99\n",
        );
    }
    let version: toml::Value = include_str!("../../../rust-toolchain.toml")
        .parse()
        .unwrap();
    let version = version["toolchain"]["channel"].as_str().unwrap();
    let host = seseragi_release::BUILD_TARGET;
    let payload = format!("#!/bin/sh\nset -eu\nprintf '%s\\n' \"$@\" > \"$SESERAGI_TEST_INIT_LOG\"\n/bin/mkdir -p \"$RUSTUP_HOME/toolchains\"\n/bin/ln -s \"$SESERAGI_TEST_SYSROOT\" \"$RUSTUP_HOME/toolchains/{version}-{host}\"\n");
    let installer = root.join("installer");
    fs::write(&installer, &payload).unwrap();
    let checksum = root.join("checksum");
    fs::write(
        &checksum,
        format!("{:x}\n", Sha256::digest(payload.as_bytes())),
    )
    .unwrap();
    script(
        &tools.join("curl"),
        r#"#!/bin/sh
set -eu
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--output" ]; then shift; destination="$1"; fi
  url="$1"
  shift
done
printf '%s\n' "$url" >> "$SESERAGI_TEST_DOWNLOAD_LOG"
case "$url" in
  *.sha256) /bin/cp "$SESERAGI_TEST_CHECKSUM" "$destination" ;;
  *) /bin/cp "$SESERAGI_TEST_INSTALLER" "$destination" ;;
esac
"#,
    );
    let cache = root.join("cache");
    let source = root.join("main.ssrg");
    fs::write(&source, "pub effect fn main = do { println \"\"; println \"雪🦀\"; println (show (-9007199254740991)); println (show 9007199254740991) }\n").unwrap();
    let mut ts = Command::new(env!("CARGO_BIN_EXE_seseragi"));
    let ts = successful(bounded(ts.arg("run").arg(&source), root));
    assert_eq!(
        ts.stdout,
        "\n雪🦀\n-9007199254740991\n9007199254740991\n".as_bytes()
    );
    let build = |profile: &str| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_seseragi"));
        bounded(
            command
                .args(["experimental", "build-native"])
                .arg(&source)
                .args(["--entry", "main", "--profile", profile, "--out-dir"])
                .arg(root.join("native"))
                .env("SESERAGI_NATIVE_CACHE", &cache)
                .env_remove("SESERAGI_NATIVE_TOOLCHAIN")
                .env("PATH", &tools)
                .env("CARGO_HOME", root.join("unused-user-cargo"))
                .env("RUSTUP_HOME", root.join("unused-user-rustup"))
                .env("RUSTUP_TOOLCHAIN", "missing-user-toolchain")
                .env("RUSTC", tools.join("rustc"))
                .env("RUSTC_WRAPPER", tools.join("rustc"))
                .env("RUSTC_WORKSPACE_WRAPPER", tools.join("rustc"))
                .env("CARGO_BUILD_TARGET", "wrong-user-target")
                .env("SESERAGI_TEST_INSTALLER", &installer)
                .env("SESERAGI_TEST_CHECKSUM", &checksum)
                .env("SESERAGI_TEST_SYSROOT", native_cache::toolchain())
                .env("SESERAGI_TEST_INIT_LOG", root.join("init-log"))
                .env("SESERAGI_TEST_DOWNLOAD_LOG", root.join("download-log")),
            root,
        )
    };
    successful(build("development"));
    let execute = || {
        bounded(
            Command::new(root.join("native/program")).env("PATH", ""),
            root,
        )
    };
    let native = successful(execute());
    assert_eq!(native.stdout, ts.stdout);
    assert_eq!(native.stderr, ts.stderr);
    assert_eq!(fs::read_to_string(root.join("init-log")).unwrap(), format!("-y\n--no-modify-path\n--profile\nminimal\n--default-host\n{host}\n--default-toolchain\n{version}\n"));
    let downloads = fs::read_to_string(root.join("download-log")).unwrap();
    assert_eq!(downloads.lines().count(), 2);
    script(
        &tools.join("curl"),
        "#!/bin/sh\nprintf 'offline download attempted\\n' >&2\nexit 77\n",
    );
    fs::remove_file(&installer).unwrap();
    fs::remove_file(&checksum).unwrap();
    for profile in ["development", "release"] {
        fs::write(&source, "import * as effects from \"std/effect\"\npub effect fn main -> Unit with Console fails String = do { println \"changed offline\" |> effects.mapError (\\_: ConsoleError -> \"output\"); effects.fail \"stopped\" }\n").unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_seseragi"));
        let ts = bounded(
            command.arg("run").arg(&source).args(["--profile", profile]),
            root,
        );
        assert_eq!(ts.status.code(), Some(1), "{ts:?}");
        assert_eq!(ts.stdout, b"changed offline\n");
        successful(build(profile));
        let native = execute();
        assert_eq!(native.status.code(), ts.status.code(), "{native:?}");
        assert_eq!(native.stdout, ts.stdout);
        let message = String::from_utf8(native.stderr).unwrap();
        let location = format!("{}:2:", source.file_name().unwrap().to_string_lossy());
        let (column, message) = message
            .strip_prefix(&location)
            .unwrap()
            .split_once(": ")
            .unwrap();
        assert!(column.parse::<usize>().is_ok());
        assert_eq!(message.as_bytes(), ts.stderr);
    }
    assert_eq!(
        fs::read_to_string(root.join("download-log")).unwrap(),
        downloads
    );
    assert!(!root.join("unused-user-cargo").exists());
    assert!(!root.join("unused-user-rustup").exists());
    assert!(fs::read_dir(cache.join("toolchains"))
        .unwrap()
        .all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".install-")));
}

#[test]
fn failed_first_download_reports_repair_and_removes_partial_installation() {
    let root = tempfile::tempdir().unwrap();
    let tools = root.path().join("tools");
    fs::create_dir(&tools).unwrap();
    script(
        &tools.join("curl"),
        "#!/bin/sh\nprintf 'network unavailable\\n' >&2\nexit 7\n",
    );
    let source = root.path().join("main.ssrg");
    fs::write(&source, "pub fn main unit: Unit -> Int = 42\n").unwrap();
    let cache = root.path().join("cache");
    let mut command = Command::new(env!("CARGO_BIN_EXE_seseragi"));
    let output = bounded(
        command
            .args(["experimental", "build-native"])
            .arg(source)
            .args(["--entry", "main", "--out-dir"])
            .arg(root.path().join("native"))
            .env("SESERAGI_NATIVE_CACHE", &cache)
            .env_remove("SESERAGI_NATIVE_TOOLCHAIN")
            .env("PATH", tools),
        root.path(),
    );
    assert_eq!(output.status.code(), Some(2));
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("network unavailable"), "{diagnostic}");
    assert!(
        diagnostic.contains(seseragi_release::BUILD_TARGET),
        "{diagnostic}"
    );
    assert!(
        diagnostic.contains("connect to the internet and retry"),
        "{diagnostic}"
    );
    assert!(
        diagnostic.contains("SESERAGI_NATIVE_TOOLCHAIN"),
        "{diagnostic}"
    );
    let entries = fs::read_dir(cache.join("toolchains"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].extension().unwrap(), "lock");
    assert!(!root.path().join("native/program").exists());
}

#[test]
fn cargo_json_linker_diagnostics_reach_the_cli_and_keep_previous_outputs() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("main.ssrg");
    fs::write(&source, "pub fn main unit: Unit -> Int = 42\n").unwrap();
    let tools = root.path().join("tools");
    fs::create_dir(&tools).unwrap();
    let rendered = "error: linker `missing-linker` not found\n";
    let message = serde_json::json!({"reason":"compiler-message", "message":{"rendered":rendered}});
    script(
        &tools.join("cargo"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' '{message}'\nprintf 'Cargo build failed\\n' >&2\nexit 1\n"
        ),
    );
    let toolchain = native_cache::mock_toolchain(&tools);
    let destination = root.path().join("native");
    fs::create_dir(&destination).unwrap();
    for name in ["program", "program.rs", "rust-backend.json"] {
        fs::write(destination.join(name), format!("old {name}")).unwrap();
    }
    let mut command = native_cache::command(env!("CARGO_BIN_EXE_seseragi"));
    let output = bounded(
        command
            .args(["experimental", "build-native"])
            .arg(source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&destination)
            .env("SESERAGI_NATIVE_TOOLCHAIN", toolchain),
        root.path(),
    );
    assert_eq!(output.status.code(), Some(2));
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains(rendered), "{diagnostic}");
    assert!(diagnostic.contains("Cargo build failed"), "{diagnostic}");
    assert!(
        diagnostic.contains(if cfg!(target_os = "macos") {
            "xcode-select --install"
        } else {
            "build-essential"
        }),
        "{diagnostic}"
    );
    for name in ["program", "program.rs", "rust-backend.json"] {
        assert_eq!(
            fs::read_to_string(destination.join(name)).unwrap(),
            format!("old {name}")
        );
    }
}
