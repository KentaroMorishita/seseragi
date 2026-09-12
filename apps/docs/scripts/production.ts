import assert from "node:assert/strict"
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  renameSync,
  rmSync,
  statSync,
} from "node:fs"
import { dirname, join, relative, resolve } from "node:path"
import { build } from "./build"

function files(root: string, current = root): string[] {
  return readdirSync(current)
    .flatMap((name) => {
      const path = join(current, name)
      return statSync(path).isDirectory()
        ? files(root, path)
        : relative(root, path)
    })
    .sort()
}

const args = process.argv.slice(2)
assert.equal(
  args.length,
  3,
  "Usage: bun apps/docs/scripts/production.ts OUTPUT ORIGIN BASE"
)
const [output, origin, base] = args
const destination = resolve(output)
assert.ok(!existsSync(destination), `Production output exists: ${destination}`)
mkdirSync(dirname(destination), { recursive: true })
const temporary = mkdtempSync(join(dirname(destination), ".docs-production-"))
try {
  const first = join(temporary, "first")
  const second = join(temporary, "second")
  const manifest = build({ out: first, origin, base, profile: "release" })
  const repeated = build({ out: second, origin, base, profile: "release" })
  assert.deepEqual(repeated, manifest, "Production build is not reproducible")
  assert.equal(
    readFileSync(join(first, "site-manifest.json"), "utf8"),
    readFileSync(join(second, "site-manifest.json"), "utf8"),
    "Site manifest is not reproducible"
  )
  const expected = [
    ...manifest.files.map(({ path }) => path),
    "site-manifest.json",
  ].sort()
  assert.deepEqual(
    files(first),
    expected,
    "Production root contains untracked files"
  )
  assert.deepEqual(
    files(second),
    expected,
    "Repeated root contains untracked files"
  )
  renameSync(first, destination)
  console.log(
    JSON.stringify(
      {
        output: destination,
        pages: manifest.pages.length,
        files: manifest.files.length,
        publishedBytes: manifest.quality.publishedBytes,
        clientJavascriptBytes: manifest.clientJavascriptBytes,
        generatorBuildId: manifest.generator.provenance.buildId,
        searchBuildId: manifest.search.artifact.provenance.buildId,
      },
      null,
      2
    )
  )
} finally {
  rmSync(temporary, { recursive: true, force: true })
}
