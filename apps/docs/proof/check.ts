import assert from "node:assert/strict"
import { createHash } from "node:crypto"
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { dirname, join, resolve } from "node:path"

// Bounded two-page protocol for the phase B proof; not the content pipeline.
const source = resolve(import.meta.dir, "generator.ssrg")
const temporary = mkdtempSync(join(tmpdir(), "seseragi-o03-ssg-proof-"))
const cli = process.env.SESERAGI_BIN ?? "seseragi"
const pages = [
  ["index.html", "Seseragi Docs", "Hello &lt;Seseragi&gt; &amp; friends"],
  ["language/index.html", "Language Guide", "Values &amp; expressions"],
] as const

function run(command: string[], cwd = temporary): string {
  const result = Bun.spawnSync(command, { cwd, stdout: "pipe", stderr: "pipe" })
  assert.equal(result.exitCode, 0, result.stderr.toString())
  return result.stdout.toString()
}

try {
  run([cli, "format", "--check", source])
  let baseline: string | undefined
  let releaseIdentity: string | undefined
  for (const profile of ["development", "release", "release"] as const) {
    const output = join(
      temporary,
      `build-${profile}-${releaseIdentity ? "repeat" : "first"}`
    )
    run([
      cli,
      "build",
      source,
      "--target",
      "process",
      "--profile",
      profile,
      "--source-map",
      "omit",
      "--out-dir",
      output,
    ])
    const manifest = JSON.parse(
      readFileSync(join(output, "artifact-manifest.json"), "utf8")
    )
    assert.equal(manifest.target, "process")
    assert.equal(manifest.profile, profile)
    assert.equal(manifest.sourceMap.policy, "omit")
    if (profile === "release") {
      if (releaseIdentity)
        assert.equal(manifest.provenance.buildId, releaseIdentity)
      releaseIdentity = manifest.provenance.buildId
    }
    const rendered = run(["bun", manifest.entry], output)
    assert.equal(run(["bun", manifest.entry], output), rendered)
    if (baseline !== undefined) assert.equal(rendered, baseline)
    baseline = rendered
    const lines = rendered.trimEnd().split("\n")
    assert.equal(lines.length, pages.length)
    const inventory = pages.map(([path, title, text], index) => {
      const expected = `<!doctype html><html lang="en"><head><title>${title}</title></head><body><main><h1>${title}</h1><p>${text}</p></main></body></html>`
      assert.equal(lines[index], expected)
      assert.ok(!lines[index].includes("<script"))
      const destination = join(output, "site", path)
      mkdirSync(dirname(destination), { recursive: true })
      writeFileSync(destination, lines[index])
      return {
        path,
        bytes: Buffer.byteLength(lines[index]),
        sha256: createHash("sha256")
          .update(readFileSync(destination))
          .digest("hex"),
      }
    })
    // Website inventory is deliberately separate from the CLI generator artifact.
    writeFileSync(
      join(output, "site", "pages.json"),
      JSON.stringify(inventory, null, 2)
    )
    console.log(
      JSON.stringify({
        profile,
        generator: manifest.provenance,
        generatorSizes: manifest.sizes,
        pages: inventory,
        clientJavascriptBytes: 0,
      })
    )
  }
} finally {
  rmSync(temporary, { recursive: true, force: true })
}
