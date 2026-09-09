import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs"
import { dirname, join, resolve } from "node:path"
import { type Budget, validateArtifact } from "./production-artifacts"

const root = resolve(import.meta.dir, "..")
const budgets: Budget[] = JSON.parse(
  readFileSync(
    join(root, "examples/spec/fixtures/production/budgets.json"),
    "utf8"
  )
)
const output = resolve(
  process.env.SESERAGI_PRODUCTION_OUTPUT ??
    join(root, "target/production-artifacts")
)
const cli = resolve(
  process.env.SESERAGI_BIN ??
    join(process.env.CARGO_TARGET_DIR ?? join(root, "target"), "debug/seseragi")
)
function run(args: string[]) {
  const result = Bun.spawnSync([cli, ...args], {
    cwd: root,
    stdout: "pipe",
    stderr: "pipe",
  })
  if (result.exitCode !== 0) throw new Error(result.stderr.toString())
}
mkdirSync(output, { recursive: true })
const report = []
const failures: string[] = []
for (const budget of budgets) {
  const input = join(output, ".inputs", budget.id)
  try {
    rmSync(input, { recursive: true, force: true })
    const source = join(root, budget.source)
    let entry = input
    if (source.endsWith(".ssrg")) {
      mkdirSync(input, { recursive: true })
      entry = join(input, "main.ssrg")
      cpSync(source, entry)
    } else {
      cpSync(source, input, { recursive: true })
      run(["lock", "update", input])
    }
    const artifact = join(output, budget.id)
    run([
      "build",
      entry,
      "--target",
      budget.target,
      "--profile",
      "release",
      "--source-map",
      "omit",
      "--out-dir",
      artifact,
    ])
    const manifest = validateArtifact(artifact, budget)
    // A second build at another output location must preserve full identity.
    const repeated = join(output, `${budget.id}-repeat`)
    run([
      "build",
      entry,
      "--target",
      budget.target,
      "--profile",
      "release",
      "--source-map",
      "omit",
      "--out-dir",
      repeated,
    ])
    const again = validateArtifact(repeated, budget)
    if (manifest.provenance.buildId !== again.provenance.buildId)
      throw new Error("non-deterministic artifact identity")
    rmSync(repeated, { recursive: true, force: true })
    if (budget.target === "process") {
      const result = Bun.spawnSync(["bun", manifest.entry], {
        cwd: artifact,
        stdout: "pipe",
        stderr: "pipe",
      })
      if (result.exitCode !== 0) throw new Error(result.stderr.toString())
      const development = join(output, `${budget.id}-development`)
      run([
        "build",
        entry,
        "--profile",
        "development",
        "--out-dir",
        development,
      ])
      const developmentManifest = JSON.parse(
        readFileSync(join(development, "artifact-manifest.json"), "utf8")
      )
      const developmentResult = Bun.spawnSync(
        ["bun", developmentManifest.entry],
        { cwd: development, stdout: "pipe", stderr: "pipe" }
      )
      if (
        developmentResult.exitCode !== result.exitCode ||
        !developmentResult.stdout.equals(result.stdout) ||
        !developmentResult.stderr.equals(result.stderr)
      )
        throw new Error("development/release observable behavior mismatch")
      rmSync(development, { recursive: true, force: true })
    }
    report.push({
      id: budget.id,
      source: budget.source,
      target: budget.target,
      buildId: manifest.provenance.buildId,
      sizes: manifest.sizes,
      sourceMap: manifest.sourceMap,
      modules: manifest.generatedModules.map((module) => module.module),
      runtime: manifest.runtimeRetention,
    })
    console.log(
      `${budget.id}: ${manifest.sizes.minifiedJavascriptBytes} bytes; shape/digests/determinism passed`
    )
  } catch (error) {
    failures.push(`${budget.id}: ${error}`)
  } finally {
    rmSync(input, { recursive: true, force: true })
  }
}
writeFileSync(
  join(output, "report.json"),
  `${JSON.stringify({ schema: 1, fixtures: report, failures }, null, 2)}\n`
)
rmSync(join(output, ".inputs"), { recursive: true, force: true })
if (failures.length) throw new Error(failures.join("\n"))
console.log(
  `Production artifacts: ${report.length} fixtures passed. Report: ${dirname(join(output, "report.json"))}/report.json`
)
