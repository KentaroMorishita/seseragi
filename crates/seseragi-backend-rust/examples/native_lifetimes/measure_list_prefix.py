"""Measure ordinary List prefix decisions at widths 1/4/16/32.

Only measurement entrypoints are attached. A baseline directory is a temporary
input to the final balanced comparison, not a rewritten program body.
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
    result = subprocess.run(list(map(str, command)), capture_output=True,
                            text=True, timeout=180, **kwargs)
    if result.returncode:
        raise RuntimeError(f"{command!r}\n{result.stdout}\n{result.stderr}")
    return result.stdout


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


WIDTHS = [1, 4, 16, 32]
JS = '''import * as kernels from "./module.mjs";
const result = {};
for (const width of [1,4,16,32]) {
  const fn = kernels[`kernel${width}`], expected=width*(width+1)/2*1000;
  const started=process.hrtime.bigint(), samplesNs=[]; let warmups=0;
  while (samplesNs.length<5) {
    const recording=warmups>=3 && process.hrtime.bigint()-started>=1000000000n;
    const before=process.hrtime.bigint(), value=fn(1000);
    const elapsed=Number(process.hrtime.bigint()-before);
    if(value!==expected) throw Error(`checksum ${width}: ${value}`);
    if(recording) samplesNs.push(elapsed); else warmups++;
  }
  result[width]={samplesNs,warmups,checksum:expected};
}
console.log(JSON.stringify(result));
'''
RUST = r'''
fn main() {
    let kernels = [KERNELS];
    let mut results=Vec::new();
    for (width, kernel) in [1_i64,4,16,32].into_iter().zip(kernels) {
        let expected=width*(width+1)/2*1000;
        let started=std::time::Instant::now();
        let mut samples=Vec::new(); let mut warmups=0;
        while samples.len()<5 {
            let recording=warmups>=3 && started.elapsed().as_secs_f64()>=1.0;
            let now=std::time::Instant::now();
            let value=std::hint::black_box(&kernel).apply(V::Int(std::hint::black_box(1000)));
            let elapsed=now.elapsed().as_nanos();
            assert!(matches!(value,V::Int(n) if n==expected));
            if recording {samples.push(elapsed)} else {warmups+=1;}
        }
        results.push(format!("\"{width}\":{{\"samplesNs\":{:?},\"warmups\":{warmups},\"checksum\":{expected}}}", samples));
    }
    clear_dictionaries(); clear_globals(); assert_eq!(LIVE.with(Cell::get),0);
    println!("{{{}}}",results.join(","));
}
'''


def main():
    parser = argparse.ArgumentParser()
    for name in ["cli", "typescript-emitter", "out"]:
        parser.add_argument("--" + name, required=True)
    parser.add_argument("--compare-with")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[4]
    source = root / "examples/spec/fixtures/rust-backend/list-prefix-kernel.ssrg"
    stage = Path(args.out).resolve()
    stage.mkdir(parents=True, exist_ok=False)
    cli, emitter = Path(args.cli).resolve(), Path(args.typescript_emitter).resolve()
    native_env = dict(os.environ, PATH="")
    run([cli, "experimental", "build-native", source, "--entry", "probe",
         "--out-dir", stage / "native"])
    assert run([stage / "native/program"], env=native_env).strip() == "675000"
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
    symbols = ["v_" + (metadata["module"] + f"::kernel{w}").encode().hex()
               + "(vec![])" for w in WIDTHS]
    generated = (stage / "native/program.rs").read_text()
    offset = generated.rfind("\nfn main()")
    assert offset > 1000
    (stage / "timing.rs").write_text(generated[:offset] + RUST.replace("KERNELS", ",".join(symbols)))
    run(["rustc", "--edition=2021", "-O", stage / "timing.rs", "-o", stage / "timing"])
    (stage / "measure.mjs").write_text(JS)
    commands = {"native": [stage / "timing"], "bun": ["bun", stage / "measure.mjs"],
                "node": ["node", stage / "measure.mjs"]}
    if args.compare_with:
        base = Path(args.compare_with).resolve()
        previous = json.loads((base / "report.json").read_text())
        assert previous["sourceSha256"] == digest(source)
        for name in ["timing", "module.mjs", "measure.mjs"]:
            assert previous["controlsSha256"][name] == digest(base / name)
        commands = {"baseline-native": [base / "timing"],
                    "baseline-bun": ["bun", base / "measure.mjs"],
                    "baseline-node": ["node", base / "measure.mjs"], **commands}
    names = list(commands)
    raw = {name: [] for name in names}
    for iteration in range(len(names) * 2):
        start = iteration % len(names)
        for name in names[start:] + names[:start]:
            data = json.loads(run(commands[name], env=native_env if "native" in name else None))
            for width in WIDTHS:
                row = data[str(width)]
                assert row["checksum"] == width*(width+1)//2*1000
                assert len(row["samplesNs"]) == 5 and row["warmups"] >= 3
            raw[name].append(data)
    report = {"sourceSha256": digest(source),
        "generatedRustSha256": digest(stage / "native/program.rs"),
        "typescriptSha256": {p: digest(stage / "ts" / (p + ".ts")) for p in ["development", "release"]},
        "controlsSha256": {p: digest(stage / p) for p in ["timing", "module.mjs", "measure.mjs"]},
        "compilerIdentity": json.loads(run([cli, "--version-json"])),
        "host": platform.platform(), "runtimes": {p: run([p, "--version"]).strip() for p in ["rustc", "bun", "node"]},
        "protocol": {"widths": WIDTHS, "callsPerSample": 1000,
            "freshProcessesPerVariant": len(names)*2, "samplesPerProcess": 5,
            "warmup": ">=3 samples and >=1 second per width", "order": "balanced rotation",
            "bodyRewritten": False, "tsPhysicalHeapMeasured": False,
            "excluded": ["compile", "module load", "process startup", "checksum assertions"]},
        "strictBothProfiles": True, "nativeManagedLiveAfterTeardown": 0, "raw": raw,
        "medianNsPerDecision": {name: {str(w): statistics.median(v for row in rows
            for v in row[str(w)]["samplesNs"])/1000 for w in WIDTHS} for name, rows in raw.items()}}
    (stage / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"medians": report["medianNsPerDecision"], "report": str(stage / "report.json")}))


if __name__ == "__main__":
    main()
