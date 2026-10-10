"""Verify the editorial contract's complete examples with the repository CLI."""

import argparse
import csv
import hashlib
import json
import re
import subprocess
from pathlib import Path

root = Path(__file__).resolve().parents[4]
base = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--cli", default="target/release/seseragi")
args = parser.parse_args()
cli = str((root / args.cli).resolve())


def execute(*arguments):
    result = subprocess.run(
        [cli, *map(str, arguments)],
        cwd=root,
        capture_output=True,
        text=True,
        timeout=120,
        check=True,
    )
    return result.stdout


contract = (base.parent / "editorial-contract.md").read_text()
snippets = re.findall(r"```seseragi\n(.*?)```", contract, re.DOTALL)
assert len(snippets) == 2, "Expected complete good and bad editorial examples"
evidence = {"cli": execute("--version").strip(), "examples": []}
for name, snippet in zip(("good", "bad"), snippets, strict=True):
    package = base / name
    source = package / "src/main.ssrg"
    assert snippet == source.read_text(), f"Displayed/source mismatch: {name}"
    execute("format", "--check", source)
    output = execute("run", package)
    assert output == "360\n", f"Unexpected actual output: {name}: {output!r}"
    evidence["examples"].append(
        {
            "source": str(source.relative_to(root)),
            "sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
            "stdout": output,
            "format": "canonical",
        }
    )

# This is a migration audit snapshot, never a build's route/identity source.
with (base.parent / "migration-inventory.tsv").open() as handle:
    rows = list(csv.DictReader(handle, delimiter="\t"))
assert len({row["route_en"] for row in rows}) == len(rows)
assert len({row["route_ja"] for row in rows}) == len(rows)
for row in rows:
    assert (root / row["source"]).is_file(), row
    expected_ja = "/ja/" if row["route_en"] == "/" else "/ja" + row["route_en"]
    assert row["route_ja"] == expected_ja, row
planned = set(
    re.findall(
        r"`(/[^`\s]+/)`", (base.parent / "reference-content-map.md").read_text()
    )
)
assert len(planned) == 351
assert planned <= {row["route_en"] for row in rows}
evidence["migration_routes"] = len(rows)
evidence["planned_reference_routes"] = len(planned)
print(json.dumps(evidence, ensure_ascii=False, indent=2))
