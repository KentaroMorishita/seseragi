"""Compare one checked pure kernel, excluding compilation and module loading.

This deliberately measures only scalar self tail recursion, not application or
Effect performance. Run with an otherwise idle local host and a built CLI.
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
    result = subprocess.run(command, capture_output=True, text=True, **kwargs)
    if result.returncode:
        raise RuntimeError(f"{command!r}\n{result.stdout}\n{result.stderr}")
    return result.stdout


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", default="target/debug/seseragi")
    parser.add_argument("--bun", default="bun")
    parser.add_argument("--node", default="node")
    parser.add_argument("--typescript-emitter", default="target/debug/examples/core_ir_performance")
    parser.add_argument("--out", default="target/native-lifetimes/tail-performance")
    args = parser.parse_args()
    stage = Path(args.out).resolve()
    stage.mkdir(parents=True, exist_ok=True)
    source = stage / "kernel.ssrg"
    source.write_text('''fn sum remaining: Int -> total: Int -> Int =
  if remaining == 0 then total else sum (remaining - 1) (total + remaining)
pub fn kernel remaining: Int -> Int = sum remaining 0
pub fn probe -> Int = kernel 200000
pub effect fn main = println (show (probe ()))
''')
    cli = str(Path(args.cli).resolve())
    run([cli, "experimental", "build-native", str(source), "--entry", "probe",
         "--out-dir", str(stage / "native")])
    # The application package has an owned executable entry, not a library
    # export. Inspect the ordinary compiler stage through the existing emitter,
    # then bundle its unmodified module with Bun for both comparison runtimes.
    run([str(Path(args.typescript_emitter).resolve()), str(source), str(stage / "ts")])
    run([args.bun, "build", str(stage / "ts/release.ts"), "--target=node",
         "--format=esm", "--outfile", str(stage / "module.mjs")])
    metadata = json.loads((stage / "native/rust-backend.json").read_text())
    symbol = metadata["module"] + "::kernel"
    rust_id = "v_" + symbol.encode().hex()
    generated = (stage / "native/program.rs").read_text()
    expected = sum(n * (n + 1) // 2 for n in range(200000, 200020))
    harness = r'''
// Handwritten comparison only: this function is not compiler output.
fn scalar_control(mut remaining: i64) -> i64 {
    fn checked(value: i64) -> i64 {
        assert!((-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&value));
        value
    }
    let mut total = 0_i64;
    while remaining != 0 {
        let next = checked(remaining.checked_sub(1).unwrap());
        let value = checked(total.checked_add(remaining).unwrap());
        remaining = next;
        total = value;
    }
    total
}
fn main() {
    let kernel = <KERNEL>(vec![]);
    let control = std::env::args().any(|value| value == "--scalar-control");
    let allocations = ALLOCATIONS.with(Cell::get);
    let mut samples = Vec::new();
    let warmup = std::time::Instant::now();
    let mut warmups = 0;
    while samples.len() < 5 {
        let recording = warmups >= 3 && warmup.elapsed().as_secs_f64() >= 1.0;
        let now = std::time::Instant::now();
        let mut checksum = 0_i64;
        for i in 0..20 {
            let input = std::hint::black_box(200000 + i);
            checksum += if control { scalar_control(input) } else {
                match kernel.apply(V::Int(input)) {
                    V::Int(value) => value,
                    _ => panic!("expected Int"),
                }
            };
        }
        let elapsed = now.elapsed().as_nanos();
        assert_eq!(checksum, <CHECKSUM>);
        if recording { samples.push(elapsed); } else { warmups += 1; }
    }
    drop(kernel);
    clear_dictionaries();
    assert_eq!(LIVE.with(Cell::get), 0);
    let allocations = ALLOCATIONS.with(Cell::get) - allocations;
    println!("{{\"samplesNs\":{:?},\"checksum\":{},\"managedAllocations\":{},\"warmups\":{}}}", samples, <CHECKSUM>_i64, allocations, warmups);
}
'''.replace("<KERNEL>", rust_id).replace("<CHECKSUM>", str(expected))
    rust_source = stage / "measure.rs"
    rust_source.write_text(generated[:generated.rfind("fn main()")] + harness)
    native = stage / ("measure.exe" if os.name == "nt" else "measure")
    run(["rustc", "--edition=2021", "-O", str(rust_source), "-o", str(native)])
    js = stage / "measure.mjs"
    js.write_text('''import { kernel } from "./module.mjs";
const samplesNs = [];
const warmup = process.hrtime.bigint();
let warmups = 0;
while (samplesNs.length < 5) {
  const recording = warmups >= 3 && process.hrtime.bigint() - warmup >= 1000000000n;
  const start = process.hrtime.bigint();
  let checksum = 0;
  for (let i = 0; i < 20; i++) checksum += kernel(200000 + i);
  const ns = Number(process.hrtime.bigint() - start);
  if (checksum !== <CHECKSUM>) throw Error(`checksum ${checksum}`);
  if (recording) samplesNs.push(ns); else warmups++;
}
console.log(JSON.stringify({ samplesNs, checksum: <CHECKSUM>, warmups }));
'''.replace("<CHECKSUM>", str(expected)))
    commands = {"native": [str(native)], "bun": [args.bun, str(js)],
                "node": [args.node, str(js)], "native-i64-control": [str(native), "--scalar-control"]}
    results = {name: [] for name in commands}
    names = list(commands)
    for process in range(4):
        # Rotate order; every runtime gets four independent fresh processes.
        for name in names[process:] + names[:process]:
            env = dict(os.environ)
            if name.startswith("native"):
                env["PATH"] = ""
            result = json.loads(run(commands[name], env=env))
            assert result["checksum"] == expected
            assert len(result["samplesNs"]) == 5
            results[name].append(result)
    medians = {name: statistics.median(ns / 20 for row in rows for ns in row["samplesNs"])
               for name, rows in results.items()}
    report = {
        "scope": "scalar self tail recursion; 200000..200019 steps per call",
        "control": "native-i64-control is handwritten checked i64, not compiler output",
        "timing": "20 calls/sample; >=3 warmups and >=1s before 5 samples/process; 4 fresh processes/runtime",
        "excluded": ["source compilation", "module loading", "process startup", "verification"],
        "host": platform.platform(), "machine": platform.machine(),
        "inputSha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        "nativeSourceSha256": hashlib.sha256(generated.encode()).hexdigest(),
        "typescriptSha256": hashlib.sha256((stage / "ts/release.ts").read_bytes()).hexdigest(),
        "bundledModuleSha256": hashlib.sha256((stage / "module.mjs").read_bytes()).hexdigest(),
        "bun": run([args.bun, "--version"]).strip(),
        "node": run([args.node, "--version"]).strip(),
        "rustc": run(["rustc", "--version"]).strip(),
        "checksum": expected, "medianNsPerCall": medians, "processes": results,
        "nativeMetadata": metadata,
    }
    (stage / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"report": str(stage / "report.json"), "medianNsPerCall": medians}))


if __name__ == "__main__":
    main()
