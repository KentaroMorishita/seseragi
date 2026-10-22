use super::{finish_run, prepare_directory_in, run_main, RunError, RunOutcome};
use std::collections::HashSet;
use std::fs;

#[test]
fn execution_directory_preserves_preexisting_legacy_paths() {
    let root = tempfile::tempdir().unwrap();
    let legacy = root
        .path()
        .join("seseragi-run")
        .join(format!("{}-0", std::process::id()));
    fs::create_dir_all(&legacy).unwrap();
    fs::write(legacy.join("sentinel"), "keep legacy directory").unwrap();
    let existing = root.path().join("seseragi-run-existing");
    fs::create_dir(&existing).unwrap();
    fs::write(existing.join("sentinel"), "keep existing sibling").unwrap();

    let directory = prepare_directory_in(root.path()).unwrap();
    let path = directory.path().to_owned();
    assert_eq!(path.parent(), Some(root.path()));
    let name = path.file_name().unwrap().to_str().unwrap();
    let random = name.strip_prefix("seseragi-run-").unwrap();
    assert!(random.len() >= 16);
    assert!(random
        .chars()
        .all(|character| character.is_ascii_alphanumeric()));
    fs::write(path.join("owned"), "this run only").unwrap();
    finish_run(Ok(RunOutcome { exit_code: 0 }), directory).unwrap();

    assert!(!path.exists());
    assert_eq!(
        fs::read_to_string(legacy.join("sentinel")).unwrap(),
        "keep legacy directory"
    );
    assert_eq!(
        fs::read_to_string(existing.join("sentinel")).unwrap(),
        "keep existing sibling"
    );
}

#[cfg(unix)]
#[test]
fn execution_directory_ignores_legacy_parent_and_child_symlinks() {
    use std::os::unix::fs::symlink;

    for redirect_parent in [true, false] {
        let root = tempfile::tempdir().unwrap();
        let victim = root.path().join("victim");
        fs::create_dir(&victim).unwrap();
        fs::write(victim.join("entry.ts"), "keep victim entry").unwrap();
        let legacy = root.path().join("seseragi-run");
        let redirect = if redirect_parent {
            legacy
        } else {
            fs::create_dir(&legacy).unwrap();
            legacy.join(format!("{}-0", std::process::id()))
        };
        symlink(&victim, &redirect).unwrap();

        let directory = prepare_directory_in(root.path()).unwrap();
        fs::write(directory.path().join("entry.ts"), "owned entry").unwrap();
        finish_run(Ok(RunOutcome { exit_code: 0 }), directory).unwrap();

        assert!(fs::symlink_metadata(redirect)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(
            fs::read_to_string(victim.join("entry.ts")).unwrap(),
            "keep victim entry"
        );
        assert_eq!(fs::read_dir(&victim).unwrap().count(), 1);
    }
}

#[cfg(unix)]
#[test]
fn execution_directory_is_private_and_cleanup_does_not_follow_child_symlinks() {
    use std::os::unix::fs::{symlink, PermissionsExt};

    let root = tempfile::tempdir().unwrap();
    let victim = root.path().join("victim");
    fs::create_dir(&victim).unwrap();
    fs::write(victim.join("sentinel"), "keep outside owned directory").unwrap();
    let directory = prepare_directory_in(root.path()).unwrap();
    let path = directory.path().to_owned();
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o700
    );
    symlink(&victim, path.join("redirect")).unwrap();

    finish_run(Ok(RunOutcome { exit_code: 0 }), directory).unwrap();
    assert!(!path.exists());
    assert_eq!(
        fs::read_to_string(victim.join("sentinel")).unwrap(),
        "keep outside owned directory"
    );
}

#[test]
fn concurrent_execution_directories_keep_separate_ownership_and_cleanup() {
    let root = tempfile::tempdir().unwrap();
    let directories = std::thread::scope(|scope| {
        let tasks = (0..8)
            .map(|index| {
                let root = root.path();
                scope.spawn(move || {
                    let directory = prepare_directory_in(root).unwrap();
                    fs::write(directory.path().join("owner"), index.to_string()).unwrap();
                    (directory, index)
                })
            })
            .collect::<Vec<_>>();
        tasks
            .into_iter()
            .map(|task| task.join().unwrap())
            .collect::<Vec<_>>()
    });
    let paths = directories
        .iter()
        .map(|(directory, _)| directory.path().to_owned())
        .collect::<HashSet<_>>();
    assert_eq!(paths.len(), 8);
    for (directory, owner) in directories {
        let path = directory.path().to_owned();
        assert_eq!(
            fs::read_to_string(path.join("owner")).unwrap(),
            owner.to_string()
        );
        finish_run(Ok(RunOutcome { exit_code: 0 }), directory).unwrap();
        assert!(!path.exists());
    }
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
}

#[test]
fn failed_execution_cleans_owned_directory_and_preserves_primary_error() {
    let root = tempfile::tempdir().unwrap();
    let directory = prepare_directory_in(root.path()).unwrap();
    let path = directory.path().to_owned();
    fs::write(path.join("partial-stage"), "owned output").unwrap();
    let result: Result<RunOutcome, RunError> = finish_run(
        Err(RunError::InvalidEntry("fixture failure".to_owned())),
        directory,
    );
    assert!(matches!(result, Err(RunError::InvalidEntry(message)) if message == "fixture failure"));
    assert!(!path.exists());
}

#[test]
fn execution_directory_is_removed_when_execution_unwinds() {
    let root = tempfile::tempdir().unwrap();
    let directory = prepare_directory_in(root.path()).unwrap();
    let path = directory.path().to_owned();
    fs::write(path.join("partial-stage"), "owned output").unwrap();
    let result = std::panic::catch_unwind(move || {
        let _owned_directory = directory;
        panic!("fixture execution unwound");
    });
    assert!(result.is_err());
    assert!(!path.exists());
}

#[test]
fn single_file_main_still_runs_with_an_owned_execution_directory() {
    let compiled = seseragi_driver::compile_module(seseragi_driver::CompileInput::new(
        "main.ssrg",
        "fixture/main",
        "pub effect fn main = succeed ()\n",
    ))
    .unwrap();
    assert_eq!(run_main(&compiled).unwrap(), RunOutcome { exit_code: 0 });
}
