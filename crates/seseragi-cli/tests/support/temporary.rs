use std::{
    fs,
    io::ErrorKind,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

/// Reserve a private directory, including when parallel tests share a clock tick.
pub fn temporary_directory(prefix: &str) -> PathBuf {
    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);
    loop {
        let ordinal = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("{prefix}-{}-{ordinal}", std::process::id()));
        match fs::create_dir(&root) {
            Ok(()) => return root,
            // A previous process with a recycled PID can leave a directory behind.
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => panic!(
                "cannot reserve temporary directory {}: {error}",
                root.display()
            ),
        }
    }
}
