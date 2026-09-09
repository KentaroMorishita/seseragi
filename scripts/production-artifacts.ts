import { createHash } from "node:crypto"
import { lstatSync, readdirSync, readFileSync } from "node:fs"
import { join } from "node:path"

export type Budget = {
  id: string
  source: string
  target: "process" | "web"
  maxMinifiedBytes: number
  requiredRuntime: string[]
  forbiddenRuntime: string[]
  forbiddenDeclarations?: string[]
  allowedRuntime?: string[]
  allowedModules?: string[]
}
export type Manifest = {
  schema: number
  profile: string
  target: string
  entry: string
  provenance: { buildId: string }
  generatedModules: { module: string; exports: string[] }[]
  files: { path: string; bytes: number; sha256: string }[]
  sizes: {
    bundledJavascriptBytes: number | null
    minifiedJavascriptBytes: number | null
  }
  sourceMap: { policy: string; files?: string[] }
  runtimeRetention: { module: string; reasons: string[] }[] | null
  reachability: { retained: { module: string; declaration: string }[] } | null
  bundles: { path: string; entry: boolean; modules: string[] }[] | null
}
const digest = (bytes: string | Uint8Array) =>
  createHash("sha256").update(bytes).digest("hex")
function requireValue(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}
export function validateShape(manifest: Manifest, budget: Budget): void {
  requireValue(
    manifest.schema === 1 && manifest.profile === "release",
    "expected release schema 1"
  )
  requireValue(manifest.target === budget.target, "target mismatch")
  requireValue(
    manifest.sourceMap.policy === "omit",
    "source-map policy violation"
  )
  requireValue(
    manifest.files.every((file) => !file.path.endsWith(".map")),
    "unexpected source map"
  )
  const size = manifest.sizes.minifiedJavascriptBytes
  requireValue(
    typeof size === "number" && size > 0 && size <= budget.maxMinifiedBytes,
    "minified size budget exceeded or unavailable"
  )
  requireValue(
    typeof manifest.sizes.bundledJavascriptBytes === "number",
    "unminified evidence unavailable"
  )
  const entry = manifest.files.find((file) => file.path === manifest.entry)
  requireValue(
    entry?.bytes === size,
    "entry bytes do not match minified measurement"
  )
  requireValue(
    manifest.runtimeRetention !== null,
    "runtime retention evidence unavailable"
  )
  const runtime = new Set(manifest.runtimeRetention.map((item) => item.module))
  if (budget.allowedRuntime)
    requireValue(
      [...runtime].every((module) => budget.allowedRuntime!.includes(module)),
      "runtime outside first-party baseline"
    )
  if (budget.allowedModules)
    requireValue(
      manifest.generatedModules.every((module) =>
        budget.allowedModules!.includes(module.module)
      ),
      "source module outside first-party baseline"
    )
  for (const required of budget.requiredRuntime)
    requireValue(
      runtime.has(required),
      `required initializer/runtime missing: ${required}`
    )
  for (const forbidden of budget.forbiddenRuntime)
    requireValue(
      ![...runtime].some((item) => item.includes(forbidden)),
      `unexpected runtime: ${forbidden}`
    )
  requireValue(
    manifest.runtimeRetention.every((item) => item.reasons.length > 0),
    "runtime retention reason missing"
  )
  requireValue(
    manifest.reachability !== null,
    "compiler reachability evidence unavailable"
  )
  for (const name of budget.forbiddenDeclarations ?? []) {
    requireValue(
      !manifest.reachability.retained.some((item) => item.declaration === name),
      `dead declaration resurrected: ${name}`
    )
    requireValue(
      !manifest.generatedModules.some((item) => item.exports.includes(name)),
      `dead export resurrected: ${name}`
    )
  }
  const generated = new Set(
    manifest.generatedModules.map((item) => item.module)
  )
  requireValue(
    manifest.bundles?.some(
      (bundle) => bundle.entry && bundle.path === manifest.entry
    ),
    "bundle entry evidence missing"
  )
  for (const bundle of manifest.bundles) {
    requireValue(
      manifest.files.some((file) => file.path === bundle.path),
      "bundle file missing"
    )
    requireValue(
      bundle.modules.every((module) => generated.has(module)),
      "bundle resurrected unknown source module"
    )
  }
  requireValue(
    manifest.files.every(
      (file) =>
        !file.path.startsWith("node_modules/") &&
        !file.path.endsWith(".ts") &&
        !file.path.startsWith(".seseragi-bundle")
    ),
    "staging code leaked into product"
  )
}
export function validateArtifact(directory: string, budget: Budget): Manifest {
  const manifest: Manifest = JSON.parse(
    readFileSync(join(directory, "artifact-manifest.json"), "utf8")
  )
  validateShape(manifest, budget)
  const actual: string[] = []
  function visit(base: string, prefix = "") {
    for (const name of readdirSync(base)) {
      const path = join(base, name)
      const relative = prefix + name
      const stat = lstatSync(path)
      requireValue(!stat.isSymbolicLink(), "artifact symlink is not permitted")
      if (stat.isDirectory()) visit(path, `${relative}/`)
      else if (relative !== "artifact-manifest.json") actual.push(relative)
    }
  }
  visit(directory)
  requireValue(
    new Set(manifest.files.map((file) => file.path)).size ===
      manifest.files.length,
    "duplicate artifact file"
  )
  requireValue(
    JSON.stringify(actual.sort()) ===
      JSON.stringify(manifest.files.map((file) => file.path).sort()),
    "artifact inventory mismatch"
  )
  for (const file of manifest.files) {
    const bytes = readFileSync(join(directory, file.path))
    requireValue(
      bytes.length === file.bytes && digest(bytes) === file.sha256,
      `artifact digest/size mismatch: ${file.path}`
    )
  }
  const identity = structuredClone(manifest)
  identity.provenance.buildId = ""
  requireValue(
    digest(JSON.stringify(identity)) === manifest.provenance.buildId,
    "build identity mismatch"
  )
  return manifest
}
