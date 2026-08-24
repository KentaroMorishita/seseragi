use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn requires_an_explicit_lock_update_and_never_rewrites_during_build() {
    let project = TempProject::new("stale");
    project.write("seseragi.toml", &manifest("fixture/locked-app", ""));
    project.write("src/main.ssrg", "pub effect fn main = println \"ready\"\n");

    let missing = seseragi(&project, ["build", "."]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&missing.stderr).contains("SES-K0102"));
    assert!(!project.path().join("seseragi.lock").exists());

    let updated = seseragi(&project, ["lock", "update"]);
    assert!(
        updated.status.success(),
        "{}",
        String::from_utf8_lossy(&updated.stderr)
    );
    let before = fs::read(project.path().join("seseragi.lock")).unwrap();
    assert_eq!(before.last(), Some(&b'\n'));

    let built = seseragi(&project, ["build", "."]);
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );

    project.write(
        "src/main.ssrg",
        "pub effect fn main = println \"changed\"\n",
    );
    let stale = seseragi(&project, ["build", "."]);
    assert_eq!(stale.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&stale.stderr).contains("content digest"));
    assert_eq!(
        fs::read(project.path().join("seseragi.lock")).unwrap(),
        before
    );
}

#[test]
fn locks_and_runs_the_exact_path_dependency_graph() {
    let project = TempProject::new("path");
    project.write(
        "seseragi.toml",
        &manifest(
            "fixture/locked-path-app",
            "math = { package = \"fixture/locked-math\", path = \"vendor/math\" }",
        ),
    );
    project.write(
        "src/main.ssrg",
        "import { message } from \"math\"\n\npub effect fn main = println message\n",
    );
    project.write(
        "vendor/math/seseragi.toml",
        "[package]\nname = \"fixture/locked-math\"\nversion = \"2.1.4\"\nlanguage = \"^0.1.0\"\n\n[exports]\n\".\" = \"lib\"\n",
    );
    project.write(
        "vendor/math/src/lib.ssrg",
        "pub let message: String = \"42\"\n",
    );

    let updated = seseragi(&project, ["lock", "update"]);
    assert!(
        updated.status.success(),
        "{}",
        String::from_utf8_lossy(&updated.stderr)
    );
    let lock = fs::read_to_string(project.path().join("seseragi.lock")).unwrap();
    assert!(lock.contains("fixture/locked-math@2.1.4#path:vendor/math"));
    assert!(!lock.contains(project.path().to_string_lossy().as_ref()));

    let run = seseragi(&project, ["run", "."]);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "42\n");
}

#[test]
fn locks_provider_selection_in_the_canonical_project_lock() {
    let project = TempProject::new("provider");
    project.write(
        "seseragi.toml",
        include_str!(
            "../../../examples/spec/fixtures/projects/provider-http-client-e2e/seseragi.toml"
        ),
    );
    project.write(
        "src/main.ssrg",
        include_str!(
            "../../../examples/spec/fixtures/projects/provider-http-client-e2e/src/main.ssrg"
        ),
    );

    let updated = seseragi(&project, ["lock", "update"]);
    assert!(
        updated.status.success(),
        "{}",
        String::from_utf8_lossy(&updated.stderr)
    );
    let path = project.path().join("seseragi.lock");
    let lock = fs::read_to_string(&path).unwrap();
    let parsed = seseragi_project::parse_lockfile(&lock).unwrap();
    assert_eq!(parsed.providers.len(), 2);
    assert!(parsed
        .providers
        .iter()
        .all(|provider| provider.service == "std/http::HttpClient"));
    let process_provider = parsed
        .providers
        .iter()
        .find(|provider| provider.target == "bun-process")
        .unwrap();
    assert!(process_provider
        .artifact_digest
        .strip_prefix("sha256:")
        .is_some_and(|digest| digest.len() == 64));

    let built = seseragi(&project, ["build", "."]);
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );

    let stale = lock.replacen(
        &process_provider.artifact_digest,
        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        1,
    );
    fs::write(&path, stale).unwrap();
    let rejected = seseragi(&project, ["build", "."]);
    assert_eq!(rejected.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("SES-K0102"));
}

fn seseragi<const N: usize>(project: &TempProject, arguments: [&str; N]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(arguments)
        .current_dir(project.path())
        .output()
        .unwrap()
}

fn manifest(name: &str, dependencies: &str) -> String {
    format!(
        "[package]\nname = \"{name}\"\nversion = \"1.0.0\"\nlanguage = \"^0.1.0\"\n\n[dependencies]\n{dependencies}\n\n[run]\nentry = \"main\"\ntarget = \"process\"\n"
    )
}

struct TempProject {
    path: PathBuf,
}

impl TempProject {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sequence = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "seseragi-cli-lock-{name}-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, relative: &str, source: &str) {
        let path = self.path.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, source).unwrap();
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.path).unwrap();
    }
}
