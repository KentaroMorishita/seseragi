import assert from "node:assert/strict"
import { copyFileSync, mkdtempSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import { publishedDiagnostics, publishedExecutions } from "./published-examples"

const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/release/seseragi")
const temporary = mkdtempSync(join(tmpdir(), "seseragi-published-examples-"))
type CompilerDiagnostic = {
  code: string
  severity: string
  message: string
  labels: { message: string }[]
}

try {
  for (const example of publishedExecutions) {
    // A file inside a package can resolve to that package's main. Copy outside
    // the package so we execute the exact source shown in the article.
    const path = join(temporary, `${example.id}.ssrg`)
    copyFileSync(join(root, example.sourcePath), path)
    const result = Bun.spawnSync([cli, "run", path], {
      cwd: root,
      timeout: 90_000,
      stdout: "pipe",
      stderr: "pipe",
    })
    assert.equal(result.exitCode, 0, `${example.id}: ${result.stderr}`)
    assert.equal(result.stdout.toString(), example.output, example.id)
  }
  for (const example of publishedDiagnostics) {
    const path = join(temporary, `${example.id}.ssrg`)
    copyFileSync(join(root, example.sourcePath), path)
    const result = Bun.spawnSync(
      [cli, "lint", path, "--diagnostic-format", "json"],
      {
        cwd: root,
        timeout: 90_000,
        stdout: "pipe",
        stderr: "pipe",
      }
    )
    assert.notEqual(
      result.exitCode,
      0,
      `${example.id}: invalid example accepted`
    )
    // Failed lint writes the structured diagnostic report to stderr.
    const report = JSON.parse(result.stderr.toString()) as {
      diagnostics: { diagnostics: { diagnostics: CompilerDiagnostic[] } }[]
    }
    const diagnostics = report.diagnostics.flatMap(
      (file) => file.diagnostics.diagnostics
    )
    const diagnostic = diagnostics.find(
      (value) => value.code === example.code && value.severity === "Error"
    )
    assert.ok(diagnostic, `${example.id}: ${example.code} missing`)
    if (example.message)
      assert.equal(diagnostic.message, example.message, example.id)
    if (example.detail)
      assert.ok(
        diagnostic.labels.some((label) => label.message === example.detail),
        `${example.id}: diagnostic detail differs`
      )
  }
  console.info(
    `${publishedExecutions.length} published outputs and ${publishedDiagnostics.length} diagnostics verified`
  )
} finally {
  rmSync(temporary, { recursive: true, force: true })
}
