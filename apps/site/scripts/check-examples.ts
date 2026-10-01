import assert from "node:assert/strict"
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { basename, join, resolve } from "node:path"
import { checkComparisons } from "./check-comparisons"

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
  checkComparisons(cli, root, temporary)
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
    ["pilot-built-in-types", "Notebook: 3, 2.5, True"],
    ["pilot-annotations", "22"],
    ["pilot-function-application", "3"],
    ["pilot-blocks", "65"],
    ["pilot-currying", "3, 3"],
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

  // Check the actual values claimed by the article explanations.
  // Build each source as its own entry so another package main cannot mask it.
  for (const [index, [slug, expression, expected, effectful]] of (
    [
      ["syntax-layout", "result", "7"],
      ["syntax-layout", "grouped", "7"],
      ["function-application", "answer", "3"],
      ["polymorphism", "enabled", "True"],
      ["polymorphism", "answer", "42"],
      ["expressions-evaluation", "result", "23"],
      ["expressions-conditionals", "label", "allowed"],
      ["expressions-blocks", "result", "score=42"],
      ["expressions-lambdas", "answer", "42"],
      ["patterns-match", "result", "3 complete"],
      ["patterns-irrefutable", "rawId", "42"],
      ["patterns-binding-rules", "score", "42"],
      ["traits-constraints", "description", "badge: ready"],
      ["types-generic-functions", "number", "42"],
      ["types-generic-functions", "text", "hello"],
      ["types-generic-functions", "explicit", "`[]"],
      ["types-generic-aliases", "answer", "42"],
      ["types-generic-structs", "text.value", "42"],
      ["types-generic-methods", "text.value", "hello"],
      ["types-generic-methods", "number.get", "42"],
      ["effects-maybe", "displayName Nothing Nothing", "anonymous"],
      [
        "effects-maybe",
        'displayName (Just "cached") (Just "requested")',
        "cached",
      ],
      [
        "effects-either",
        "match increment (positive 1) { Right value -> value; Left (NotPositive value) -> value }",
        "2",
      ],
      [
        "effects-either",
        "match positive 0 { Right value -> value; Left (NotPositive value) -> value }",
        "0",
      ],
      ["effects-derived-signals", "stableTotal ()", "30", true],
      ["effects-signal-operators", "readAndWrite ()", "2", true],
      ["effects-cancellation-resources", "useScoped ()", "True", true],
      ["effects-signal-subscription", "observeThenRelease ()", "2", true],
      ["effects-fiber-supervision", "supervisedChild ()", "42", true],
      ["effects-scheduler-fairness", "cooperativeStep ()", "42", true],
      ["effects-task", "cached 42", "42", true],
    ] as Array<[string, string, string, boolean?]>
  ).entries()) {
    const snippet = join(temporary, `explanation-${index}`)
    mkdirSync(join(snippet, "src"), { recursive: true })
    copyFileSync(
      join(examples, "seseragi.toml"),
      join(snippet, "seseragi.toml")
    )
    const source = readFileSync(
      join(examples, "src/language", `${slug}.ssrg`),
      "utf8"
    )
    const main = effectful
      ? `pub effect fn main = do { value <- ${expression}; println (show value) }\n`
      : `pub effect fn main = println (show (${expression}))\n`
    writeFileSync(join(snippet, "src/main.ssrg"), `${source}\n${main}`)
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
    assert.equal(
      result.stdout.toString(),
      `${expected}\n`,
      `${slug}: ${expression}`
    )
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
