use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[allow(dead_code)]
#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, successful};

const SINGLE: &str =
    "fn boom n: Int -> Int = n / 0\npub effect fn main = println (show (boom 3))\n";
const EXPECTED: &str = "main.ssrg:1:25: Seseragi Defect: Seseragi Int division by zero\n";

fn build(source: &Path, target: &str, profile: &str, out: &Path, cwd: &Path) {
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("build")
            .arg(source)
            .args(["--target", target, "--profile", profile, "--out-dir"])
            .arg(out),
        cwd,
    ));
}

fn javascript(directory: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(root: &Path, current: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(current).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(root, &path, files);
            } else if path.extension().is_some_and(|extension| extension == "js") {
                files.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    collect(directory, directory, &mut files);
    assert!(!files.is_empty());
    files
}

fn no_directory(bytes: &[u8], directory: &Path) {
    let native = directory.to_string_lossy();
    let portable = native.replace('\\', "/");
    for path in [native.as_ref(), portable.as_str()] {
        assert!(
            !bytes
                .windows(path.len())
                .any(|window| window == path.as_bytes()),
            "embedded absolute directory: {path}"
        );
    }
}

fn assert_defect(command: &mut Command, cwd: &Path, expected: &str) {
    let output = bounded(command, cwd);
    assert_eq!(output.status.code(), Some(70), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(output.stderr, expected.as_bytes());
}

fn assert_process(out: &Path, cwd: &Path, expected: &str) {
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(out.join("artifact-manifest.json")).unwrap()).unwrap();
    for engine in ["bun", "node"] {
        assert_defect(
            Command::new(engine)
                .arg(out.join(manifest["entry"].as_str().unwrap()))
                .current_dir(cwd),
            cwd,
            expected,
        );
    }
}

#[test]
fn standalone_bundles_are_independent_of_absolute_checkout_paths() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let roots = [
        fs::canonicalize(first.path()).unwrap(),
        fs::canonicalize(second.path()).unwrap(),
    ];
    for root in &roots {
        fs::create_dir(root.join("nested")).unwrap();
        fs::write(root.join("nested/main.ssrg"), SINGLE).unwrap();
    }
    for profile in ["development", "release"] {
        let mut bundles = Vec::new();
        for root in &roots {
            let source = root.join("nested/main.ssrg");
            let out = root.join(profile);
            build(&source, "process", profile, &out, root);
            let files = javascript(&out);
            for bytes in files.values() {
                no_directory(bytes, root);
            }
            assert_process(&out, root, EXPECTED);
            assert_defect(
                Command::new(env!("CARGO_BIN_EXE_seseragi"))
                    .arg("run")
                    .arg(&source)
                    .args(["--profile", profile]),
                root,
                EXPECTED,
            );
            bundles.push(files);
        }
        assert_eq!(bundles[0], bundles[1]);
    }
}

#[test]
fn package_and_native_outputs_use_the_package_root_with_custom_source_layouts() {
    let scratch = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(scratch.path()).unwrap();
    fs::create_dir_all(root.join("code/errors")).unwrap();
    fs::write(root.join("seseragi.toml"), "[package]\nname = \"fixture/runtime-paths\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[layout]\nsource = \"code\"\n[run]\nentry = \"main\"\ntarget = \"process\"\n").unwrap();
    fs::write(
        root.join("code/main.ssrg"),
        "import { boom } from \"./errors/boom\"\npub effect fn main = println (show (boom 3))\n",
    )
    .unwrap();
    fs::write(
        root.join("code/errors/boom.ssrg"),
        "pub fn boom n: Int -> Int = n / 0\n",
    )
    .unwrap();
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&root),
        &root,
    ));
    let expected = "code/errors/boom.ssrg:1:29: Seseragi Defect: Seseragi Int division by zero\n";
    for profile in ["development", "release"] {
        let process = root.join(format!("process-{profile}"));
        build(&root, "process", profile, &process, &root);
        for bytes in javascript(&process).values() {
            no_directory(bytes, &root);
        }
        assert_process(&process, &root, expected);
        let native = root.join(format!("native-{profile}"));
        build(&root, "native", profile, &native, &root);
        let executable = native.join(format!("program{}", std::env::consts::EXE_SUFFIX));
        no_directory(&fs::read(native.join("program.rs")).unwrap(), &root);
        no_directory(&fs::read(&executable).unwrap(), &root);
        assert_defect(&mut Command::new(executable), &root, expected);
    }
    for target in ["process", "native"] {
        assert_defect(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["run", "--target", target])
                .arg(&root),
            &root,
            expected,
        );
    }
}
