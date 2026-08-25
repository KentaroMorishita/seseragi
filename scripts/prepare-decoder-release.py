from pathlib import Path
import re
import subprocess

VERSION = "0.26.0"
BRANCH = "fix/sqlite-decoder-applicative"

cargo = Path("Cargo.toml")
source = cargo.read_text()
match = re.search(r'(?ms)^\[workspace\.package\].*?^version\s*=\s*"([^"]+)"', source)
if not match:
    raise SystemExit("cannot read workspace version")

if match.group(1) == VERSION:
    raise SystemExit(0)

subprocess.run(["bun", "run", "release:bump", "--", VERSION], check=True)

changelog = Path("CHANGELOG.md")
text = changelog.read_text()
entry = f'''## [{VERSION}] - 2026-08-25

- Replaced the official SQLite decoder `map2` surface with `Functor` and
  `Applicative` composition, so curried row constructors use `<$>` and `<*>`.
- Preserved canonical nominal identity for qualified namespace imports during
  generic type hydration, fixing imported Applicative dispatch for types such as
  `sqlite.Decoder<A>`.
- Aligned the TypeScript SQLite runtime with the opaque Decoder newtype while
  retaining compatibility with decoder artifacts generated before this change.

'''
if f"## [{VERSION}]" not in text:
    marker = "# Change Log\n\n"
    if not text.startswith(marker):
        raise SystemExit("unexpected CHANGELOG.md header")
    changelog.write_text(marker + entry + text[len(marker):])

subprocess.run(["cargo", "update", "-w"], check=True)
subprocess.run(["bun", "run", "build:playground:wasm"], check=True)
subprocess.run(["bun", "run", "release:check"], check=True)

subprocess.run(["git", "config", "user.name", "github-actions[bot]"], check=True)
subprocess.run(
    [
        "git",
        "config",
        "user.email",
        "41898282+github-actions[bot]@users.noreply.github.com",
    ],
    check=True,
)
subprocess.run(["git", "add", "-A"], check=True)
if subprocess.run(["git", "diff", "--cached", "--quiet"]).returncode == 0:
    raise SystemExit(0)
subprocess.run(["git", "commit", "-m", "release: prepare 0.26.0"], check=True)
subprocess.run(["git", "push", "origin", f"HEAD:{BRANCH}"], check=True)
