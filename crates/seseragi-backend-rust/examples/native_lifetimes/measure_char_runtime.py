"""Measure the ordinary generated Char kernel; allocator counting is a separate run.

Keep a baseline directory for the later, interleaved product comparison. Only
measurement main functions are attached; generated runtime/kernel bodies remain
byte-identical to the selected compiler's output.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess


def run(command, **kwargs):
    result = subprocess.run(list(map(str, command)), capture_output=True, text=True,
                            timeout=120, **kwargs)
    if result.returncode:
        raise RuntimeError(f"{command!r}\n{result.stdout}\n{result.stderr}")
    return result.stdout


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


TIMING = r'''
fn main() {
    let kernel = KERNEL(vec![]);
    let started = std::time::Instant::now();
    let mut warmups = 0;
    let mut samples = Vec::new();
    let mut managed_per_call = 0;
    while samples.len() < 5 {
        let recording = warmups >= 3 && started.elapsed().as_secs_f64() >= 1.0;
        let before = ALLOCATIONS.with(Cell::get);
        let now = std::time::Instant::now();
        let result = std::hint::black_box(&kernel)
            .apply(V::Int(std::hint::black_box(1114113)));
        let elapsed = now.elapsed().as_nanos();
        managed_per_call = ALLOCATIONS.with(Cell::get) - before;
        assert!(matches!(result, V::Int(620506872831)));
        if recording { samples.push(elapsed); } else { warmups += 1; }
    }
    drop(kernel); clear_dictionaries(); clear_globals();
    assert_eq!(LIVE.with(Cell::get), 0);
    println!("{{\"samplesNs\":{:?},\"warmups\":{},\"checksum\":620506872831,\"managedAllocationsPerCall\":{},\"managedLive\":0}}", samples, warmups, managed_per_call);
}
'''

ALLOCATOR = r'''
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
static COUNT_SYSTEM: AtomicBool = AtomicBool::new(false);
static SYSTEM_ALLOCS: AtomicUsize = AtomicUsize::new(0);
static SYSTEM_FREES: AtomicUsize = AtomicUsize::new(0);
static SYSTEM_REALLOCS: AtomicUsize = AtomicUsize::new(0);
static SYSTEM_REQUESTED: AtomicUsize = AtomicUsize::new(0);
static SYSTEM_RELEASED: AtomicUsize = AtomicUsize::new(0);
struct CountingSystem;
unsafe impl GlobalAlloc for CountingSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() && COUNT_SYSTEM.load(Relaxed) {
            SYSTEM_ALLOCS.fetch_add(1, Relaxed);
            SYSTEM_REQUESTED.fetch_add(layout.size(), Relaxed);
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if COUNT_SYSTEM.load(Relaxed) {
            SYSTEM_FREES.fetch_add(1, Relaxed);
            SYSTEM_RELEASED.fetch_add(layout.size(), Relaxed);
        }
        unsafe { System.dealloc(ptr, layout) };
    }
    unsafe fn realloc(&self, ptr: *mut u8, old: Layout, size: usize) -> *mut u8 {
        let next = unsafe { System.realloc(ptr, old, size) };
        if !next.is_null() && COUNT_SYSTEM.load(Relaxed) {
            SYSTEM_REALLOCS.fetch_add(1, Relaxed);
            SYSTEM_REQUESTED.fetch_add(size, Relaxed);
            SYSTEM_RELEASED.fetch_add(old.size(), Relaxed);
        }
        next
    }
}
#[global_allocator] static MEASURE_SYSTEM: CountingSystem = CountingSystem;
fn main() {
    let kernel = KERNEL(vec![]);
    assert!(matches!(kernel.apply(V::Int(1)), V::Int(0)));
    let before = ALLOCATIONS.with(Cell::get);
    let live = LIVE.with(Cell::get);
    COUNT_SYSTEM.store(true, Relaxed);
    let result = std::hint::black_box(&kernel)
        .apply(V::Int(std::hint::black_box(1114113)));
    COUNT_SYSTEM.store(false, Relaxed);
    assert!(matches!(result, V::Int(620506872831)));
    let nodes = ALLOCATIONS.with(Cell::get) - before;
    assert_eq!(LIVE.with(Cell::get), live);
    assert_eq!(SYSTEM_ALLOCS.load(Relaxed), SYSTEM_FREES.load(Relaxed));
    assert_eq!(SYSTEM_REQUESTED.load(Relaxed), SYSTEM_RELEASED.load(Relaxed));
    drop(kernel); clear_dictionaries(); clear_globals();
    assert_eq!(LIVE.with(Cell::get), 0);
    println!("{{\"allocations\":{},\"deallocations\":{},\"reallocations\":{},\"cumulativeRequestedBytes\":{},\"cumulativeReleasedBytes\":{},\"managedAllocations\":{},\"managedLive\":0,\"checksum\":620506872831}}", SYSTEM_ALLOCS.load(Relaxed), SYSTEM_FREES.load(Relaxed), SYSTEM_REALLOCS.load(Relaxed), SYSTEM_REQUESTED.load(Relaxed), SYSTEM_RELEASED.load(Relaxed), nodes);
}
'''

JS = '''import { kernel } from "./module.mjs";
const started = process.hrtime.bigint();
const samplesNs = [];
let warmups = 0;
while (samplesNs.length < 5) {
  const recording = warmups >= 3 && process.hrtime.bigint() - started >= 1000000000n;
  const before = process.hrtime.bigint();
  const result = kernel(1114113);
  const elapsed = Number(process.hrtime.bigint() - before);
  if (result !== 620506872831) throw Error(`checksum ${result}`);
  if (recording) samplesNs.push(elapsed); else warmups++;
}
console.log(JSON.stringify({samplesNs, warmups, checksum: 620506872831}));
'''


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", required=True)
    parser.add_argument("--typescript-emitter", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--compare-with")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[4]
    source = root / "examples/spec/fixtures/rust-backend/char-roundtrip-kernel.ssrg"
    stage = Path(args.out).resolve()
    stage.mkdir(parents=True, exist_ok=False)
    cli = Path(args.cli).resolve()
    emitter = Path(args.typescript_emitter).resolve()
    run([cli, "experimental", "build-native", source, "--entry", "probe",
         "--out-dir", stage / "native"])
    native = stage / "native" / ("program.exe" if os.name == "nt" else "program")
    native_env = dict(os.environ, PATH="")
    assert run([native], env=native_env).strip() == "620506872831"
    run([emitter, source, stage / "ts"])
    config = {
        "compilerOptions": {"strict": True, "noEmit": True, "skipLibCheck": False,
            "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler",
            "allowImportingTsExtensions": True, "types": ["bun", "node"],
            "typeRoots": [str(root / "node_modules/@types")], "baseUrl": str(root),
            "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]},
            "lib": ["ES2023", "DOM", "DOM.Iterable"]},
        "files": [str(stage / "ts" / (p + ".ts")) for p in ["development", "release"]]}
    (stage / "tsconfig.json").write_text(json.dumps(config))
    run(["node", root / "node_modules/typescript/bin/tsc", "-p", stage / "tsconfig.json"])
    run(["bun", "build", stage / "ts/release.ts", "--target=node", "--format=esm",
         "--outfile", stage / "module.mjs"])
    metadata = json.loads((stage / "native/rust-backend.json").read_text())
    symbol = "v_" + (metadata["module"] + "::kernel").encode().hex()
    generated = (stage / "native/program.rs").read_text()
    main_offset = generated.rfind("\nfn main()")
    assert main_offset > 1000
    prefix = generated[:main_offset]
    binaries = {}
    for name, harness in [("timing", TIMING), ("allocation", ALLOCATOR)]:
        rust = stage / (name + ".rs")
        rust.write_text(prefix + harness.replace("KERNEL", symbol))
        binary = stage / (name + (".exe" if os.name == "nt" else ""))
        run(["rustc", "--edition=2021", "-O", rust, "-o", binary])
        binaries[name] = binary
    (stage / "measure.mjs").write_text(JS)
    allocation = json.loads(run([binaries["allocation"]], env=native_env))
    commands = {"native": [binaries["timing"]], "bun": ["bun", stage / "measure.mjs"],
                "node": ["node", stage / "measure.mjs"]}
    if args.compare_with:
        baseline = Path(args.compare_with).resolve()
        previous = json.loads((baseline / "report.json").read_text())
        assert previous["sourceSha256"] == digest(source)
        baseline_binary = baseline / ("timing.exe" if os.name == "nt" else "timing")
        assert previous["timingBinarySha256"] == digest(baseline_binary)
        commands = {"baseline-native": [baseline_binary], **commands}
    names = list(commands)
    results = {name: [] for name in names}
    rounds = len(names) * 2  # Each variant occupies every order position twice.
    for iteration in range(rounds):
        order = iteration % len(names)
        for name in names[order:] + names[:order]:
            env = native_env if "native" in name else dict(os.environ)
            result = json.loads(run(commands[name], env=env))
            assert result["checksum"] == 620506872831 and len(result["samplesNs"]) == 5
            if "native" in name:
                assert result["managedLive"] == 0
            results[name].append(result)
    report = {"scope": "ordinary generated Char/Maybe kernel, 1114113 inputs per call",
        "sourceSha256": digest(source), "generatedRustSha256": digest(stage / "native/program.rs"),
        "typescriptReleaseSha256": digest(stage / "ts/release.ts"),
        "typescriptDevelopmentSha256": digest(stage / "ts/development.ts"),
        "timingBinarySha256": digest(binaries["timing"]),
        "compilerIdentity": json.loads(run([cli, "--version-json"])),
        "host": platform.platform(), "machine": platform.machine(),
        "runtimes": {p: run([p, "--version"]).strip() for p in ["rustc", "bun", "node"]},
        "protocol": {"freshProcessesPerVariant": rounds, "samplesPerProcess": 5,
            "warmup": ">=3 calls and >=1 second", "order": "balanced rotation",
            "excluded": ["compile", "module load", "startup", "checksum assertion"],
            "allocatorCounterUsedForTiming": False},
        "raw": results, "allocation": allocation,
        "medianMsPerDomain": {name: statistics.median(x / 1e6 for row in rows for x in row["samplesNs"])
                              for name, rows in results.items()},
        "typescriptStrictBothProfiles": True,
        "tsPhysicalHeapMeasured": False, "peakRssMeasured": False}
    if args.compare_with:
        report["baselineReportSha256"] = digest(Path(args.compare_with) / "report.json")
    (stage / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"medianMsPerDomain": report["medianMsPerDomain"], "allocation": allocation,
                      "sourceSha256": report["sourceSha256"]}), flush=True)


if __name__ == "__main__":
    main()
