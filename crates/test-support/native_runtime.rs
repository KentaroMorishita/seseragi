#![allow(dead_code)]

#[path = "native_cache.rs"]
mod build_outputs;

pub fn configure(command: &mut std::process::Command) {
    build_outputs::configure(command);
}

pub fn write_program(
    path: impl AsRef<std::path::Path>,
    source: impl AsRef<str>,
) -> std::io::Result<()> {
    std::fs::write(path, inline_runtime(source.as_ref()))
}

// Instrumentation probes compile the runtime in the same crate as their counters.
pub fn inline_runtime(source: &str) -> String {
    source
        .replace(
            "use seseragi_native_runtime::*;",
            seseragi_backend_rust::native_runtime::SOURCE,
        )
        .replace(
            "use std::{cell::{Cell, RefCell}, collections::BTreeMap};\n",
            "",
        )
}
