#[path = "../../test-support/native_runtime.rs"]
mod link;
#[path = "support/generic_impl_process.rs"]
mod process;

use process::{bounded, runtime, successful};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

fn cli() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_seseragi"));
    link::configure(&mut command);
    command
}

struct Program {
    name: String,
    executable: PathBuf,
    argument: Option<PathBuf>,
}

impl Program {
    fn run(&self, cwd: &Path, input: &Path, logs: &Path) -> Output {
        let mut command = Command::new(&self.executable);
        if let Some(argument) = &self.argument {
            command.arg(argument);
        }
        let output = bounded(
            command
                .current_dir(cwd)
                .stdin(Stdio::from(File::open(input).unwrap()))
                .env("PATH", "")
                .env("SESERAGI_NATIVE_METRICS", "1"),
            logs,
        );
        if self.argument.is_none() {
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("managed-live=0"),
                "{}: {output:?}",
                self.name
            );
        }
        successful(output)
    }
}

fn native_program(source: &Path, scratch: &Path) -> Program {
    let native = scratch.join("native");
    successful(bounded(
        cli()
            .args(["experimental", "build-native"])
            .arg(source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        scratch,
    ));
    Program {
        name: "native".into(),
        executable: native.join(format!("program{}", std::env::consts::EXE_SUFFIX)),
        argument: None,
    }
}

fn programs(source: &Path, scratch: &Path) -> Vec<Program> {
    let input = process::copy_input(source, scratch);
    let source = input.as_path();
    let mut programs = vec![native_program(source, scratch)];
    for profile in ["development", "release"] {
        let output = scratch.join(profile);
        successful(bounded(
            cli()
                .arg("build")
                .arg(source)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&output),
            scratch,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("artifact-manifest.json")).unwrap())
                .unwrap();
        for host in ["bun", "node"] {
            programs.push(Program {
                name: format!("{profile}-{host}"),
                executable: runtime(host, scratch),
                argument: Some(output.join(manifest["entry"].as_str().unwrap())),
            });
        }
    }
    programs
}

fn metadata(kind: &str, size: u64) -> String {
    let description = format!(
        "FileMetadata {{ fileType: {kind}, sizeBytes: {size}, modified: Just, created: Just }}"
    );
    let label = match kind {
        "RegularFile" => "file",
        "Directory" => "directory",
        "SymbolicLink" => "link",
        "OtherFileType" => "other",
        _ => unreachable!(),
    };
    format!(
        "Right {description} | {description} | {label} | {kind} | {kind} | {size} | True | True"
    )
}

fn error(operation: &str, path: &str, kind: &str) -> String {
    let path = serde_json::to_string(path).unwrap();
    format!(
        "Left FileSystemError {{ operation: {operation}, path: {path}, otherPath: Nothing, kind: {kind} }}"
    )
}

