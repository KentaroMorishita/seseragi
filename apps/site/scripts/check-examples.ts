import assert from "node:assert/strict"
import { mkdtempSync, readdirSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { basename, join, resolve } from "node:path"

const root = resolve(import.meta.dir, "../../..")
const cli = process.env.SESERAGI_BIN ?? join(root, "target/debug/seseragi")
const examples = join(root, "apps/site/examples")

function build(packagePath: string, output: string) {
  return Bun.spawnSync(
    [cli, "build", packagePath, "--target", "process", "--out-dir", output],
    {
      cwd: root,
      stdout: "pipe",
      stderr: "pipe",
    }
  )
}

const temporary = mkdtempSync(join(tmpdir(), "seseragi-site-examples-"))
try {
  const valid = build(examples, join(temporary, "valid"))
  assert.equal(
    valid.exitCode,
    0,
    `${valid.stderr.toString()}${valid.stdout.toString()}`
  )
  const execution = Bun.spawnSync([cli, "run", examples], {
    cwd: root,
    stdout: "pipe",
    stderr: "pipe",
  })
  assert.equal(
    execution.exitCode,
    0,
    `Compiled examples failed at runtime: ${execution.stderr.toString()}${execution.stdout.toString()}`
  )

  const invalid = build(join(examples, "invalid"), join(temporary, "invalid"))
  assert.notEqual(
    invalid.exitCode,
    0,
    "Rejected examples unexpectedly compiled"
  )
  const diagnostics = `${invalid.stderr.toString()}${invalid.stdout.toString()}`
  const expectedInvalidSources = readdirSync(
    join(examples, "invalid/src/language")
  )
    .filter((name) => name.endsWith(".ssrg"))
    .sort()
  for (const name of expectedInvalidSources) {
    assert.ok(
      diagnostics.includes(basename(name)),
      `Rejected example did not produce a diagnostic: ${name}`
    )
  }
  const invalidImport = build(
    join(examples, "invalid/import"),
    join(temporary, "invalid-import")
  )
  assert.notEqual(invalidImport.exitCode, 0)
  assert.ok(invalidImport.stderr.toString().includes("MissingExport"))
  const invalidVisibility = build(
    join(examples, "invalid/visibility"),
    join(temporary, "invalid-visibility")
  )
  assert.notEqual(invalidVisibility.exitCode, 0)
  assert.ok(invalidVisibility.stderr.toString().includes("PrivateExport"))
} finally {
  rmSync(temporary, { recursive: true, force: true })
}
