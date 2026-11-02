"""Compile measurement entrypoints with this checkout's native runtime source."""
from pathlib import Path
import re
import subprocess


def rustc(arguments):
    result = subprocess.run(["rustc", *map(str, arguments)], capture_output=True,
                            text=True, timeout=120)
    if result.returncode:
        raise RuntimeError(result.stderr)


def compile_native(source, executable):
    source = Path(source)
    program = source.read_text()
    if "use seseragi_native_runtime::*;" in program:
        manifest = (Path(__file__).resolve().parents[2]
                    / "src/native_runtime_source.rs")
        runtime = "".join((manifest.parent / name).read_text() for name in
                          re.findall(r'include_str!\("([^"]+)"\)', manifest.read_text()))
        program = program.replace("use seseragi_native_runtime::*;", runtime)
        program = program.replace(
            "use std::{cell::{Cell, RefCell}, collections::BTreeMap};\n", "")
        source.write_text(program)
    rustc(["--edition=2021", "-O", source, "-o", executable])
