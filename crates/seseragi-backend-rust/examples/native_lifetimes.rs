//! Opt-in research. Generated programs still use the documented pure ABI.
#![allow(dead_code)]
#[path = "native_lifetimes/core_emit.rs"]
mod core_emit;
#[path = "native_lifetimes/effect.rs"]
mod effect;
#[path = "native_lifetimes/heap.rs"]
mod heap;
#[path = "native_lifetimes/scope.rs"]
mod scope;

use heap::Heap;
use serde_json::json;
use std::time::Instant;

fn main() -> Result<(), String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if !args.is_empty() {
        if args.len() != 3 || !["emit", "dump"].contains(&args[0].as_str()) {
            return Err("native_lifetimes [emit|dump source.ssrg output]".into());
        }
        let source = std::fs::read_to_string(&args[1]).map_err(|e| e.to_string())?;
        let checked = seseragi_driver::compile_core_module(seseragi_driver::CompileInput::new(
            &args[1],
            "research/main",
            &source,
        ))
        .map_err(|e| format!("{e:?}"))?;
        let output = if args[0] == "dump" {
            serde_json::to_string_pretty(&checked.core_ir).unwrap()
        } else {
            core_emit::emit(&checked.core_ir)?
        };
        std::fs::write(&args[2], output).map_err(|e| e.to_string())?;
        return Ok(());
    }
    let mut measurements = Vec::new();
    for count in [1_000, 10_000, 100_000] {
        let heap = Heap::default();
        for _ in 0..count {
            let cell = heap.reference(heap.int(0));
            let callback = heap.function(vec![cell.clone()], |_, n| n);
            cell.write(callback);
        }
        assert_eq!(heap.live(), count * 2);
        let before = heap.registry();
        let start = Instant::now();
        let collected = heap.collect().unwrap();
        let elapsed = start.elapsed();
        assert_eq!(collected.reclaimed, count * 2);
        assert_eq!(heap.live(), 0);
        measurements.push(json!({"cycles": count, "registryBefore": before,
            "collection": collected, "elapsedMicros": elapsed.as_micros()}));
    }
    println!(
        "{}",
        json!({"model": "single-thread-rc-trace-v2",
        "productionIntegrated": false, "measurements": measurements,
        "resourceTraces": scope::scenarios()})
    );
    Ok(())
}
