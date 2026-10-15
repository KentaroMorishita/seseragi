"""Build a checked pure program WASM and a separately checked Seseragi web UI."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[4]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--emitter", required=True)
    parser.add_argument("--cli", required=True)
    parser.add_argument("--bun", required=True)
    parser.add_argument("--rustc", default="rustc")
    args = parser.parse_args()
    emitter, cli, bun = [Path(shutil.which(p) or p).resolve() for p in (args.emitter, args.cli, args.bun)]
    out = ROOT / "target/native-wasm"
    out.mkdir(exist_ok=True)
    env = dict(os.environ, PATH=f"{bun.parent}{os.pathsep}{os.environ['PATH']}")

    def run(command, expected=0):
        result = subprocess.run([str(p) for p in command], cwd=ROOT, env=env,
                                capture_output=True, text=True, timeout=90)
        if result.returncode != expected:
            raise RuntimeError(f"{command}\n{result.stdout}\n{result.stderr}")
        return result.stdout

    source = ROOT / "examples/spec/fixtures/rust-backend-research/wasm-compute.ssrg"
    invalid = out / "invalid-projection.ssrg"
    invalid.write_text("pub fn probe -> Int = 42\n")
    rejected = out / "invalid-projection.rs"
    rejected.unlink(missing_ok=True)
    run([emitter, "emit-wasm", invalid, rejected], expected=1)
    assert not rejected.exists(), "invalid typed projection must not emit a module"
    run([emitter, "emit-wasm", source, out / "compute.rs"])
    run([args.rustc, "--edition=2021", "--target=wasm32-unknown-unknown", "--crate-type=cdylib",
         "-O", "-C", "strip=symbols", out / "compute.rs", "-o", out / "compute.wasm"])
    run([bun, "build", Path(__file__).with_name("wasm-bridge.ts"), "--target=browser",
         "--outfile", out / "wasm-bridge.js"])
    source_dir = out / "ui-source"
    source_dir.mkdir(exist_ok=True)
    shutil.copyfile(ROOT / "examples/spec/fixtures/rust-backend-research/web-ui.ssrg", source_dir / "main.ssrg")
    shutil.copyfile(Path(__file__).with_name("wasm-host.ts"), source_dir / "wasm-host.ts")
    run([cli, "build", source_dir / "main.ssrg", "--target", "web", "--out-dir", out / "ui"])
    manifest = json.loads((out / "ui/artifact-manifest.json").read_text())
    entry = manifest["entry"]
    assert not Path(entry).is_absolute() and ".." not in Path(entry).parts
    (out / "package.json").write_text('{"type":"module"}\n')
    (out / "index.html").write_text('''<!doctype html><html lang="ja"><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1"><title>Seseragi WASM research</title>
<link rel="stylesheet" href="ui/assets/app.css"><style>body{font:18px system-ui;margin:3rem auto;padding:1rem;max-width:36rem}button{font:inherit;padding:.5rem 1rem;margin-right:1rem}</style>
<div id="app">WASMを読み込み中…</div><script type="module">
import {loadCompute} from './wasm-bridge.js';
globalThis.seseragiCompute=await loadCompute(await (await fetch('./compute.wasm')).arrayBuffer());
await import(@@UI_ENTRY@@);
</script></html>''')
    launcher = out / "index.html"
    launcher.write_text(launcher.read_text().replace("@@UI_ENTRY@@", json.dumps("./ui/" + entry)))
    probe = out / "verify.mjs"
    probe.write_text('''import fs from 'node:fs';
import assert from 'node:assert/strict';
import {loadCompute,WasmDefect} from './wasm-bridge.js';
const bytes=fs.readFileSync(new URL('./compute.wasm',import.meta.url));
const module=await WebAssembly.compile(bytes);
assert.deepEqual(WebAssembly.Module.imports(module),[]);
const compute=await loadCompute(bytes);
for(let n=-500;n<500;n++) assert.equal(compute(n),n*n+7);
assert.throws(()=>compute(1.5),TypeError);
assert.equal(compute(2),11);
assert.throws(()=>compute(Number.MAX_SAFE_INTEGER),WasmDefect);
assert.throws(()=>compute(2),/stopped after a defect/);
console.log(JSON.stringify({imports:[],calls:1000,invalidInput:'rejected',defect:'instance retired'}));
''')
    result = json.loads(run(["node", probe]))
    # Use the TS target on the SAME calculation source as an additional oracle.
    oracle = out / "oracle.ssrg"
    oracle.write_text(source.read_text() + '\npub effect fn main = println $ `values: ${probe () 2}, ${probe () 3}`\n')
    assert run([cli, "run", oracle]) == "values: 11, 16\n"
    result.update({"wasmBytes": (out / "compute.wasm").stat().st_size,
                   "wasmSha256": hashlib.sha256((out / "compute.wasm").read_bytes()).hexdigest(),
                   "sourceSha256": hashlib.sha256(source.read_bytes()).hexdigest(),
                   "ui": "Seseragi source -> existing web/TS target", "productionIntegrated": False})
    (out / "report.json").write_text(json.dumps(result, indent=2) + "\n")
    print(f"program WASM + Seseragi web UI verified: {out}")


if __name__ == "__main__":
    main()
