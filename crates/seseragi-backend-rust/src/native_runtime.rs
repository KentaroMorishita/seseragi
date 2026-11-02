//! Source and native linking for the generated program runtime.
include!("native_runtime_source.rs");

#[cfg(not(target_arch = "wasm32"))]
mod cache {
    use sha2::{Digest, Sha256};
    use std::{
        fs::{self, File},
        io,
        ops::{Deref, DerefMut},
        path::{Path, PathBuf},
        process::Command,
        sync::atomic::{AtomicUsize, Ordering},
        time::{Duration, SystemTime},
    };

    pub fn cache_directory() -> Result<PathBuf, String> {
        if let Some(path) =
            std::env::var_os("XDG_CACHE_HOME").filter(|path| Path::new(path).is_absolute())
        {
            return Ok(PathBuf::from(path).join("seseragi/native-runtime"));
        }
        #[cfg(windows)]
        let base = std::env::var_os("LOCALAPPDATA")
            .filter(|path| Path::new(path).is_absolute())
            .map(PathBuf::from);
        #[cfg(not(windows))]
        let base = std::env::var_os("HOME")
            .filter(|path| Path::new(path).is_absolute())
            .map(|path| {
                PathBuf::from(path).join(if cfg!(target_os = "macos") {
                    "Library/Caches"
                } else {
                    ".cache"
                })
            });
        base.map(|path| path.join("seseragi/native-runtime"))
            .ok_or_else(|| {
                "native builds require a user cache directory; set XDG_CACHE_HOME".into()
            })
    }

    const OPTIONS: &[&str] = &[
        "--edition=2021",
        "--crate-name=seseragi_native_runtime",
        "--crate-type=rlib",
        "-O",
        "--cap-lints=allow",
    ];
    const RETAINED_ENTRIES: usize = 3;

    /// A compiler command that keeps its runtime cache entry locked.
    /// Keep this value alive until the compiler process exits.
    pub struct Compiler {
        command: Command,
        _lock: File,
    }

    impl Deref for Compiler {
        type Target = Command;

        fn deref(&self) -> &Command {
            &self.command
        }
    }

    impl DerefMut for Compiler {
        fn deref_mut(&mut self) -> &mut Command {
            &mut self.command
        }
    }

    /// Prepares rustc to link the runtime, compiling it only on a cache miss.
    /// `cache` must be a user or project cache outside the source tree.
    pub fn compiler(cache: &Path) -> Result<Compiler, String> {
        let version = Command::new("rustc")
            .arg("-vV")
            .output()
            .map_err(|e| format!("could not start rustc: {e}; install a Rust toolchain"))?;
        if !version.status.success() {
            return Err(format!(
                "could not query rustc: {}",
                String::from_utf8_lossy(&version.stderr)
            ));
        }
        let key = cache_key(super::SOURCE, &version.stdout);
        let (directory, lock) = lock_entry(cache, &key).map_err(|e| e.to_string())?;
        let library = directory.join("libseseragi_native_runtime.rlib");
        let build = !library.is_file();
        if build {
            let stage = Staging::new(&directory)?;
            let source = stage.0.join("lib.rs");
            let output = stage.0.join("libseseragi_native_runtime.rlib");
            fs::write(&source, super::SOURCE).map_err(|e| e.to_string())?;
            let result = Command::new("rustc")
                .args(OPTIONS)
                .arg(&source)
                .arg("-o")
                .arg(&output)
                .output()
                .map_err(|e| format!("could not start rustc: {e}; install a Rust toolchain"))?;
            if !result.status.success() {
                return Err(format!(
                    "native runtime compilation failed:\n{}",
                    String::from_utf8_lossy(&result.stderr)
                ));
            }
            fs::rename(output, &library).map_err(|e| e.to_string())?;
        }
        fs::write(directory.join("last-used"), []).map_err(|e| e.to_string())?;
        if build {
            let _ = prune(cache);
        }
        let mut compiler = Command::new("rustc");
        let mut dependency = std::ffi::OsString::from("seseragi_native_runtime=");
        dependency.push(library);
        compiler.arg("--extern").arg(dependency);
        Ok(Compiler {
            command: compiler,
            _lock: lock,
        })
    }