fn canonical(path: &Path) -> String {
    let path = fs::canonicalize(path).unwrap();
    let path = path.to_str().unwrap();
    #[cfg(windows)]
    let path = path.strip_prefix(r"\\?\").unwrap_or(path);
    format!("Right {}", path.replace('\\', "/"))
}

struct Case {
    path: String,
    expected: String,
    node_expected: Option<String>,
}

impl Case {
    fn new(path: &str, found: &str, metadata: &str, link: &str, canonical: &str) -> Self {
        Self {
            path: path.into(),
            expected: format!(
                "exists {found}\nmetadata {metadata}\nsymlinkMetadata {link}\ncanonicalize {canonical}\n"
            ),
            node_expected: None,
        }
    }

    fn failure(path: &str, kind: &str, missing: bool) -> Self {
        Self::new(
            path,
            &if missing {
                "Right False".into()
            } else {
                error("ReadMetadata", path, kind)
            },
            &error("ReadMetadata", path, kind),
            &error("ReadMetadata", path, kind),
            &error("CanonicalizePath", path, kind),
        )
    }
}

#[test]
fn filesystem_queries_match_ts_for_files_links_errors_and_lexical_boundaries() {
    let scratch = tempfile::tempdir().unwrap();
    let scratch = scratch.path();
    let data = scratch.join("data");
    fs::create_dir(&data).unwrap();
    fs::create_dir_all(data.join("nested/inside")).unwrap();
    fs::write(data.join("file.txt"), "初期値🙂\n").unwrap();
    fs::write(data.join("empty.txt"), "").unwrap();
    fs::write(data.join("雪🙂.txt"), "hello").unwrap();
    fs::write(data.join("nested/only.txt"), "nested").unwrap();
    let file = metadata("RegularFile", "初期値🙂\n".len() as u64);
    let empty = metadata("RegularFile", 0);
    let unicode = metadata("RegularFile", 5);
    let dir = metadata(
        "Directory",
        fs::metadata(data.join("nested")).unwrap().len(),
    );
    let current = metadata("Directory", fs::metadata(&data).unwrap().len());
    let mut cases = vec![];
    for (path, value, target) in [
        ("file.txt", &file, data.join("file.txt")),
        ("./file.txt", &file, data.join("file.txt")),
        ("nested/../file.txt", &file, data.join("file.txt")),
        ("empty.txt", &empty, data.join("empty.txt")),
        ("雪🙂.txt", &unicode, data.join("雪🙂.txt")),
        ("nested/", &dir, data.join("nested")),
        (".", &current, data.clone()),
    ] {
        cases.push(Case::new(
            path,
            "Right True",
            value,
            value,
            &canonical(&target),
        ));
    }
    cases.push(Case::new(
        &data.join("file.txt").to_str().unwrap().replace('\\', "/"),
        "Right True",
        &file,
        &file,
        &canonical(&data.join("file.txt")),
    ));
    for path in ["missing", "missing/child"] {
        cases.push(Case::failure(path, "FileNotFound", true));
    }
    cases.push(Case::failure("file.txt/child", "NotADirectory", false));
    cases.push(Case::failure(&"a".repeat(300), "PathNotSupported", false));
    cases.push(Case::new(
        "missing/../file.txt",
        "Right False",
        &file,
        &file,
        &error("CanonicalizePath", "missing/../file.txt", "FileNotFound"),
    ));
    cases.push(Case::new(
        "file.txt/../missing",
        &error("ReadMetadata", "file.txt/../missing", "NotADirectory"),
        &error("ReadMetadata", "file.txt/../missing", "FileNotFound"),
        &error("ReadMetadata", "file.txt/../missing", "FileNotFound"),
        &error("CanonicalizePath", "file.txt/../missing", "NotADirectory"),
    ));
    cases.push(Case::new(
        "file.txt/",
        &error("ReadMetadata", "file.txt/", "NotADirectory"),
        &file,
        &file,
        &error("CanonicalizePath", "file.txt/", "NotADirectory"),
    ));

    #[cfg(unix)]
    {
        use std::os::unix::{ffi::OsStrExt, fs::symlink};
        for (target, name) in [
            ("file.txt", "file-link"),
            ("nested/inside", "dir-link"),
            ("missing", "dangling"),
            ("loop", "loop"),
            ("file-link", "chain"),
        ] {
            symlink(target, data.join(name)).unwrap();
        }
        for name in ["file-link", "chain"] {
            cases.push(Case::new(
                name,
                "Right True",
                &file,
                &metadata(
                    "SymbolicLink",
                    fs::symlink_metadata(data.join(name)).unwrap().len(),
                ),
                &canonical(&data.join("file.txt")),
            ));
        }
        cases.push(Case::new(
            "dir-link",
            "Right True",
            &metadata(
                "Directory",
                fs::metadata(data.join("nested/inside")).unwrap().len(),
            ),
            &metadata(
                "SymbolicLink",
                fs::symlink_metadata(data.join("dir-link")).unwrap().len(),
            ),
            &canonical(&data.join("nested/inside")),
        ));
        for (name, kind, found) in [
            ("dangling", "FileNotFound", "Right False".to_string()),
            (
                "loop",
                "SymbolicLinkLoop",
                error("ReadMetadata", "loop", "SymbolicLinkLoop"),
            ),
        ] {
            cases.push(Case::new(
                name,
                &found,
                &error("ReadMetadata", name, kind),
                &metadata(
                    "SymbolicLink",
                    fs::symlink_metadata(data.join(name)).unwrap().len(),
                ),
                &error("CanonicalizePath", name, kind),
            ));
        }
        for (path, exists_kind, canonical_kind) in [
            ("loop/../file.txt", "SymbolicLinkLoop", "SymbolicLinkLoop"),
            ("file-link/", "NotADirectory", "NotADirectory"),
        ] {
            let link = if path == "file-link/" {
                metadata(
                    "SymbolicLink",
                    fs::symlink_metadata(data.join("file-link")).unwrap().len(),
                )
            } else {
                file.clone()
            };
            cases.push(Case::new(
                path,
                &error("ReadMetadata", path, exists_kind),
                &file,
                &link,
                &error("CanonicalizePath", path, canonical_kind),
            ));
        }
        cases.push(Case::new(
            "dir-link/../only.txt",
            "Right True",
            &error("ReadMetadata", "dir-link/../only.txt", "FileNotFound"),
            &error("ReadMetadata", "dir-link/../only.txt", "FileNotFound"),
            &canonical(&data.join("nested/only.txt")),
        ));
        unsafe extern "C" {
            fn mkfifo(path: *const std::ffi::c_char, mode: u32) -> i32;
        }
        let fifo = std::ffi::CString::new(data.join("fifo").as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { mkfifo(fifo.as_ptr(), 0o600) }, 0);
        let other = metadata(
            "OtherFileType",
            fs::metadata(data.join("fifo")).unwrap().len(),
        );
        cases.push(Case::new(
            "fifo",
            "Right True",
            &other,
            &other,
            &canonical(&data.join("fifo")),
        ));
    }

    #[cfg(unix)]
    let _permissions = {
        use std::os::unix::fs::PermissionsExt;
        struct Restore(PathBuf);
        impl Drop for Restore {
            fn drop(&mut self) {
                fs::set_permissions(&self.0, fs::Permissions::from_mode(0o700)).unwrap();
            }
        }
        let restricted = data.join("restricted");
        fs::create_dir(&restricted).unwrap();
        fs::write(restricted.join("file.txt"), "hidden").unwrap();
        let restore = Restore(restricted.clone());
        fs::set_permissions(&restricted, fs::Permissions::from_mode(0)).unwrap();
        if fs::metadata(restricted.join("file.txt"))
            .is_err_and(|e| e.kind() == std::io::ErrorKind::PermissionDenied)
        {
            cases.push(Case::failure(
                "restricted/file.txt",
                "PermissionDenied",
                false,
            ));
        } else {
            eprintln!("permission denial requires a host user without permission overrides");
        }
        restore
    };

    // Node uses macOS realpath's permissive handling of trailing slash and file/...
    #[cfg(target_os = "macos")]
    for case in &mut cases {
        let canonical = match case.path.as_str() {
            "file.txt/../missing" => error("CanonicalizePath", &case.path, "FileNotFound"),
            "file.txt/" | "file-link/" => canonical(&data.join("file.txt")),
            _ => continue,
        };
        let prefix = case.expected.rsplit_once("canonicalize ").unwrap().0;
        case.node_expected = Some(format!("{prefix}canonicalize {canonical}\n"));
    }

    // Directory size can change while the host fixtures are being created.
    let current = metadata("Directory", fs::metadata(&data).unwrap().len());
    *cases.iter_mut().find(|case| case.path == ".").unwrap() =
        Case::new(".", "Right True", &current, &current, &canonical(&data));
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/rust-backend/filesystem-metadata.ssrg");
    let programs = programs(&source, scratch);
    let input = scratch.join("input");
    for case in cases {
        fs::write(&input, format!("{}\n", case.path)).unwrap();
        let mut first = None;
        for program in &programs {
            let output = program.run(&data, &input, scratch);
            let actual = String::from_utf8(output.stdout).unwrap();
            let expected = if program.name.ends_with("-node") {
                case.node_expected.as_ref().unwrap_or(&case.expected)
            } else {
                &case.expected
            };
            assert_eq!(&actual, expected, "{} / {}", program.name, case.path);
            if expected != &case.expected {
                continue;
            }
            if let Some(first) = &first {
                assert_eq!(&actual, first, "{} / {}", program.name, case.path);
            } else {
                first = Some(actual);
            }
        }
    }
}

#[test]
fn filesystem_metadata_instants_match_the_ts_runtime_at_epoch_and_precision_boundaries() {
    use std::time::{Duration, UNIX_EPOCH};
    let scratch = tempfile::tempdir().unwrap();
    let scratch = scratch.path();
    seseragi_runtime::stage_typescript_package(scratch).unwrap();
    let typescript = scratch.join("instants.ts");
    fs::write(&typescript, r#"
import { metadata, symlinkMetadata } from "./node_modules/@seseragi/runtime/src/filesystem.ts"
import { instantNanoseconds } from "./node_modules/@seseragi/runtime/src/clock-value.ts"
import { createEffectExecution } from "./node_modules/@seseragi/runtime/src/effect.ts"
import { parse } from "./node_modules/@seseragi/runtime/src/path.ts"
import { createProviderFileSystem } from "./node_modules/@seseragi/runtime/src/provider-filesystem.ts"
import { createFileSystemProvider } from "./node_modules/seseragi/filesystem.ts"
const provider = `seseragi/runtime-${process.versions.bun ? "bun" : "node"}#filesystem`
const target = process.versions.bun ? "bun-process" : "node-process"
const environment = { fileSystem: createProviderFileSystem({
  provider, service: "std/fs::FileSystem", entry: createFileSystemProvider(provider, target)
}) }
const execution = createEffectExecution()
const path = parse(process.argv.at(-1))
if (path.tag !== "Right") throw new Error("invalid test path")
try {
  for (const [name, operation] of [["std/fs::metadata", metadata], ["std/fs::symlinkMetadata", symlinkMetadata]]) {
    try {
      const value = await operation(path.value)(environment, execution.context)
      for (const field of ["modified", "created"]) {
        const instant = value[field]
        console.log(name, field, instant.tag === "Just" ? String(instantNanoseconds(instant.value)) : "Nothing")
      }
    } catch (error) {
      if (error.name !== "ProviderBoundaryDefect" || error.message !== "filesystem metadata is invalid") throw error
      console.log(name, "Defect", error.message)
    }
  }
} finally {
  await execution.close()
}

"#).unwrap();
    let javascript = scratch.join("instants.mjs");
    successful(bounded(
        Command::new(runtime("bun", scratch))
            .arg("build")
            .arg(&typescript)
            .args(["--target", "node", "--outfile"])
            .arg(&javascript),
        scratch,
    ));
    let rust = scratch.join("instants.rs");
    link::write_program(
        &rust,
        r#"
use seseragi_native_runtime::*;
fn main() {
    let path = std::env::args().nth(1).unwrap();
    for name in ["std/fs::metadata", "std/fs::symlinkMetadata"] {
        let plan = native_filesystem_operation(name, vec![native_path(&path)]);
        let value = match NativeContext::default().root(plan) {
            Ok(value) => value,
            Err(NativeExit::Defect(errors)) => {
                assert_eq!(errors, ["filesystem metadata is invalid"]);
                println!("{name} Defect filesystem metadata is invalid");
                continue;
            }
            Err(_) => panic!("metadata returned a typed failure or cancellation"),
        };
        for field in ["modified", "created"] {
            let instant = value.field(field);
            if instant.tagged("std/prelude::Nothing") {
                println!("{name} {field} Nothing");
            } else {
                assert!(instant.tagged("std/prelude::Just"));
                let instant = instant.payload();
                assert!(instant.tagged("std/time::Instant"));
                println!("{name} {field} {}", instant.payload().render());
            }
        }
    }
    clear_dictionaries();
    assert_eq!(LIVE.with(std::cell::Cell::get), 0);
}
"#,
    )
    .unwrap();
    let binary = scratch.join(format!("instants{}", std::env::consts::EXE_SUFFIX));
    successful(bounded(
        Command::new("rustc")
            .args(["--edition=2021", "-o"])
            .arg(&binary)
            .arg(&rust),
        scratch,
    ));
    let node = runtime("node", scratch);
    let bun = runtime("bun", scratch);
    for (index, time) in [
        UNIX_EPOCH,
        UNIX_EPOCH - Duration::from_nanos(1),
        UNIX_EPOCH - Duration::new(1_234_567, 123_456_789),
        UNIX_EPOCH + Duration::new(1_793_586_967, 539_172_942),
        UNIX_EPOCH + Duration::new(10_000_000_000, 999_999_999),
    ]
    .into_iter()
    .enumerate()
    {
        let path = scratch.join(format!("time-{index}"));
        let file = File::create(&path).unwrap();
        file.set_times(fs::FileTimes::new().set_modified(time))
            .unwrap();
        drop(file);
        let mut paths = vec![path.clone()];
        #[cfg(unix)]
        {
            let link = scratch.join(format!("link-{index}"));
            std::os::unix::fs::symlink(&path, &link).unwrap();
            paths.push(link);
        }
        for path in paths {
            let native = successful(bounded(Command::new(&binary).arg(&path), scratch));
            if index == 2 && path.file_name().unwrap() == "time-2" {
                assert_eq!(
                    native.stdout,
                    b"std/fs::metadata Defect filesystem metadata is invalid\nstd/fs::symlinkMetadata Defect filesystem metadata is invalid\n"
                );
            }
            for host in [&bun, &node] {
                let ts = successful(bounded(
                    Command::new(host).arg(&javascript).arg(&path),
                    scratch,
                ));
                assert_eq!(
                    native.stdout,
                    ts.stdout,
                    "{} / {}",
                    host.display(),
                    path.display()
                );
            }
        }
    }
}

#[test]
fn filesystem_query_effects_are_cold_and_observe_changes_when_repeated() {
    let scratch = tempfile::tempdir().unwrap();
    let scratch = scratch.path();
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/rust-backend/filesystem-metadata-effects.ssrg");
    let input = scratch.join("input");
    fs::write(&input, "").unwrap();
    let description = |size| {
        format!(
        "FileMetadata {{ fileType: RegularFile, sizeBytes: {size}, modified: Just, created: Just }}"
    )
    };
    let missing = error("ReadMetadata", "changing.txt", "FileNotFound");
    let missing_path = error("CanonicalizePath", "changing.txt", "FileNotFound");
    let expected = format!(
        "False\n{missing}\n{missing}\n{}\nTrue\n{}\n{}\nTrue\nJust changing.txt\n{}\n{}\n",
        missing_path.strip_prefix("Left ").unwrap(),
        description(1),
        description(1),
        description(3),
        description(3),
    );
    for program in programs(&source, scratch) {
        let cwd = scratch.join(format!("run-{}", program.name));
        fs::create_dir(&cwd).unwrap();
        let result = program.run(&cwd, &input, scratch);
        assert_eq!(
            String::from_utf8(result.stdout).unwrap(),
            expected,
            "{}",
            program.name
        );
        assert_eq!(fs::read(cwd.join("changing.txt")).unwrap(), b"abc");
        assert!(!cwd.join("unused.txt").exists());
    }
}

#[test]
fn native_filesystem_queries_support_saved_generic_and_returned_functions() {
    let scratch = tempfile::tempdir().unwrap();
    let scratch = scratch.path();
    let original =
        include_str!("../../../examples/spec/fixtures/rust-backend/filesystem-metadata.ssrg");
    let saved = original
        .replace("pub effect fn main", "fn keep<A> value: A -> A = value\nfn returned -> (paths.Path -> Effect<{fileSystem: fs.FileSystem}, fs.FileSystemError, fs.FileMetadata>) = fs.metadata\npub effect fn main")
        .replace("found <- fs.exists path", "let saved = keep fs.exists\n  found <- saved path")
        .replace("metadata <- fs.metadata path", "let savedMetadata = returned ()\n  metadata <- savedMetadata path")
        .replace("link <- fs.symlinkMetadata path", "let savedLink = keep fs.symlinkMetadata\n  link <- savedLink path")
        .replace("resolved <- fs.canonicalize path", "let savedCanonical = keep fs.canonicalize\n  resolved <- savedCanonical path");
    let source = scratch.join("saved.ssrg");
    fs::write(&source, saved).unwrap();
    let program = native_program(&source, scratch);
    let input = scratch.join("input");
    let data = scratch.join("data");
    fs::create_dir(&data).unwrap();
    fs::write(data.join("file.txt"), "hello").unwrap();
    let value = metadata("RegularFile", 5);
    for case in [
        Case::new(
            "file.txt",
            "Right True",
            &value,
            &value,
            &canonical(&data.join("file.txt")),
        ),
        Case::failure("missing", "FileNotFound", true),
    ] {
        fs::write(&input, format!("{}\n", case.path)).unwrap();
        let result = program.run(&data, &input, scratch);
        assert_eq!(
            String::from_utf8(result.stdout).unwrap(),
            case.expected,
            "{}",
            case.path
        );
    }
}
