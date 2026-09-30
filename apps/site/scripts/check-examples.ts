import assert from "node:assert/strict"
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  rmSync,
} from "node:fs"
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

  // These are runnable snippets, not merely modules that happen to typecheck.
  // Keep each article's displayed output tied to actual CLI execution.
  for (const [slug, expected] of [
    ["principle-expression-oriented", "pass"],
    ["principle-immutable-by-default", "10 -> 11"],
    ["principle-no-hidden-danger", "not found"],
    ["principle-backend-independent-semantics", "-3, 3.5"],
    ["principle-diagnosable-behavior", "`[]"],
    ["principle-visible-costs", "[2, 4, 6]"],
    ["principle-readable-density", "7"],
    ["reader-method-calls", "10\n15"],
    ["reader-pipelines", "7\n7\n7"],
    ["reader-records", "10\n42\nAki"],
    ["reader-structs", "Aki\nMio\n1"],
    ["reader-collections", "answer\n42\nJust 20\nNothing\n`[Ren, Aki, Mio]"],
    ["reader-trait", "ticket-42"],
  ]) {
    const source = join(examples, "src/language", `${slug}.ssrg`)
    // run <file> inside a package uses that package's declared entry point.
    // Isolate this snippet as main so its effectful entry is actually executed.
    const snippet = join(temporary, slug)
    mkdirSync(join(snippet, "src"), { recursive: true })
    copyFileSync(
      join(examples, "seseragi.toml"),
      join(snippet, "seseragi.toml")
    )
    copyFileSync(source, join(snippet, "src/main.ssrg"))
    const lock = Bun.spawnSync([cli, "lock", "update", snippet], {
      cwd: root,
      stdout: "pipe",
      stderr: "pipe",
    })
    assert.equal(lock.exitCode, 0, lock.stderr.toString())
    const result = Bun.spawnSync([cli, "run", snippet], {
      cwd: root,
      stdout: "pipe",
      stderr: "pipe",
    })
    assert.equal(result.exitCode, 0, `${slug}: ${result.stderr}`)
    assert.equal(result.stdout.toString(), `${expected}\n`, slug)
  }

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
