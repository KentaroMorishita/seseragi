use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_PROJECT: AtomicU64 = AtomicU64::new(0);

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

struct LockedProject {
    root: PathBuf,
    path: PathBuf,
}

impl LockedProject {
    fn doc_tests() -> Self {
        fn copy_directory(source: &Path, destination: &Path) {
            fs::create_dir_all(destination).unwrap();
            for entry in fs::read_dir(source).unwrap() {
                let entry = entry.unwrap();
                let from = entry.path();
                let to = destination.join(entry.file_name());
                if from.is_dir() {
                    copy_directory(&from, &to);
                } else {
                    fs::copy(from, to).unwrap();
                }
            }
        }

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sequence = NEXT_PROJECT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "seseragi-doc-tests-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        let path = root.join("project");
        copy_directory(
            &repository_root().join("examples/spec/fixtures/projects/doc-tests"),
            &path,
        );
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&path)
            .output()
            .unwrap();
        assert!(output.status.success());
        Self { root, path }
    }

    fn source(&self) -> PathBuf {
        self.path.join("src/math.ssrg")
    }

    fn run(&self) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("doc")
            .arg(&self.path)
            .arg("--test")
            .output()
            .unwrap()
    }

    fn update_lock(&self) {
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&self.path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

impl Drop for LockedProject {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

#[test]
fn runs_the_doc_tests_product_fixture_without_rewriting_source() {
    let project = LockedProject::doc_tests();
    let before = fs::read(project.source()).unwrap();
    let output = project.run();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        fs::read_to_string(project.path.join("expected.stdout")).unwrap()
    );
    assert_eq!(fs::read(project.source()).unwrap(), before);
}

const API_SOURCE: &str = concat!(
    "//! # Migration API 🙂\n",
    "//!\n",
    "//! - A list with [current].\n",
    "//!\n",
    "//! | Name | State |\n//! | --- | --- |\n//! | old | deprecated |\n",
    "//!\n//! <script>alert('module')</script>\n",
    "/// Prefer [current]. Private [secret] and absent [Missing] are unresolved.\n",
    "/// [unsafe](javascript:alert%281%29)\n",
    "pub deprecated \"Use <script>current</script>\" since \"0.61.0\" fn old value: Int -> Int = value\n",
    "pub fn current value: Int -> Int = value\n",
    "/// Private API.\n",
    "deprecated \"Use current\" fn secret value: Int -> Int = value\n",
    "pub trait Render<A> {\n",
    "  /// Render member.\n",
    "  deprecated \"Use render\" fn old value: A -> String\n",
    "}\n",
    "instance Render<Int> { fn old value: Int -> String = \"value\" }\n",
    "pub struct Box { pub value: Int }\n",
    "impl Box {\n",
    "  /// Inherent member.\n",
    "  pub deprecated \"Use read\" fn old self: Box -> Int = self.value\n",
    "}\n",
    "/// Detached.\n\npub fn separate -> Int = 1\n",
);

fn generate(project: &LockedProject, flags: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("doc")
        .arg(&project.path)
        .args(flags)
        .output()
        .unwrap()
}

fn document_bytes(root: &Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, path: &Path, result: &mut std::collections::BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                visit(root, &entry.path(), result);
            } else {
                result.insert(
                    entry.path().strip_prefix(root).unwrap().to_owned(),
                    fs::read(entry.path()).unwrap(),
                );
            }
        }
    }
    let mut result = std::collections::BTreeMap::new();
    visit(root, root, &mut result);
    result
}

