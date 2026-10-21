"""Explicit sample assembly until program assets join the package build graph."""
import argparse
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def run(command, cwd):
    child = subprocess.Popen([str(x) for x in command], cwd=cwd,
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                             start_new_session=os.name != "nt")
    try:
        stdout, stderr = child.communicate(timeout=120)
        if child.returncode:
            raise RuntimeError(f"{command}\n{stdout.decode(errors='replace')}\n{stderr.decode(errors='replace')}")
        return stdout
    finally:
        if child.poll() is None:
            if os.name == "nt":
                # The owned CLI may still have a compiler descendant.
                with subprocess.Popen(["taskkill", "/F", "/T", "/PID", str(child.pid)],
                                      stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) as terminator:
                    try:
                        terminator.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        terminator.kill()
                        terminator.wait()
            else:
                os.killpg(child.pid, signal.SIGTERM)
                # Keep the group leader unreaped until descendants are stopped.
                import time
                time.sleep(0.25)
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
            child.kill()
        child.wait()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    cli = Path(shutil.which(args.cli) or args.cli).resolve()
    output = args.out.resolve()
    output.mkdir(parents=True, exist_ok=False)
    source = output / "source"
    shutil.copytree(ROOT / "examples/spec/fixtures/rust-backend-projects/program-wasm-ui", source)
    command = [cli, "experimental", "build-wasm", ROOT / "examples/spec/fixtures/rust-backend/program-wasm.ssrg"]
    for name in ["label", "validate", "enabled", "optional", "transport", "staged", "ready", "deep"]:
        command += ["--export", name]
    command += ["--out-dir", source / "program"]
    run(command, output)
    site = output / "site"
    run([cli, "build", source / "main.ssrg", "--target", "web", "--out-dir", site / "ui"], output)
    shutil.copytree(source / "program", site / "program")
    manifest = json.loads((site / "ui/artifact-manifest.json").read_text())
    entry = Path(manifest["entry"])
    if entry.is_absolute() or ".." in entry.parts:
        raise RuntimeError("invalid generated UI entry")
    (site / "index.html").write_text('''<!doctype html><html lang="ja"><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1"><title>Seseragi program WASM</title>
<link rel="stylesheet" href="ui/assets/app.css"><style>body{font:18px system-ui;margin:2rem auto;padding:1rem;max-width:48rem}button{font:inherit;padding:.5rem;margin:.25rem}pre{white-space:pre-wrap}</style>
<div id="app">program WASMを読み込み中…</div><script type="module" src="ui/''' + entry.as_posix() + '''"></script></html>''')
    print(json.dumps({"site": str(site), "programMetadata": str(site / "program/program-wasm.json"),
                      "assembly": "explicit sample assembly; package build/watch asset integration pending"}))


if __name__ == "__main__":
    main()
