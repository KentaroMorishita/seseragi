"""Opt-in source -> checked CoreIr -> Rust parity and lifetime evidence."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[4]


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def command(args, env=None, expected=0):
    result = subprocess.run([str(a) for a in args], cwd=ROOT, env=env,
                            capture_output=True, text=True, timeout=90)
    if result.returncode != expected:
        raise RuntimeError(f"{args}: exit {result.returncode}\n{result.stdout}\n{result.stderr}")
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--emitter", required=True)
    parser.add_argument("--cli", required=True)
    parser.add_argument("--bun", required=True)
    parser.add_argument("--rustc", default="rustc")
    args = parser.parse_args()
    emitter, cli, bun = [Path(shutil.which(p) or p).resolve() for p in (args.emitter, args.cli, args.bun)]
    env = dict(os.environ, PATH=f"{bun.parent}{os.pathsep}{os.environ['PATH']}")
    output = ROOT / "target/native-lifetimes"
    output.mkdir(exist_ok=True)
    report = {"productionIntegrated": False, "sourceCases": [],
              "tools": {"rustc": command([args.rustc, "--version"]).stdout.strip(),
                        "bun": command([bun, "--version"]).stdout.strip()},
              "implementationHashes": {str(p.relative_to(ROOT)): digest(p)
                  for p in sorted([*Path(__file__).parent.glob("*.rs"), Path(__file__),
                                   Path(__file__).parent.parent / "native_lifetimes.rs"])}}
    cases = [("lesson-18", ROOT / "examples/spec/lessons/18-resource-scope.ssrg", "released: True\n")]
    cases += [(name, ROOT / f"examples/spec/fixtures/rust-backend-research/{name}.ssrg", expected)
              for name, expected in [("ref-cycle", "answer: 42\n"),
                                     ("cold-effect", "before: 0\nafter: 2\n"),
                                     ("nested-scope", "order: 3241\n"),
                                     ("escaped-callback", "escaped: 42\n"),
                                     ("failure-scope", "failure released: 42\n")]]
    for name, source, expected in cases:
        rust = output / f"{name}.rs"
        binary = output / (name + (".exe" if os.name == "nt" else ""))
        core = output / f"{name}.core.json"
        command([emitter, "dump", source, core])
        command([emitter, "emit", source, rust])
        command([args.rustc, "--edition=2021", "-O", rust, "-o", binary])
        native = command([binary], env=dict(os.environ, PATH=""))
        candidate = command([binary], env=dict(os.environ, PATH="", SESERAGI_RESEARCH_GC="candidates"))
        ts = command([cli, "run", source], env=env)
        assert native.stdout == candidate.stdout == ts.stdout == expected, name
        assert re.search(r'live=0 registry=0 checkpoints=\d+ reclaimed=\d+', native.stderr), native.stderr
        assert re.search(r'live=0 registry=0 checkpoints=\d+ reclaimed=\d+', candidate.stderr), candidate.stderr
        if name == "ref-cycle":
            assert int(re.search(r'reclaimed=(\d+)', native.stderr)[1]) >= 2, "cycle must need tracing"
        report["sourceCases"].append({"name": name, "sourceHash": digest(source),
                                      "coreHash": digest(core), "rustHash": digest(rust),
                                      "stdout": native.stdout, "lifetime": native.stderr.strip(),
                                      "candidateLifetime": candidate.stderr.strip()})
    source = ROOT / "examples/spec/fixtures/rust-backend-research/release-callback-timing.ssrg"
    rust = output / "release-callback-timing.rs"
    binary = output / ("release-callback-timing.exe" if os.name == "nt" else "release-callback-timing")
    command([emitter, "emit", source, rust])
    command([args.rustc, "--edition=2021", "-O", rust, "-o", binary])
    native = command([binary], env=dict(os.environ, PATH=""), expected=1)
    candidate = command([binary], env=dict(os.environ, PATH="", SESERAGI_RESEARCH_GC="candidates"), expected=1)
    ts = command([cli, "run", source], env=env, expected=70)
    assert native.stdout == candidate.stdout == ts.stdout == "body before release defect\n"
    assert 'Defect(["Seseragi Int overflow"])' in native.stderr and "live=0 registry=0" in native.stderr
    assert 'Defect(["Seseragi Int overflow"])' in candidate.stderr and "live=0 registry=0" in candidate.stderr
    assert "runtime defect" in ts.stderr
    report["sourceCases"].append({"name": "release-callback-timing", "sourceHash": digest(source),
                                 "rustHash": digest(rust), "stdout": native.stdout,
                                 "exit": "defect", "lifetime": native.stderr.strip(),
                                 "candidateLifetime": candidate.stderr.strip()})
    model = json.loads(command([emitter]).stdout)
    oracle = Path(__file__).with_name("oracle.ts")
    ts_traces = json.loads(command([bun, oracle]).stdout)
    assert model["resourceTraces"] == ts_traces, (model["resourceTraces"], ts_traces)
    report["model"] = model
    report["tsRuntimeTraceHash"] = digest(oracle)
    report["resourceTraceParity"] = len(ts_traces)
    # Compile only the experiment runtime with rustc -O. The debug frontend
    # runner's measurements above are NOT production performance evidence.
    benchmark_source = (output / "lesson-18.rs").read_text().rsplit("\nfn main()", 1)[0]
    benchmark_source += r'''
fn main() {
    for case in ["garbage-heavy", "mostly-live"] {
        for candidates in [false, true] {
            for round in 0..3 {
                let h = heap::Heap::default();
                let mut live = h.int(42);
                if case == "mostly-live" {
                    for _ in 0..100_000 { live = h.sequence(vec![live]); }
                }
                let cycles = if case == "garbage-heavy" { 10_000 } else { 1_000 };
                for _ in 0..cycles {
                    let cell = h.reference(h.int(0));
                    let callback = h.function(vec![cell.clone()], |_, n| n);
                    cell.write(callback);
                }
                let start = std::time::Instant::now();
                let result = if candidates { h.collect_candidates() } else { h.collect() }.unwrap();
                let nanos = start.elapsed().as_nanos();
                assert_eq!(result.reclaimed, cycles * 2);
                assert_eq!(result.scanned, if candidates { cycles * 2 } else { cycles * 2 + if case == "mostly-live" { 100_001 } else { 1 } });
                println!("{case},{candidates},{round},{},{},{nanos}", result.scanned, result.reclaimed);
                drop(live);
                assert_eq!(h.live(), 0);
            }
        }
    }
}
'''
    benchmark_rust = output / "collector-benchmark.rs"
    benchmark_binary = output / ("collector-benchmark.exe" if os.name == "nt" else "collector-benchmark")
    benchmark_rust.write_text(benchmark_source)
    command([args.rustc, "--edition=2021", "-O", benchmark_rust, "-o", benchmark_binary])
    report["optimizedCollectorPauses"] = []
    for line in command([benchmark_binary], env=dict(os.environ, PATH="")).stdout.splitlines():
        case, candidates, round_, scanned, reclaimed, nanos = line.split(",")
        report["optimizedCollectorPauses"].append({"case": case, "candidates": candidates == "true",
            "round": int(round_), "scanned": int(scanned), "reclaimed": int(reclaimed), "nanoseconds": int(nanos)})
    # Check target rejection before artifact creation, after the same frontend.
    for name, source in [("type-error", 'pub fn bad -> Int = "wrong"\n'),
                         ("unsupported", 'import * as effects from "std/effect"\npub effect fn main = effects.yieldNow ()\n')]:
        input_path = output / f"{name}.ssrg"
        input_path.write_text(source)
        rejected = output / f"{name}.rs"
        rejected.unlink(missing_ok=True)
        result = command([emitter, "emit", input_path, rejected], expected=1)
        assert not rejected.exists()
        assert ("SES-T0101" if name == "type-error" else "research operation unsupported") in result.stderr
        report[name] = result.stderr.strip()
    destination = output / "report.json"
    destination.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(f"{len(report['sourceCases'])} checked source cases, {len(ts_traces)} resource traces: {destination}")


if __name__ == "__main__":
    main()
