import assert from "node:assert/strict"
import { copyFileSync, mkdtempSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import { publishedExecutions } from "./published-examples"

const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/release/seseragi")
const temporary = mkdtempSync(join(tmpdir(), "seseragi-published-examples-"))

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
  for (const id of [
    "pilot-function-application",
    "pilot-currying",
    "syntax-reader-pipelines",
  ]) {
    const path = join(temporary, `${id}-invalid.ssrg`)
    copyFileSync(
      join(root, `apps/site/examples/invalid/src/language/${id}.ssrg`),
      path
    )
    const result = Bun.spawnSync([cli, "lint", path], {
      cwd: root,
      timeout: 90_000,
      stdout: "pipe",
      stderr: "pipe",
    })
    assert.notEqual(result.exitCode, 0, `${id}: invalid example accepted`)
    assert.match(result.stderr.toString(), /SES-T0101/u, id)
  }
  console.info(
    `${publishedExecutions.length} published outputs and 3 diagnostics verified`
  )
} finally {
  rmSync(temporary, { recursive: true, force: true })
}
