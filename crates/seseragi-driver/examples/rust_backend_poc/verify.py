#!/usr/bin/env python3
"""Reproduce the native PoC and compare its pure results with the TS backend."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[4]
FIXTURES = ROOT / "examples/spec/fixtures/rust-backend-poc"


def run(args, *, env=None):
    result = subprocess.run(args, cwd=ROOT, env=env, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(f"{args!r}\n{result.stdout}\n{result.stderr}")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bun", default=shutil.which("bun"))
    parser.add_argument("--iterations", type=int, default=100_000)
    args = parser.parse_args()
    if args.iterations < 1:
        parser.error("--iterations must be positive")
    if not args.bun or run([args.bun, "--version"]).stdout.strip() != "1.3.11":
        parser.error("TS comparison requires Bun 1.3.11; pass --bun /absolute/path/to/bun")
    bun = Path(args.bun).resolve()
    ts_environment = dict(os.environ, PATH=f"{bun.parent}{os.pathsep}{os.environ.get('PATH', '')}")
    native_environment = dict(os.environ, PATH="")
    directory = ROOT / "target/rust-backend-poc"
    directory.mkdir(parents=True, exist_ok=True)
    run(["cargo", "build", "--locked", "-p", "seseragi-driver", "--example", "rust_backend_poc"])
    run(["cargo", "build", "--locked", "-p", "seseragi-cli", "--bin", "seseragi"])
    tests = run(["cargo", "test", "--locked", "-p", "seseragi-driver", "--example", "rust_backend_poc", "--", "--nocapture"])
    (directory / "poc-tests.log").write_text(tests.stdout + tests.stderr)
    generator = ROOT / "target/debug/examples/rust_backend_poc"
    cli = ROOT / "target/debug/seseragi"
    report = {
        "schema": 1,
        "source_commit": run(["git", "rev-parse", "HEAD"]).stdout.strip(),
        "scope": "experimental type-erased single-thread pure CoreIr to Rust emitter",
        "rustc": run(["rustc", "--version"]).stdout.strip(),
        "bun_comparison_version": "1.3.11",
        "native_execution_path": "",
        "verification_log": "poc-tests.log",
        "implementation_sha256": {
            str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted([Path(__file__).parent.parent / "rust_backend_poc.rs",
                                *Path(__file__).parent.glob("*")])
            if path.is_file()
        },
        "cases": [],
    }
    for fixture in sorted(FIXTURES.glob("*.ssrg")):
        run([str(cli), "format", "--check", str(fixture)])
        case = directory / fixture.stem
        case.mkdir(exist_ok=True)
        core = run([str(generator), "dump", str(fixture)]).stdout
        (case / "core.json").write_text(core)
        generated = case / "generated.rs"
        native = case / "native"
        run([str(generator), "emit", str(fixture), str(generated), str(args.iterations)])
        started = time.monotonic()
        run(["rustc", "--edition=2021", "-O", "-C", "debuginfo=0", str(generated), "-o", str(native)])
        compile_ms = round((time.monotonic() - started) * 1000, 2)
        result = run([str(native)], env=native_environment)
        # The source entry is pure. Console/show is added only to the TS oracle.
        baseline = case / "baseline.ssrg"
        baseline.write_text(fixture.read_text() + "\npub effect fn main = println (show (probe ()))\n")
        reference = run([str(cli), "run", str(baseline)], env=ts_environment)
        if result.stdout != reference.stdout:
            raise RuntimeError(f"{fixture.name}: Rust={result.stdout!r}, TS={reference.stdout!r}")
        metrics = dict(item.split("=", 1) for item in result.stderr.strip().split()[1:])
        assert int(metrics["live"]) == 0
        record = {
            "fixture": str(fixture.relative_to(ROOT)),
            "source_sha256": hashlib.sha256(fixture.read_bytes()).hexdigest(),
            "generated_sha256": hashlib.sha256(generated.read_bytes()).hexdigest(),
            "stdout": result.stdout,
            "ts_stdout": reference.stdout,
            "metrics": {key: int(value) for key, value in metrics.items()},
            "native_bytes": native.stat().st_size,
            "rustc_compile_ms": compile_ms,
        }
        report["cases"].append(record)
        print(f"PASS {fixture.stem}: {result.stdout.strip()} | {result.stderr.strip()}", flush=True)
    # An isolated child may abort: observe the recursive destructor's limit.
    # This model does not claim to compile a deep Seseragi program.
    depth = directory / "drop-depth"
    depth.mkdir(exist_ok=True)
    depth_binary = depth / "native"
    run(["rustc", "--edition=2021", "-O", str(Path(__file__).parent / "drop_depth.rs"), "-o", str(depth_binary)])
    drop_result = subprocess.run([str(depth_binary)], env=native_environment,
                                 text=True, capture_output=True, timeout=10)
    category = "completed" if drop_result.returncode == 0 else "stack-overflow"
    if drop_result.returncode != 0 and "stack overflow" not in drop_result.stderr:
        raise RuntimeError(f"Unexpected drop-depth failure: {drop_result.stderr}")
    (depth / "stderr.txt").write_text(drop_result.stderr)
    report["recursive_drop_model"] = {
        "nodes": 20_000,
        "stack_bytes": 256 * 1024,
        "exit_code": drop_result.returncode,
        "category": category,
        "stderr": drop_result.stderr,
    }
    print(f"OBSERVED recursive-drop model: {category}")
    (directory / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(f"Report: {directory / 'report.json'}")


if __name__ == "__main__":
    main()
