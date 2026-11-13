use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

const UNFORMATTED: &str = "let  answer =   42\n";
const FORMATTED: &str = "let answer = 42\n";

fn seseragi(directory: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(arguments)
        .current_dir(directory)
        .output()
        .unwrap()
}

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

#[test]
fn formats_files_directories_and_the_current_package() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write(
        &root.join("pkg/seseragi.toml"),
        "[package]\nname = \"fixture/pkg\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n",
    );
    write(&root.join("pkg/.gitignore"), "generated/\n*.skip.ssrg\n");
    for file in [
        "pkg/src/main.ssrg",
        "pkg/src/nested/util.ssrg",
        "pkg/tests/main.ssrg",
        "pkg/generated/out.ssrg",
        "pkg/src/old.skip.ssrg",
        "pkg/dist/out.ssrg",
        "pkg/node_modules/dep/x.ssrg",
        "pkg/.seseragi/cache.ssrg",
        "loose/a.ssrg",
        "loose/b.ssrg",
    ] {
        write(&root.join(file), UNFORMATTED);
    }
    write(&root.join("pkg/src/broken.ssrg"), "let = \n");
    write(&root.join("outside/elsewhere.ssrg"), UNFORMATTED);
    #[cfg(unix)]
    std::os::unix::fs::symlink(root.join("outside"), root.join("pkg/src/linked")).unwrap();

    // --check lists every file that needs formatting, keeps going past a
    // syntax error, and changes nothing.
    let checked = seseragi(root, &["format", "--check", "pkg"]);
    assert_eq!(checked.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&checked.stderr);
    for file in [
        "pkg/src/main.ssrg",
        "pkg/src/nested/util.ssrg",
        "pkg/tests/main.ssrg",
    ] {
        assert!(
            stderr.contains(&format!("{file} is not canonically formatted")),
            "{file}: {stderr}"
        );
    }
    assert!(stderr.contains("broken.ssrg"), "{stderr}");
    for skipped in ["generated", "old.skip", "dist", "node_modules", ".seseragi"] {
        assert!(!stderr.contains(skipped), "{skipped}: {stderr}");
    }
    assert_eq!(
        fs::read_to_string(root.join("pkg/src/main.ssrg")).unwrap(),
        UNFORMATTED
    );

    // Without a path, format works on the package containing the directory.
    let formatted = seseragi(&root.join("pkg/src/nested"), &["format"]);
    assert_eq!(
        formatted.status.code(),
        Some(2),
        "the broken file still fails"
    );
    for file in [
        "pkg/src/main.ssrg",
        "pkg/src/nested/util.ssrg",
        "pkg/tests/main.ssrg",
    ] {
        assert_eq!(
            fs::read_to_string(root.join(file)).unwrap(),
            FORMATTED,
            "{file}"
        );
    }
    for file in [
        "pkg/generated/out.ssrg",
        "pkg/dist/out.ssrg",
        "pkg/src/old.skip.ssrg",
    ] {
        assert_eq!(
            fs::read_to_string(root.join(file)).unwrap(),
            UNFORMATTED,
            "{file}"
        );
    }
    assert_eq!(
        fs::read_to_string(root.join("pkg/src/broken.ssrg")).unwrap(),
        "let = \n"
    );
    assert_eq!(
        fs::read_to_string(root.join("outside/elsewhere.ssrg")).unwrap(),
        UNFORMATTED,
        "links out of the package are not followed"
    );

    // Several explicit files.
    let checked = seseragi(root, &["format", "--check", "loose/a.ssrg", "loose/b.ssrg"]);
    assert_eq!(checked.status.code(), Some(1));
    let formatted = seseragi(root, &["format", "loose/a.ssrg", "loose/b.ssrg"]);
    assert_eq!(formatted.status.code(), Some(0));
    let checked = seseragi(root, &["format", "--check", "loose"]);
    assert_eq!(
        checked.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let outside = seseragi(root, &["format"]);
    assert_eq!(outside.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&outside.stderr).contains("not inside a package"));
}

#[test]
fn formats_standard_input_to_standard_output() {
    let directory = tempfile::tempdir().unwrap();
    let format = |arguments: &[&str], input: &str| {
        let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(arguments)
            .current_dir(directory.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    };
    let output = format(&["format", "-"], UNFORMATTED);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), FORMATTED);

    let output = format(&["format", "--check", "-"], UNFORMATTED);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());

    let output = format(&["format", "-"], "let = \n");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}
