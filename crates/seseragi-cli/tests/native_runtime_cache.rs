use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    path::Path,
    process::Command,
    time::{Duration, SystemTime},
};

#[path = "../../test-support/native_cache.rs"]
mod isolated_cache;

#[allow(dead_code)]
#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, successful};

fn build(source: &Path, destination: &Path, cache: &Path, logs: &Path) {
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(source)
            .args(["--entry", "main", "--out-dir"])
            .arg(destination)
            .env("XDG_CACHE_HOME", cache),
        logs,
    ));
}

#[test]
fn concurrent_native_builds_and_program_edits_reuse_the_runtime_crate() {
    let scratch = tempfile::tempdir().unwrap();
    let root = scratch.path();
    let cache = root.join("cache 雪");
    std::thread::scope(|scope| {
        for index in 0..3 {
            let cache = &cache;
            scope.spawn(move || {
                let directory = root.join(format!("program-{index}"));
                fs::create_dir(&directory).unwrap();
                let source = directory.join("main.ssrg");
                fs::write(
                    &source,
                    format!("pub effect fn main = println \"program {index}\"\n"),
                )
                .unwrap();
                build(&source, &directory.join("native"), cache, &directory);
                let output = successful(bounded(
                    &mut Command::new(
                        directory.join(format!("native/program{}", std::env::consts::EXE_SUFFIX)),
                    ),
                    &directory,
                ));
                assert_eq!(output.stdout, format!("program {index}\n").as_bytes());
            });
        }
    });
    let entries = fs::read_dir(cache.join("seseragi/native-runtime"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 2);
    assert!(cache.join("seseragi/native-runtime/.cache.lock").is_file());
    let entry = entries.iter().find(|path| path.is_dir()).unwrap();
    let library = entry.join("libseseragi_native_runtime.rlib");
    println!(
        "runtime cache entry: {} bytes",
        fs::metadata(&library).unwrap().len()
    );
    let original = Sha256::digest(fs::read(&library).unwrap());
    let modified = fs::metadata(&library).unwrap().modified().unwrap();
    let last_used = entry.join("last-used");
    File::options()
        .write(true)
        .open(&last_used)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(SystemTime::UNIX_EPOCH))
        .unwrap();
    let source = root.join("program-0/main.ssrg");
    fs::write(&source, "pub effect fn main = println \"edited\"\n").unwrap();
    let destination = root.join("program-0/native");
    build(&source, &destination, &cache, root);
    assert_eq!(Sha256::digest(fs::read(&library).unwrap()), original);
    assert_eq!(
        fs::metadata(&library).unwrap().modified().unwrap(),
        modified
    );
    assert!(fs::metadata(last_used).unwrap().modified().unwrap() > SystemTime::UNIX_EPOCH);
    assert_eq!(
        fs::read_dir(entry)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<BTreeSet<_>>(),
        ["build.lock", "last-used", "libseseragi_native_runtime.rlib"]
            .into_iter()
            .map(std::ffi::OsString::from)
            .collect()
    );
    let generated = fs::read_to_string(destination.join("program.rs")).unwrap();
    assert!(generated.contains("use seseragi_native_runtime::*;"));
    assert!(!generated.contains("enum V {"));
    fs::remove_dir_all(&cache).unwrap();
    let output = successful(bounded(
        &mut Command::new(destination.join(format!("program{}", std::env::consts::EXE_SUFFIX))),
        root,
    ));
    assert_eq!(output.stdout, b"edited\n");
}

#[test]
fn new_runtime_build_prunes_old_entries_but_preserves_an_active_entry() {
    let scratch = tempfile::tempdir().unwrap();
    let root = scratch.path();
    let cache = root.join("cache");
    let runtime = cache.join("seseragi/native-runtime");
    fs::create_dir_all(&runtime).unwrap();
    let mut old = Vec::new();
    for index in 1..=5 {
        let directory = runtime.join(format!("{index:064x}"));
        fs::create_dir(&directory).unwrap();
        fs::write(
            directory.join("libseseragi_native_runtime.rlib"),
            "old runtime",
        )
        .unwrap();
        File::create(directory.join("last-used"))
            .unwrap()
            .set_times(
                fs::FileTimes::new()
                    .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(index)),
            )
            .unwrap();
        old.push(directory);
    }
    let active = File::create(old[0].join("build.lock")).unwrap();
    active.lock().unwrap();
    let source = root.join("main.ssrg");
    fs::write(&source, "pub fn main -> Int = 42\n").unwrap();
    let destination = root.join("native");
    build(&source, &destination, &cache, root);
    for (index, directory) in old.iter().enumerate() {
        assert_eq!(directory.exists(), [0, 3, 4].contains(&index), "{index}");
    }
    assert_eq!(
        fs::read_dir(&runtime)
            .unwrap()
            .filter(|entry| entry.as_ref().unwrap().file_type().unwrap().is_dir())
            .count(),
        4
    );
    let output = successful(bounded(
        &mut Command::new(destination.join(format!("program{}", std::env::consts::EXE_SUFFIX))),
        root,
    ));
    assert_eq!(output.stdout, b"42\n");
}

#[test]
fn test_cache_is_shared_only_while_test_threads_are_alive() {
    let directory = std::thread::spawn(|| {
        let directory = isolated_cache::directory();
        assert!(directory.starts_with(std::env::temp_dir()));
        assert!(directory.is_dir());
        let other = std::thread::spawn(isolated_cache::directory)
            .join()
            .unwrap();
        assert_eq!(other, directory);
        let command = isolated_cache::command("seseragi");
        assert_eq!(
            command
                .get_envs()
                .find(|(key, _)| *key == "XDG_CACHE_HOME")
                .unwrap()
                .1,
            Some(directory.as_os_str())
        );
        directory
    })
    .join()
    .unwrap();
    assert!(!directory.exists());
}