#[test]
fn selected_operator_notices_reach_the_actual_doc_and_lint_commands() {
    let project = LockedProject::doc_tests();
    let source = "// 演算 🙂\npub struct Box { pub value: Int }\nimpl Box { deprecated \"Use combine\" since \"0.61.0\" operator + self -> other: Box -> Box = Box { value: self.value + other.value } }\npub fn combine left: Box -> right: Box -> Box = left + right\npub fn scalar -> Int = 1 + 2\n";
    fs::write(project.source(), source).unwrap();
    project.update_lock();
    let output = generate(&project, &[]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let html = fs::read_to_string(project.path.join("dist/doc/index.html")).unwrap();
    let index: serde_json::Value =
        serde_json::from_slice(&fs::read(project.path.join("dist/doc/search-index.json")).unwrap())
            .unwrap();
    let operators = index["symbols"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|symbol| symbol["namespace"] == "operator-implementation")
        .collect::<Vec<_>>();
    assert_eq!(operators.len(), 1);
    let operator = operators[0];
    assert_eq!(operator["deprecated"]["message"], "Use combine");
    assert_eq!(operator["deprecated"]["since"], "0.61.0");
    assert_eq!(operator["owner"], operator["metadata"]["instance"]);
    let notice = &operator["deprecationSource"]["origin"];
    assert_eq!(
        &source
            [notice["start"].as_u64().unwrap() as usize..notice["end"].as_u64().unwrap() as usize],
        "deprecated \"Use combine\" since \"0.61.0\""
    );
    assert!(html.contains("operator +"));
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("lint")
        .arg("--diagnostic-format")
        .arg("json")
        .arg("--deny-warnings")
        .arg(&project.path)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let wire: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    let facts = wire["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|document| document["diagnostics"]["diagnostics"].as_array().unwrap())
        .filter(|diagnostic| diagnostic["code"] == "SES-L0201")
        .collect::<Vec<_>>();
    assert_eq!(facts.len(), 1, "{facts:#?}");
    assert_eq!(
        facts[0]["primary"]["start"].as_u64().unwrap() as usize,
        source.find("left + right").unwrap() + 5
    );
}

#[test]
fn namespace_owner_notices_reach_real_docs_and_deny_warnings_without_infecting_children() {
    let project = LockedProject::doc_tests();
    let source = "// 元 🙂\npub foreign \"typescript\" from \"./host.mjs\" {\n /// Legacy API.\n deprecated \"Use current\" since \"0.61.0\" namespace legacy { pure fn read value: Int -> Int }\n namespace current { pure fn read value: Int -> Int }\n}\nforeign \"typescript\" from \"./host.mjs\" { deprecated \"Private only\" namespace internal { pure fn read value: Int -> Int } }\npub fn answer -> Int = legacy.read 41\n";
    fs::write(project.source(), source).unwrap();
    fs::write(project.path.join("src/host.mjs"), "export const legacy = { read: value => value }; export const current = legacy; export const internal = legacy;\n").unwrap();
    let manifest = project.path.join("seseragi.toml");
    fs::write(&manifest, format!("{}\n[foreign.typescript]\nresolver = \"node\"\nmanifest = \"host/package.json\"\nlockfile = \"host/bun.lock\"\n", fs::read_to_string(&manifest).unwrap())).unwrap();
    fs::create_dir_all(project.path.join("host")).unwrap();
    for name in ["package.json", "bun.lock"] {
        fs::copy(
            repository_root()
                .join("examples/spec/fixtures/projects/foreign-pure-load/host")
                .join(name),
            project.path.join("host").join(name),
        )
        .unwrap();
    }
    project.update_lock();
    let output = generate(&project, &[]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let html = fs::read_to_string(project.path.join("dist/doc/index.html")).unwrap();
    let index: serde_json::Value =
        serde_json::from_slice(&fs::read(project.path.join("dist/doc/search-index.json")).unwrap())
            .unwrap();
    let symbols = index["symbols"].as_array().unwrap();
    let namespace = symbols
        .iter()
        .find(|symbol| symbol["namespace"] == "namespace" && symbol["name"] == "legacy")
        .unwrap();
    assert_eq!(namespace["deprecated"]["message"], "Use current");
    assert_eq!(namespace["deprecated"]["since"], "0.61.0");
    let notice = &namespace["deprecationSource"]["origin"];
    assert_eq!(
        &source
            [notice["start"].as_u64().unwrap() as usize..notice["end"].as_u64().unwrap() as usize],
        "deprecated \"Use current\" since \"0.61.0\""
    );
    assert!(namespace["deprecationSource"]["source"]
        .as_str()
        .unwrap()
        .ends_with("math.ssrg"));
    assert!(symbols
        .iter()
        .filter(|symbol| symbol["name"]
            .as_str()
            .is_some_and(|name| name.ends_with("read")))
        .all(|symbol| symbol.get("deprecated").is_none()));
    assert!(!html.contains("Private only"));
    assert!(html.contains("namespace legacy"));
    let lint = |deny: bool| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_seseragi"));
        command.arg("lint");
        if deny {
            command.arg("--deny-warnings");
        }
        command.arg(&project.path).output().unwrap()
    };
    let ordinary = lint(false);
    assert!(
        ordinary.status.success(),
        "{}",
        String::from_utf8_lossy(&ordinary.stderr)
    );
    assert!(String::from_utf8_lossy(&ordinary.stderr).contains("SES-L0201"));
    let denied = lint(true);
    assert_eq!(denied.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&denied.stderr).contains("Use current"));
    let private = generate(&project, &["--include-private"]);
    assert!(
        private.status.success(),
        "{}",
        String::from_utf8_lossy(&private.stderr)
    );
    assert!(fs::read_to_string(project.path.join("dist/doc/index.html"))
        .unwrap()
        .contains("Private only"));
}

#[test]
fn publishes_deterministic_html_and_json_with_positioned_deprecation_and_safe_markdown() {
    let project = LockedProject::doc_tests();
    fs::write(project.source(), API_SOURCE).unwrap();
    project.update_lock();
    let first = generate(&project, &[]);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(first.stdout.is_empty());
    let output = project.path.join("dist/doc");
    let html = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(html.contains("<table>"));
    assert!(html.contains("<ul>"));
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
    assert!(!html.contains("javascript:"));
    assert!(html.contains("</aside>\n<pre class=\"signature\">"));
    let index: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("search-index.json")).unwrap()).unwrap();
    assert_eq!(index["schema"], 1);
    let symbols = index["symbols"].as_array().unwrap();
    assert!(!symbols.iter().any(|symbol| symbol["name"] == "secret"));
    for (name, message) in [
        ("old", "Use <script>current</script>"),
        ("Render.old", "Use render"),
        ("Box.old", "Use read"),
    ] {
        let symbol = symbols
            .iter()
            .find(|symbol| symbol["displayName"] == name)
            .unwrap();
        assert_eq!(symbol["deprecated"]["message"], message);
        if name == "old" {
            assert_eq!(symbol["deprecated"]["since"], "0.61.0");
        } else {
            assert!(symbol["deprecated"].get("since").is_none());
        }
        let range = &symbol["source"];
        let start = range["start"].as_u64().unwrap() as usize;
        let end = range["end"].as_u64().unwrap() as usize;
        assert!(API_SOURCE[start..end].contains("deprecated"));
        let href = range["href"].as_str().unwrap();
        assert!(output.join(href.split('#').next().unwrap()).is_file());
    }
    let current = symbols
        .iter()
        .find(|symbol| symbol["displayName"] == "current")
        .unwrap();
    assert!(current.get("deprecated").is_none());
    assert!(html.contains(&format!(
        "href=\"#{}\"",
        current["href"].as_str().unwrap().split('#').nth(1).unwrap()
    )));
    let unresolved = index["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|warning| warning["message"].as_str().unwrap().contains("code link"))
        .collect::<Vec<_>>();
    assert_eq!(unresolved.len(), 2);
    for warning in unresolved {
        let start = warning["start"].as_u64().unwrap() as usize;
        let end = warning["end"].as_u64().unwrap() as usize;
        assert!(matches!(&API_SOURCE[start..end], "[secret]" | "[Missing]"));
    }
    let before = document_bytes(&output);
    fs::write(output.join("stale-owned.txt"), "old output").unwrap();
    let second = generate(&project, &[]);
    assert!(second.status.success());
    assert_eq!(first.stderr, second.stderr);
    assert_eq!(before, document_bytes(&output));
    assert!(!project.path.join("dist/.seseragi-doc.lock").exists());
    assert!(fs::read_dir(project.path.join("dist"))
        .unwrap()
        .all(|entry| entry.unwrap().file_name() == "doc"));
}

