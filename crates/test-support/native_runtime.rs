#![allow(dead_code)]

#[path = "native_cache.rs"]
mod cache;

pub fn rustc() -> seseragi_backend_rust::native_runtime::Compiler {
    seseragi_backend_rust::native_runtime::compiler(
        &cache::directory().join("seseragi/native-runtime"),
    )
    .unwrap()
}

pub fn configure(command: &mut std::process::Command) {
    cache::configure(command);
}

// Instrumentation probes insert counters into runtime bodies before compiling.
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
