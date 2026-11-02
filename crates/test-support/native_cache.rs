#![allow(dead_code)]

#[path = "temporary.rs"]
mod temporary;

use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex, Weak},
};

struct Cache(PathBuf);

impl Drop for Cache {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn shared_cache() -> Arc<Cache> {
    static ACTIVE: Mutex<Weak<Cache>> = Mutex::new(Weak::new());
    let mut active = ACTIVE.lock().unwrap();
    if let Some(cache) = active.upgrade() {
        return cache;
    }
    let cache = Arc::new(Cache(temporary::temporary_directory(
        "seseragi-native-test-cache",
    )));
    *active = Arc::downgrade(&cache);
    cache
}

thread_local! {
    // Test threads share compilation work and release the cache when they exit.
    static CACHE: Arc<Cache> = shared_cache();
}

pub fn directory() -> PathBuf {
    CACHE.with(|cache| cache.0.clone())
}

pub fn configure(command: &mut Command) {
    if !command.get_envs().any(|(key, _)| key == "XDG_CACHE_HOME") {
        command.env("XDG_CACHE_HOME", directory());
    }
}

pub fn command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut command = Command::new(program);
    configure(&mut command);
    command
}