    fn open_lock(path: &Path) -> io::Result<File> {
        File::options()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)
    }

    fn lock_cache(cache: &Path) -> io::Result<File> {
        fs::create_dir_all(cache)?;
        let lock = open_lock(&cache.join(".cache.lock"))?;
        lock.lock()?;
        Ok(lock)
    }

    fn lock_entry(cache: &Path, key: &str) -> io::Result<(PathBuf, File)> {
        loop {
            // Coordinate opening entry locks with removal so a waiter cannot
            // acquire a lock on a directory that has already been deleted.
            let cache_lock = lock_cache(cache)?;
            let directory = cache.join(key);
            fs::create_dir_all(&directory)?;
            let lock = open_lock(&directory.join("build.lock"))?;
            match lock.try_lock() {
                Ok(()) => return Ok((directory, lock)),
                Err(std::fs::TryLockError::WouldBlock) => {}
                Err(std::fs::TryLockError::Error(error)) => return Err(error),
            }
            drop(lock);
            drop(cache_lock);
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn prune(cache: &Path) -> io::Result<()> {
        let _cache_lock = lock_cache(cache)?;
        let mut entries = Vec::new();
        for entry in fs::read_dir(cache)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !entry.file_type()?.is_dir()
                || name.len() != 64
                || !name.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                continue;
            }
            let directory = entry.path();
            let used = fs::metadata(directory.join("last-used"))
                .or_else(|_| fs::metadata(directory.join("libseseragi_native_runtime.rlib")))
                .and_then(|metadata| metadata.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            entries.push((used, directory));
        }
        entries.sort_unstable_by(|a, b| b.cmp(a));
        for (_, directory) in entries.into_iter().skip(RETAINED_ENTRIES) {
            let Ok(lock) = open_lock(&directory.join("build.lock")) else {
                continue;
            };
            if lock.try_lock().is_ok() {
                // The cache lock still prevents new users from opening the entry.
                // Close this handle so Windows can remove its file and directory.
                drop(lock);
                let _ = fs::remove_dir_all(directory);
            }
        }
        Ok(())
    }

    fn cache_key(source: &str, compiler: &[u8]) -> String {
        let mut hash = Sha256::new();
        for input in std::iter::once(source.as_bytes())
            .chain(std::iter::once(compiler))
            .chain(OPTIONS.iter().map(|option| option.as_bytes()))
        {
            hash.update((input.len() as u64).to_le_bytes());
            hash.update(input);
        }
        format!("{:x}", hash.finalize())
    }

    struct Staging(PathBuf);
    impl Staging {
        fn new(parent: &Path) -> Result<Self, String> {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            loop {
                let path = parent.join(format!(
                    ".build-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
                match fs::create_dir(&path) {
                    Ok(()) => return Ok(Self(path)),
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(e) => return Err(e.to_string()),
                }
            }
        }
    }
    impl Drop for Staging {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn scratch() -> Staging {
            Staging::new(&std::env::temp_dir()).unwrap()
        }

        fn entry(cache: &Path, index: u64) -> PathBuf {
            let directory = cache.join(format!("{index:064x}"));
            fs::create_dir(&directory).unwrap();
            fs::write(
                directory.join("libseseragi_native_runtime.rlib"),
                b"runtime",
            )
            .unwrap();
            last_used(&directory, index);
            directory
        }

        fn last_used(directory: &Path, seconds: u64) {
            let marker = File::create(directory.join("last-used")).unwrap();
            marker
                .set_times(
                    fs::FileTimes::new()
                        .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)),
                )
                .unwrap();
        }

        #[test]
        fn cache_identity_tracks_runtime_source_and_rustc_build() {
            let key = cache_key("runtime", b"rustc host/version");
            assert_eq!(key, cache_key("runtime", b"rustc host/version"));
            assert_ne!(key, cache_key("changed runtime", b"rustc host/version"));
            assert_ne!(key, cache_key("runtime", b"different rustc host/version"));
        }

        #[test]
        fn pruning_keeps_the_three_most_recently_used_entries() {
            let scratch = scratch();
            let directories = (0..6)
                .map(|index| entry(&scratch.0, index))
                .collect::<Vec<_>>();
            last_used(&directories[0], 100);
            let unrelated = scratch.0.join("other-cache");
            fs::create_dir(&unrelated).unwrap();
            fs::write(unrelated.join("keep"), "unrelated").unwrap();
            prune(&scratch.0).unwrap();
            for (index, directory) in directories.iter().enumerate() {
                assert_eq!(directory.exists(), [0, 4, 5].contains(&index), "{index}");
            }
            assert_eq!(
                fs::read_to_string(unrelated.join("keep")).unwrap(),
                "unrelated"
            );
        }

        #[test]
        fn pruning_skips_builds_in_progress_and_handles_entries_without_markers() {
            let scratch = scratch();
            let directories = (0..6)
                .map(|index| entry(&scratch.0, index))
                .collect::<Vec<_>>();
            fs::remove_file(directories[0].join("libseseragi_native_runtime.rlib")).unwrap();
            fs::remove_file(directories[0].join("last-used")).unwrap();
            let (_, building) = lock_entry(&scratch.0, &format!("{:064x}", 0)).unwrap();
            let legacy = directories[1].join("libseseragi_native_runtime.rlib");
            File::options()
                .write(true)
                .open(legacy)
                .unwrap()
                .set_times(fs::FileTimes::new().set_modified(SystemTime::UNIX_EPOCH))
                .unwrap();
            fs::remove_file(directories[1].join("last-used")).unwrap();
            prune(&scratch.0).unwrap();
            assert!(directories[0].exists());
            assert!(!directories[1].exists());
            assert!(!directories[2].exists());
            for directory in &directories[3..] {
                assert!(directory.exists());
            }
            drop(building);
            prune(&scratch.0).unwrap();
            assert!(!directories[0].exists());
        }

        #[test]
        fn cache_hits_refresh_last_use_and_hold_the_lock_until_linking_finishes() {
            let scratch = scratch();
            let version = Command::new("rustc").arg("-vV").output().unwrap();
            assert!(version.status.success());
            let key = cache_key(super::super::SOURCE, &version.stdout);
            let directory = scratch.0.join(key);
            fs::create_dir(&directory).unwrap();
            let library = directory.join("libseseragi_native_runtime.rlib");
            fs::write(&library, "cached runtime").unwrap();
            last_used(&directory, 0);
            let modified = fs::metadata(&library).unwrap().modified().unwrap();
            let compiler = compiler(&scratch.0).unwrap();
            assert!(
                fs::metadata(directory.join("last-used"))
                    .unwrap()
                    .modified()
                    .unwrap()
                    > SystemTime::UNIX_EPOCH
            );
            assert_eq!(fs::read_to_string(&library).unwrap(), "cached runtime");
            assert_eq!(
                fs::metadata(&library).unwrap().modified().unwrap(),
                modified
            );
            let lock = open_lock(&directory.join("build.lock")).unwrap();
            assert!(matches!(
                lock.try_lock(),
                Err(std::fs::TryLockError::WouldBlock)
            ));
            for index in 1..=3 {
                entry(&scratch.0, index);
            }
            last_used(&directory, 0);
            prune(&scratch.0).unwrap();
            assert!(library.exists());
            drop(compiler);
            assert!(lock.try_lock().is_ok());
            drop(lock);
            prune(&scratch.0).unwrap();
            assert!(!directory.exists());
        }

        #[test]
        fn entry_acquisition_waits_for_pruning_and_opens_the_recreated_directory() {
            let scratch = scratch();
            let directory = entry(&scratch.0, 0);
            std::thread::scope(|scope| {
                let cache_lock = lock_cache(&scratch.0).unwrap();
                let lock = open_lock(&directory.join("build.lock")).unwrap();
                lock.try_lock().unwrap();
                drop(lock);
                let (ready, started) = std::sync::mpsc::channel();
                let (acquired, result) = std::sync::mpsc::channel();
                let cache = &scratch.0;
                scope.spawn(move || {
                    ready.send(()).unwrap();
                    acquired
                        .send(lock_entry(cache, &format!("{:064x}", 0)).unwrap())
                        .unwrap();
                });
                started.recv_timeout(Duration::from_secs(10)).unwrap();
                assert!(matches!(
                    result.recv_timeout(Duration::from_millis(50)),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout)
                ));
                fs::remove_dir_all(&directory).unwrap();
                drop(cache_lock);
                let (recreated, _lock) = result.recv_timeout(Duration::from_secs(10)).unwrap();
                assert_eq!(recreated, directory);
                assert!(recreated.is_dir());
                assert!(!recreated.join("libseseragi_native_runtime.rlib").exists());
                let probe = open_lock(&recreated.join("build.lock")).unwrap();
                assert!(matches!(
                    probe.try_lock(),
                    Err(std::fs::TryLockError::WouldBlock)
                ));
            });
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use cache::{cache_directory, compiler, Compiler};
