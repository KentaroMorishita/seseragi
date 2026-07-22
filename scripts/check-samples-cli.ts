import { readdir, readFile } from "node:fs/promises"
import { resolve } from "node:path"
import {
  parseLearningPaths,
  parseSampleMetadata,
  validateSampleCatalog,
} from "../apps/playground/src/sample-catalog"

const repositoryRoot = resolve(import.meta.dir, "..")
const samplesRoot = resolve(repositoryRoot, "examples/samples")
const entries = await readdir(samplesRoot, { withFileTypes: true })
const directories = entries
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .sort()

const samples = await Promise.all(
  directories.map(async (id) => {
    const directory = resolve(samplesRoot, id)
    const metadata = parseSampleMetadata(
      JSON.parse(await readFile(resolve(directory, "sample.json"), "utf8")),
      id
    )
    return { directory, metadata }
  })
)
const paths = parseLearningPaths(
  JSON.parse(
    await readFile(resolve(samplesRoot, "learning-paths.json"), "utf8")
  )
)
validateSampleCatalog(
  samples.map(({ metadata }) => metadata),
  paths
)

const build = Bun.spawn(["cargo", "build", "-q", "-p", "seseragi-cli"], {
  cwd: repositoryRoot,
  stdout: "inherit",
  stderr: "inherit",
})
if ((await build.exited) !== 0) throw new Error("failed to build seseragi CLI")

const executable = resolve(repositoryRoot, "target/debug/seseragi")
let checked = 0
for (const { directory, metadata } of samples) {
  if (metadata.interactive) continue
  const source = resolve(directory, metadata.files.source)
  const stdin = metadata.files.stdin
    ? await readFile(resolve(directory, metadata.files.stdin), "utf8")
    : ""
  const expected = metadata.files.expectedOutput
    ? await readFile(resolve(directory, metadata.files.expectedOutput), "utf8")
    : ""
  const run = Bun.spawn([executable, "run", source], {
    cwd: repositoryRoot,
    stdin: "pipe",
    stdout: "pipe",
    stderr: "pipe",
  })
  run.stdin.write(stdin)
  run.stdin.end()
  const [status, stdout, stderr] = await Promise.all([
    run.exited,
    new Response(run.stdout).text(),
    new Response(run.stderr).text(),
  ])
  if (status !== 0) {
    throw new Error(`sample ${metadata.id} failed in CLI:\n${stderr}`)
  }
  const normalizedStdout = stdout.replace(/\r?\n$/u, "")
  if (normalizedStdout !== expected) {
    throw new Error(
      `sample ${metadata.id} output mismatch\nexpected: ${JSON.stringify(expected)}\nactual: ${JSON.stringify(normalizedStdout)}`
    )
  }
  checked += 1
}

console.log(
  `Validated ${checked} executable samples with the native Seseragi CLI (${samples.length - checked} browser-interactive skipped).`
)
