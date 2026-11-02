#![allow(dead_code)]

#[path = "temporary.rs"]
mod temporary;

use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex, Weak},
};

struct Cache(PathBuf, Option<PathBuf>);

impl Drop for Cache {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
        if let Some(target) = &self.1 {
            if target.exists() {
                fs::remove_dir_all(target).unwrap();
            }
        }
    }
}

fn shared_cache() -> Arc<Cache> {
    static ACTIVE: Mutex<Weak<Cache>> = Mutex::new(Weak::new());
    let mut active = ACTIVE.lock().unwrap();
    if let Some(cache) = active.upgrade() {
        return cache;
    }
    let directory = temporary::temporary_directory("seseragi-native-test-cache");
    let target = std::env::var_os("CARGO_TARGET_DIR").map(|base| {
        PathBuf::from(base)
            .join("seseragi-native-tests")
            .join(directory.file_name().unwrap())
    });
    let cache = Arc::new(Cache(directory, target));
    *active = Arc::downgrade(&cache);
    cache
}

thread_local! {
    // Test threads share compilation work and release the cache when they exit.
    static CACHE: Arc<Cache> = shared_cache();
}

pub fn configure(command: &mut Command) {
    if !command.get_envs().any(|(key, _)| key == "CARGO_TARGET_DIR") {
        CACHE.with(|cache| {
            if let Some(target) = &cache.1 {
                command.env("CARGO_TARGET_DIR", target);
            }
        });
    }
}

pub fn command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut command = Command::new(program);
    configure(&mut command);
    command
}
