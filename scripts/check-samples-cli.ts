// Runs the samples under examples/samples with the native CLI.
//
//   bun scripts/check-samples-cli.ts          # samples changed from the base
//   bun scripts/check-samples-cli.ts --all    # every sample
//
// A sample passes when it exits with 0. When it has an expected output file,
// its stdout must also match that file. Interactive samples cannot run in the
// CLI, so they are only compiled (or built, for a package).
//
// The base defaults to origin/main; set SESERAGI_SAMPLE_BASE to change it.
// When the changed files cannot be found, every sample runs.
//
// The metadata and catalog checks live in `check:sample-catalog`, and the Tour
// checks live in `check:tour`.

import { readdir, readFile } from "node:fs/promises"
import { resolve } from "node:path"
import { runSampleCommand, withTemporaryDirectory } from "./sample-cli-process"
import {
  readSampleRunPlan,
  type SampleRunPlan,
  selectChangedSamples,
} from "./sample-selection"

const repositoryRoot = resolve(import.meta.dir, "..")
const samplesRoot = resolve(repositoryRoot, "examples/samples")
const checkAll = process.argv.includes("--all")

const entries = await readdir(samplesRoot, { withFileTypes: true })
const sampleIds = entries
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .sort()

const selectedIds = checkAll
  ? sampleIds
  : await changedSampleIds(sampleIds).catch((error: unknown) => {
      console.log(
        `Could not list changed files, so every sample runs: ${String(error)}`
      )
      return sampleIds
    })

if (selectedIds.length === 0) {
  console.log("No sample changed; nothing to run.")
  process.exit(0)
}
console.log(
  checkAll || selectedIds.length === sampleIds.length
    ? `Running all ${selectedIds.length} samples.`
    : `Running ${selectedIds.length} changed sample(s): ${selectedIds.join(", ")}`
)

const samples = await Promise.all(
  selectedIds.map(async (id) => {
    const directory = resolve(samplesRoot, id)
    const metadataPath = resolve(directory, "sample.json")
    const text = await readFile(metadataPath, "utf8").catch(() => {
      throw new Error(`sample ${id} has no sample.json`)
    })
    let value: unknown
    try {
      value = JSON.parse(text)
    } catch (error) {
      throw new Error(`sample ${id}/sample.json is not valid JSON: ${error}`)
    }
    const plan = readSampleRunPlan(value, id)
    for (const file of [plan.source, plan.manifest, plan.stdin]) {
      if (file === undefined) continue
      await readFile(resolve(directory, file)).catch(() => {
        throw new Error(`sample ${id} names a missing file: ${file}`)
      })
    }
    return { directory, plan }
  })
)

const cargoTargetDirectory = resolve(
  repositoryRoot,
  process.env.CARGO_TARGET_DIR ?? "target"
)
const build = await runSampleCommand(
  ["cargo", "build", "-q", "-p", "seseragi-cli"],
  { cwd: repositoryRoot, label: "native CLI build", timeoutMs: 600_000 }
)
if (build.status !== 0) {
  throw new Error(`failed to build seseragi CLI:\n${build.stderr}`)
}
const executable = resolve(cargoTargetDirectory, "debug/seseragi")

let ran = 0
let compared = 0
let compiled = 0
let builtPackages = 0
for (const { directory, plan } of samples) {
  if (plan.interactive) {
    if (plan.manifest !== undefined) {
      await validateWebPackageBuild(directory, plan.id)
      builtPackages += 1
    } else {
      await validateInteractiveCompile(directory, plan)
      compiled += 1
    }
    continue
  }
  const outcome = await runSample(directory, plan)
  ran += 1
  if (outcome === "compared") compared += 1
}

console.log(
  `Checked ${samples.length} sample(s) with the native Seseragi CLI: ${ran} ran (${compared} compared with their expected output), ${compiled} interactive compiled, ${builtPackages} Web package(s) built.`
)

async function changedSampleIds(
  ids: readonly string[]
): Promise<readonly string[]> {
  const base = process.env.SESERAGI_SAMPLE_BASE ?? "origin/main"
  const mergeBase = (await git(["merge-base", base, "HEAD"])).trim()
  // Committed, staged and unstaged changes since the base, plus new files.
  // --no-renames lists both sides of a move, so the sample a file left runs.
  const changed = await git(["diff", "--name-only", "--no-renames", mergeBase])
  const untracked = await git([
    "ls-files",
    "--others",
    "--exclude-standard",
    "--",
    "examples/samples",
  ])
  const paths = `${changed}\n${untracked}`
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line !== "")
  return selectChangedSamples(paths, ids)
}

async function git(args: readonly string[]): Promise<string> {
  const result = await runSampleCommand(["git", ...args], {
    cwd: repositoryRoot,
    label: `git ${args[0]}`,
  })
  if (result.status !== 0) {
    throw new Error(`git ${args.join(" ")} failed: ${result.stderr.trim()}`)
  }
  return result.stdout
}

async function runSample(
  directory: string,
  plan: SampleRunPlan
): Promise<"ran" | "compared"> {
  const runTarget =
    plan.manifest === undefined
      ? resolve(directory, plan.source ?? "")
      : directory
  const stdin = plan.stdin
    ? await readFile(resolve(directory, plan.stdin), "utf8")
    : ""
  const { status, stdout, stderr } = await runSampleCommand(
    [executable, "run", runTarget],
    { cwd: repositoryRoot, label: `sample ${plan.id} run`, stdin }
  )
  if (status !== 0) {
    throw new Error(
      `sample ${plan.id} exited with ${status} in the CLI:\n${stderr}`
    )
  }
  if (plan.expectedOutput === undefined) return "ran"
  const expected = await readFile(
    resolve(directory, plan.expectedOutput),
    "utf8"
  ).catch(() => {
    throw new Error(
      `sample ${plan.id} names a missing file: ${plan.expectedOutput}`
    )
  })
  const normalizedExpected = expected.replace(/\r?\n$/u, "")
  const normalizedStdout = stdout.replace(/\r?\n$/u, "")
  if (normalizedStdout !== normalizedExpected) {
    throw new Error(
      `sample ${plan.id} output mismatch\nexpected: ${JSON.stringify(normalizedExpected)}\nactual: ${JSON.stringify(normalizedStdout)}`
    )
  }
  return "compared"
}

async function validateInteractiveCompile(
  directory: string,
  plan: SampleRunPlan
): Promise<void> {
  const source = resolve(directory, plan.source ?? "")
  const check = await runSampleCommand([executable, "lint", source], {
    cwd: repositoryRoot,
    label: `interactive sample ${plan.id} compile`,
  })
  if (check.status !== 0) {
    throw new Error(
      `interactive sample ${plan.id} failed to compile:\n${check.stdout}${check.stderr}`
    )
  }
}

async function validateWebPackageBuild(
  packageDirectory: string,
  id: string
): Promise<void> {
  await withTemporaryDirectory(`seseragi-web-${id}-`, async (outputRoot) => {
    const outputDirectory = resolve(outputRoot, "dist")
    const result = await runSampleCommand(
      [executable, "build", packageDirectory, "--out-dir", outputDirectory],
      { cwd: repositoryRoot, label: `Web sample ${id} build` }
    )
    if (result.status !== 0) {
      throw new Error(
        `Web sample ${id} failed to build:\n${result.stdout}${result.stderr}`
      )
    }
    await Promise.all([
      readFile(resolve(outputDirectory, "index.html")),
      readFile(resolve(outputDirectory, "assets/app.js")),
    ])
  })
}
