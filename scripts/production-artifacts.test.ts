import { expect, test } from "bun:test"
import { createHash } from "node:crypto"
import { mkdtempSync, rmSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"
import {
  type Budget,
  type Manifest,
  validateArtifact,
  validateShape,
} from "./production-artifacts"

const budget: Budget = {
  id: "test",
  source: "test",
  target: "process",
  maxMinifiedBytes: 100,
  requiredRuntime: ["hash"],
  forbiddenRuntime: ["postgres"],
  forbiddenDeclarations: ["dead"],
}
const hash = (text: string) => createHash("sha256").update(text).digest("hex")
function fixture(): Manifest {
  return {
    schema: 1,
    profile: "release",
    target: "process",
    entry: "entry.js",
    provenance: { buildId: "" },
    generatedModules: [{ module: "main", exports: ["main"] }],
    files: [{ path: "entry.js", bytes: 4, sha256: hash("true") }],
    sizes: { bundledJavascriptBytes: 10, minifiedJavascriptBytes: 4 },
    sourceMap: { policy: "omit" },
    runtimeRetention: [
      { module: "hash", reasons: ["startup-required-initializer"] },
    ],
    reachability: { retained: [{ module: "main", declaration: "main" }] },
    bundles: [{ path: "entry.js", entry: true, modules: ["main"] }],
  }
}
test("quality uses a budget rather than an exact byte snapshot", () => {
  const m = fixture()
  validateShape(m, budget)
  m.files[0].bytes = 50
  m.sizes.minifiedJavascriptBytes = 50
  expect(() => validateShape(m, budget)).not.toThrow()
  m.sizes.minifiedJavascriptBytes = 101
  expect(() => validateShape(m, budget)).toThrow("budget")
})
for (const [name, mutate, message] of [
  [
    "required initializer",
    (m: Manifest) => {
      m.runtimeRetention = []
    },
    "required initializer",
  ],
  [
    "unexpected runtime",
    (m: Manifest) => {
      m.runtimeRetention!.push({ module: "postgres", reasons: ["referenced"] })
    },
    "unexpected runtime",
  ],
  [
    "dead declaration",
    (m: Manifest) => {
      m.reachability!.retained.push({ module: "main", declaration: "dead" })
    },
    "dead declaration",
  ],
  [
    "dead export",
    (m: Manifest) => {
      m.generatedModules[0].exports.push("dead")
    },
    "dead export",
  ],
  [
    "unknown module",
    (m: Manifest) => {
      m.bundles![0].modules.push("dead-module")
    },
    "unknown source module",
  ],
  [
    "source map policy",
    (m: Manifest) => {
      m.sourceMap.policy = "emit"
    },
    "source-map policy",
  ],
  [
    "source map leakage",
    (m: Manifest) => {
      m.files.push({ path: "entry.js.map", bytes: 0, sha256: "" })
    },
    "unexpected source map",
  ],
] as const)
  test(`rejects ${name} regression`, () => {
    const m = fixture()
    mutate(m)
    expect(() => validateShape(m, budget)).toThrow(message)
  })
test("checks real file digest, complete inventory, and manifest identity", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-artifact-gate-"))
  try {
    const m = fixture()
    m.provenance.buildId = hash(JSON.stringify(m))
    const save = () =>
      writeFileSync(
        join(directory, "artifact-manifest.json"),
        JSON.stringify(m)
      )
    save()
    writeFileSync(join(directory, "entry.js"), "true")
    expect(validateArtifact(directory, budget).provenance.buildId).toBe(
      m.provenance.buildId
    )
    writeFileSync(join(directory, "entry.js"), "null")
    expect(() => validateArtifact(directory, budget)).toThrow("digest/size")
    writeFileSync(join(directory, "entry.js"), "true")
    writeFileSync(join(directory, "leaked.js"), "")
    expect(() => validateArtifact(directory, budget)).toThrow("inventory")
    rmSync(join(directory, "leaked.js"))
    m.provenance.buildId = "corrupted"
    save()
    expect(() => validateArtifact(directory, budget)).toThrow("identity")
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