#[test]
fn include_private_keeps_private_api_in_a_separate_link_scope() {
    let project = LockedProject::doc_tests();
    fs::write(project.source(), API_SOURCE).unwrap();
    project.update_lock();
    let result = generate(&project, &["--include-private"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let index: serde_json::Value =
        serde_json::from_slice(&fs::read(project.path.join("dist/doc/search-index.json")).unwrap())
            .unwrap();
    let secret = index["symbols"]
        .as_array()
        .unwrap()
        .iter()
        .find(|symbol| symbol["name"] == "secret")
        .unwrap();
    assert_eq!(secret["scope"], "private");
    assert_eq!(secret["deprecated"]["message"], "Use current");
    assert!(index["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning["message"].as_str().unwrap().contains("[secret]")));
}

#[test]
fn test_or_compile_failure_preserves_the_previous_complete_document_directory() {
    let project = LockedProject::doc_tests();
    let initial = generate(&project, &[]);
    assert!(initial.status.success());
    let output = project.path.join("dist/doc");
    let before = document_bytes(&output);
    let bad_fence = fs::read_to_string(project.source())
        .unwrap()
        .replace("// 3", "// 4");
    fs::write(project.source(), bad_fence).unwrap();
    project.update_lock();
    let failed = generate(&project, &["--test"]);
    assert_eq!(failed.status.code(), Some(1));
    assert_eq!(before, document_bytes(&output));
    fs::write(project.source(), "pub fn invalid -> Int = \"wrong\"\n").unwrap();
    project.update_lock();
    let failed = generate(&project, &[]);
    assert_eq!(failed.status.code(), Some(2));
    assert_eq!(before, document_bytes(&output));
}

#[test]
fn zero_test_blocks_generate_documents_without_requiring_a_target() {
    let project = LockedProject::doc_tests();
    fs::write(project.source(), "pub fn answer -> Int = 42\n").unwrap();
    project.update_lock();
    let output = generate(&project, &["--test"]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"0 passed; 0 failed\n");
    assert!(project.path.join("dist/doc/index.html").is_file());
}

#[test]
fn tests_trait_and_inherent_member_fences_before_document_publication() {
    let project = LockedProject::doc_tests();
    let source = concat!(
        "pub trait Render<A> {\n",
        "  /// ```seseragi\n  /// let traitExample: Int = 1\n  /// ```\n",
        "  deprecated \"Use render\" fn old value: A -> String\n}\n",
        "pub struct Box { pub value: Int }\nimpl Box {\n",
        "  /// ```seseragi\n  /// let methodExample: Int = 2\n  /// ```\n",
        "  pub deprecated \"Use read\" fn old self: Box -> Int = self.value\n}\n",
    );
    fs::write(project.source(), source).unwrap();
    project.update_lock();
    let result = generate(&project, &["--test"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "PASS math::Render.old#1 check\nPASS math::Box.old#1 check\n2 passed; 0 failed\n"
    );
    assert!(project.path.join("dist/doc/search-index.json").is_file());
}

#[test]
fn refuses_to_replace_unowned_documents_and_releases_its_output_lease() {
    let project = LockedProject::doc_tests();
    let output = project.path.join("dist/doc");
    fs::create_dir_all(&output).unwrap();
    fs::write(output.join("reader.txt"), "preserve").unwrap();
    let rejected = generate(&project, &[]);
    assert_eq!(rejected.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("unmanaged"));
    assert_eq!(fs::read(output.join("reader.txt")).unwrap(), b"preserve");
    assert!(!project.path.join("dist/.seseragi-doc.lock").exists());
}

#[cfg(unix)]
#[test]
fn refuses_symlink_document_output_and_parent_without_touching_the_target() {
    let project = LockedProject::doc_tests();
    let outside = project.root.join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("reader.txt"), "preserve").unwrap();
    let dist = project.path.join("dist");
    fs::create_dir(&dist).unwrap();
    std::os::unix::fs::symlink(&outside, dist.join("doc")).unwrap();
    assert_eq!(generate(&project, &[]).status.code(), Some(2));
    assert_eq!(fs::read(outside.join("reader.txt")).unwrap(), b"preserve");
    fs::remove_file(dist.join("doc")).unwrap();
    fs::remove_dir(&dist).unwrap();
    std::os::unix::fs::symlink(&outside, &dist).unwrap();
    assert_eq!(generate(&project, &[]).status.code(), Some(2));
    assert!(!outside.join("doc").exists());
}

#[test]
fn an_existing_output_lease_preserves_the_holder_and_the_previous_documents() {
    let project = LockedProject::doc_tests();
    assert!(generate(&project, &[]).status.success());
    let output = project.path.join("dist/doc");
    let previous = document_bytes(&output);
    let lease = project.path.join("dist/.seseragi-doc.lock");
    fs::write(&lease, "another writer owns this lease\n").unwrap();
    let rejected = generate(&project, &[]);
    assert_eq!(rejected.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("cannot acquire doc output lease"));
    assert_eq!(
        fs::read(&lease).unwrap(),
        b"another writer owns this lease\n"
    );
    assert_eq!(document_bytes(&output), previous);
}

#[test]
fn reports_stdout_and_diagnostic_code_mismatches_with_original_locations() {
    let project = LockedProject::doc_tests();
    let source = fs::read_to_string(project.source())
        .unwrap()
        .replace("// 3", "// 4")
        .replace("compile_fail SES-T0101", "compile_fail SES-T9999");
    fs::write(project.source(), source).unwrap();
    project.update_lock();
    let output = project.run();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("FAIL math::add#2 run"));
    assert!(stdout.contains("FAIL math::add#3 compile_fail SES-T9999"));
    assert!(stdout.ends_with("1 passed; 2 failed\n"));
    assert!(stderr.contains("math.ssrg:"));
    assert!(stderr.contains("stdout mismatch"));
    assert!(stderr.contains("expected error[SES-T9999] was not produced"));
    assert!(stderr.contains("SES-T0101"));
    assert!(stderr.contains("math.ssrg:22:"));
}

#[test]
fn rejects_run_blocks_without_a_selected_target_before_reporting_blocks() {
    let project = LockedProject::doc_tests();
    let manifest = fs::read_to_string(project.path.join("seseragi.toml"))
        .unwrap()
        .replace("\n[test]\ntarget = \"test-js\"\n", "\n");
    fs::write(project.path.join("seseragi.toml"), manifest).unwrap();
    project.update_lock();
    let output = project.run();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("doc run target is required"));
}

#[test]
fn help_exposes_the_document_test_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("seseragi doc [path/to/package] --test"));
}
