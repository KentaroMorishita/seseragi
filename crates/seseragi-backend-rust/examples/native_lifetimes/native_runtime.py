"""Link measurement entrypoints to the native runtime used by this checkout."""
from functools import cache
from pathlib import Path
import subprocess


def rustc(arguments):
    result = subprocess.run(["rustc", *map(str, arguments)], capture_output=True,
                            text=True, timeout=120)
    if result.returncode:
        raise RuntimeError(result.stderr)


@cache
def runtime_library(directory):
    source = (Path(__file__).resolve().parents[4]
              / "crates/seseragi-native-runtime/src/lib.rs")
    library = directory / "libseseragi_native_runtime.rlib"
    rustc(["--edition=2021", "--crate-name=seseragi_native_runtime",
           "--crate-type=rlib", "-O", source, "-o", library])
    return library


def compile_native(source, executable):
    source = Path(source)
    arguments = ["--edition=2021", "-O", source, "-o", executable]
    if "use seseragi_native_runtime::*;" in source.read_text():
        arguments += ["--extern",
                      f"seseragi_native_runtime={runtime_library(source.parent)}"]
    rustc(arguments)
