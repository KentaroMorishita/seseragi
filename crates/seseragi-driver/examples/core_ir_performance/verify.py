#!/usr/bin/env python3
"""Measure generated TS, closed dictionary sharing, and native PoC construction costs."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import statistics
import subprocess
import sys

sys.dont_write_bytecode = True
from analyze_core import analyze, self_check

ROOT = Path(__file__).resolve().parents[4]
FIXTURES = ROOT / "examples/spec/fixtures/core-ir-performance"
OUTPUT = ROOT / "target/core-ir-performance"


def run(command, *, env=None):
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(f"{command!r}\n{result.stdout}\n{result.stderr}")
    return result


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bun", default=shutil.which("bun"))
    parser.add_argument("--processes", type=int, default=3)
    args = parser.parse_args()
    if args.processes < 3:
        parser.error("at least 3 fresh processes are required")
    if not args.bun or run([args.bun, "--version"]).stdout.strip() != "1.3.11":
        parser.error("pass --bun /absolute/path/to/bun-1.3.11")
    bun = str(Path(args.bun).resolve())
    self_check()
    OUTPUT.mkdir(parents=True, exist_ok=True)
    base = run(["git", "rev-parse", "HEAD"]).stdout.strip()
    implementation = [ROOT / "crates/seseragi-driver/examples/core_ir_performance.rs",
                      *Path(__file__).parent.glob("*.py"), *Path(__file__).parent.glob("*.ts"),
                      *FIXTURES.glob("*.ssrg")]
    identities = {str(path.relative_to(ROOT)): digest(path) for path in sorted(implementation)}
    run(["cargo", "build", "--locked", "-p", "seseragi-driver", "--example", "core_ir_performance", "--example", "rust_backend_poc"])
    generator = ROOT / "target/debug/examples/core_ir_performance"
    rust_generator = ROOT / "target/debug/examples/rust_backend_poc"
    analyses = {}
    for fixture in sorted(FIXTURES.glob("*.ssrg")):
        directory = OUTPUT / fixture.stem
        run([str(generator), str(fixture), str(directory)])
        analysis = analyze(json.loads((directory / "release.core.json").read_text()))
        (directory / "analysis.json").write_text(json.dumps(analysis, indent=2) + "\n")
        analyses[fixture.stem] = analysis
    assert next(item for item in analyses["calls"]["lambdas"] if item["owner"].endswith("::captureOne"))["free_names"] == ["seed"]
    assert analyses["dictionaries"]["factory_dependencies"]["parameter-dependent"] >= 1
    processes = []
    for index in range(args.processes):
        print(f"Measuring fresh Bun process {index + 1}/{args.processes}...", flush=True)
        measured = run([bun, str(Path(__file__).parent / "measure.ts"), str(OUTPUT), str(index)])
        (OUTPUT / f"measurement-{index}.json").write_text(measured.stdout)
        processes.append(json.loads(measured.stdout))
    run([str(ROOT / "apps/playground/node_modules/.bin/tsc"), "--noEmit", "--skipLibCheck",
         "--module", "ESNext", "--moduleResolution", "Bundler", "--target", "ES2022",
         "--esModuleInterop", "--types", "bun", str(Path(__file__).parent / "measure.ts"),
         str(OUTPUT / "calls/release.ts"), str(OUTPUT / "calls/direct.ts"),
         str(OUTPUT / "dictionaries/release.ts"), str(OUTPUT / "dictionaries/shared.ts")])
    summaries = {}
    for name in processes[0]["samples_ns_per_op"]:
        samples = [value for process in processes for value in process["samples_ns_per_op"][name]]
        process_medians = [statistics.median(process["samples_ns_per_op"][name]) for process in processes]
        ordered = sorted(samples)
        summaries[name] = {"median_ns": statistics.median(samples),
                           "sample_min_ns": min(samples), "sample_max_ns": max(samples),
                           "q25_ns": ordered[len(ordered)//4], "q75_ns": ordered[len(ordered)*3//4],
                           "process_medians_ns": process_medians}
    for process in processes[1:]:
        assert process["emitted_constructions_per_op"] == processes[0]["emitted_constructions_per_op"]
        assert process["dictionary_sharing"] == processes[0]["dictionary_sharing"]
        assert process["saturated_direct_call"] == processes[0]["saturated_direct_call"]
    native = []
    native_cases = [("calls", name, f"{name} 42", "49") for name in ["saturated", "partial", "immediateLambda"]]
    native_cases.append(("calls", "captureOne-unused", "{ let callback = captureOne 42 [1, 2, 3]; callback () }", "42"))
    native_cases += [("dictionaries", name, f"{name} 42", "98" if name == "returned" else "59" if name == "other" else "49")
                     for name in ["concrete", "nested", "returned", "other"]]
    for fixture, name, expression, expected in native_cases:
        directory = OUTPUT / "native" / name
        directory.mkdir(parents=True, exist_ok=True)
        source = directory / "probe.ssrg"
        source.write_text((FIXTURES / f"{fixture}.ssrg").read_text() + f"\npub fn probe -> Int = {expression}\n")
        generated = directory / "generated.rs"
        binary = directory / "native"
        run([str(rust_generator), "emit", str(source), str(generated), "10000"])
        run(["rustc", "--edition=2021", "-O", "-C", "debuginfo=0", str(generated), "-o", str(binary)])
        measured = run([str(binary)], env=dict(os.environ, PATH=""))
        assert measured.stdout.strip() == expected
        metrics = {key: int(value) for key, value in (field.split("=", 1) for field in measured.stderr.split()[1:])}
        assert metrics["live"] == 0
        native.append({"case": name, "metrics": metrics,
                       "managed_nodes_per_entry": metrics["allocations"] / metrics["iterations"],
                       "scope": "current erased Rust PoC; constant-input counters, not TS/native timing comparison"})
    assert base == run(["git", "rev-parse", "HEAD"]).stdout.strip(), "checkout changed during measurement"
    for file, sha in identities.items():
        assert digest(ROOT / file) == sha, f"measurement source changed: {file}"
    report = {"schema": 1, "source_commit": base, "host": run(["uname", "-sm"]).stdout.strip(),
              "bun": "1.3.11", "implementation_sha256": identities,
              "raw_process_files": [f"measurement-{index}.json" for index in range(args.processes)],
              "summaries": summaries, "emitted_constructions_per_op": processes[0]["emitted_constructions_per_op"],
              "dictionary_sharing": processes[0]["dictionary_sharing"], "core_analyses": analyses,
              "saturated_direct_call": processes[0]["saturated_direct_call"],
              "native_poc": native,
              "profiles_identical": {name: (OUTPUT/name/"release.ts").read_bytes() == (OUTPUT/name/"development.ts").read_bytes()
                                     for name in analyses}}
    (OUTPUT / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    for name, summary in summaries.items():
        print(f"{name}: {summary['median_ns']:.2f} ns/op, process medians {summary['process_medians_ns']}")
    print(f"Report: {OUTPUT / 'report.json'}")


if __name__ == "__main__":
    main()
