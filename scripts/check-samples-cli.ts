import {
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
  writeFile,
} from "node:fs/promises"
import { tmpdir } from "node:os"
import { dirname, join, resolve } from "node:path"
import {
  parseDiscoverGroups,
  parseSampleMetadata,
  validateSampleCatalog,
} from "../apps/playground/src/sample-catalog"
import { loadValidatedTourCurriculum } from "./tour-curriculum"
import { repositoryPath } from "./tour-lessons"

const repositoryRoot = resolve(import.meta.dir, "..")
const updateTourDiagnostics = process.argv.includes("--update-tour-diagnostics")
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
const discoverGroups = parseDiscoverGroups(
  JSON.parse(
    await readFile(resolve(samplesRoot, "discover-groups.json"), "utf8")
  )
)
validateSampleCatalog(
  samples.map(({ metadata }) => metadata),
  discoverGroups
)
const { lessons: tourLessons } =
  await loadValidatedTourCurriculum(repositoryRoot)
const cargoTargetDirectory = resolve(
  repositoryRoot,
  process.env.CARGO_TARGET_DIR ?? "target"
)

const build = Bun.spawn(["cargo", "build", "-q", "-p", "seseragi-cli"], {
  cwd: repositoryRoot,
  stdout: "inherit",
  stderr: "inherit",
})
if ((await build.exited) !== 0) throw new Error("failed to build seseragi CLI")

const executable = resolve(cargoTargetDirectory, "debug/seseragi")
let checked = 0
for (const { directory, metadata } of samples) {
  if (metadata.interactive) continue
  const source = resolve(directory, metadata.files.source)
  const temporaryPackage =
    metadata.workspace === undefined
      ? undefined
      : await createTemporarySamplePackage(directory, metadata)
  const runTarget = temporaryPackage ?? source
  const stdin = metadata.files.stdin
    ? await readFile(resolve(directory, metadata.files.stdin), "utf8")
    : ""
  const expected = metadata.files.expectedOutput
    ? await readFile(resolve(directory, metadata.files.expectedOutput), "utf8")
    : ""
  try {
    const run = Bun.spawn([executable, "run", runTarget], {
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
    const normalizedExpected = expected.replace(/\r?\n$/u, "")
    const normalizedStdout = stdout.replace(/\r?\n$/u, "")
    if (normalizedStdout !== normalizedExpected) {
      throw new Error(
        `sample ${metadata.id} output mismatch\nexpected: ${JSON.stringify(normalizedExpected)}\nactual: ${JSON.stringify(normalizedStdout)}`
      )
    }
  } finally {
    if (temporaryPackage !== undefined) {
      await rm(temporaryPackage, { recursive: true, force: true })
    }
  }
  checked += 1
}

let checkedTourLessons = 0
let checkedTourExercises = 0
let checkedTourDiagnostics = 0
for (const lesson of tourLessons) {
  if (!lesson.metadata.interactive) {
    if (
      lesson.expectedOutputPath === undefined &&
      lesson.expectedFailurePath === undefined
    ) {
      throw new Error(
        `Tour lesson ${lesson.metadata.id} has no expected result`
      )
    }
    const formatCheck = Bun.spawn(
      [executable, "format", "--check", lesson.sourcePath],
      {
        cwd: repositoryRoot,
        stdout: "pipe",
        stderr: "pipe",
      }
    )
    const [formatStatus, formatStdout, formatStderr] = await Promise.all([
      formatCheck.exited,
      new Response(formatCheck.stdout).text(),
      new Response(formatCheck.stderr).text(),
    ])
    if (formatStatus !== 0) {
      throw new Error(
        `Tour lesson ${lesson.metadata.id} is not formatted:\n${formatStdout}${formatStderr}`
      )
    }
    const stdin = lesson.stdinPath
      ? await readFile(lesson.stdinPath, "utf8")
      : ""
    const expected = lesson.expectedOutputPath
      ? await readFile(lesson.expectedOutputPath, "utf8")
      : ""
    const run = Bun.spawn([executable, "run", lesson.sourcePath], {
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
    const expectedFailure = lesson.expectedFailurePath
      ? await readFile(lesson.expectedFailurePath, "utf8")
      : undefined
    const normalizedExpected = expected.replace(/\r?\n$/u, "")
    const normalizedStdout = stdout.replace(/\r?\n$/u, "")
    const normalizedStderr = stderr.replace(/\r?\n$/u, "")
    if (expectedFailure === undefined && status !== 0) {
      throw new Error(
        `Tour lesson ${lesson.metadata.id} failed in CLI:\n${stderr}`
      )
    }
    if (expectedFailure !== undefined && status === 0) {
      throw new Error(
        `Tour lesson ${lesson.metadata.id} succeeded instead of failing`
      )
    }
    if (normalizedStdout !== normalizedExpected) {
      throw new Error(
        `Tour lesson ${lesson.metadata.id} output mismatch\nexpected: ${JSON.stringify(normalizedExpected)}\nactual: ${JSON.stringify(normalizedStdout)}`
      )
    }
    if (
      expectedFailure !== undefined &&
      normalizedStderr !== expectedFailure.replace(/\r?\n$/u, "")
    ) {
      throw new Error(
        `Tour lesson ${lesson.metadata.id} failure mismatch\nexpected: ${JSON.stringify(expectedFailure.replace(/\r?\n$/u, ""))}\nactual: ${JSON.stringify(normalizedStderr)}`
      )
    }
    checkedTourLessons += 1
  }

  if (lesson.metadata.format === undefined) continue
  if (
    lesson.exercisePath === undefined ||
    lesson.exerciseExpectedOutputPath === undefined ||
    lesson.diagnosticExamplePath === undefined ||
    lesson.diagnosticOutputPath === undefined
  ) {
    throw new Error(
      `Structured Tour lesson ${lesson.metadata.id} is missing exercise or diagnostic files`
    )
  }
  const exerciseFormat = Bun.spawn(
    [executable, "format", "--check", lesson.exercisePath],
    {
      cwd: repositoryRoot,
      stdout: "pipe",
      stderr: "pipe",
    }
  )
  const [exerciseFormatStatus, exerciseFormatStdout, exerciseFormatStderr] =
    await Promise.all([
      exerciseFormat.exited,
      new Response(exerciseFormat.stdout).text(),
      new Response(exerciseFormat.stderr).text(),
    ])
  if (exerciseFormatStatus !== 0) {
    throw new Error(
      `Tour lesson ${lesson.metadata.id} exercise is not formatted:\n${exerciseFormatStdout}${exerciseFormatStderr}`
    )
  }
  const exerciseRun = Bun.spawn([executable, "run", lesson.exercisePath], {
    cwd: repositoryRoot,
    stdin: "pipe",
    stdout: "pipe",
    stderr: "pipe",
  })
  exerciseRun.stdin.end()
  const [exerciseStatus, exerciseStdout, exerciseStderr] = await Promise.all([
    exerciseRun.exited,
    new Response(exerciseRun.stdout).text(),
    new Response(exerciseRun.stderr).text(),
  ])
  if (exerciseStatus !== 0) {
    throw new Error(
      `Tour lesson ${lesson.metadata.id} exercise failed in CLI:\n${exerciseStderr}`
    )
  }
  const expectedExercise = (
    await readFile(lesson.exerciseExpectedOutputPath, "utf8")
  ).replace(/\r?\n$/u, "")
  if (exerciseStdout.replace(/\r?\n$/u, "") !== expectedExercise) {
    throw new Error(
      `Tour lesson ${lesson.metadata.id} exercise output mismatch\nexpected: ${JSON.stringify(expectedExercise)}\nactual: ${JSON.stringify(exerciseStdout.replace(/\r?\n$/u, ""))}`
    )
  }
  checkedTourExercises += 1

  const diagnosticRun = Bun.spawn(
    [
      executable,
      "run",
      repositoryPath(repositoryRoot, lesson.diagnosticExamplePath),
    ],
    {
      cwd: repositoryRoot,
      stdin: "pipe",
      stdout: "pipe",
      stderr: "pipe",
    }
  )
  diagnosticRun.stdin.end()
  const [diagnosticStatus, diagnosticStdout, diagnosticStderr] =
    await Promise.all([
      diagnosticRun.exited,
      new Response(diagnosticRun.stdout).text(),
      new Response(diagnosticRun.stderr).text(),
    ])
  if (diagnosticStatus === 0 || diagnosticStderr.trim() === "") {
    throw new Error(
      `Tour lesson ${lesson.metadata.id} diagnostic example did not fail`
    )
  }
  if (diagnosticStdout !== "") {
    throw new Error(
      `Tour lesson ${lesson.metadata.id} diagnostic example wrote stdout`
    )
  }
  if (updateTourDiagnostics) {
    await writeFile(lesson.diagnosticOutputPath, diagnosticStderr)
  } else {
    const expectedDiagnostic = await readFile(
      lesson.diagnosticOutputPath,
      "utf8"
    )
    if (diagnosticStderr !== expectedDiagnostic) {
      throw new Error(
        `Tour lesson ${lesson.metadata.id} diagnostic snapshot is stale; run \`bun run tour:diagnostics:update\``
      )
    }
  }
  checkedTourDiagnostics += 1
}

console.log(
  `Validated ${checked} executable samples, ${checkedTourLessons} Tour lessons, ${checkedTourExercises} exercises and ${checkedTourDiagnostics} Tour diagnostics with the native Seseragi CLI (${samples.length - checked + tourLessons.length - checkedTourLessons} browser-interactive skipped).`
)

async function createTemporarySamplePackage(
  sampleDirectory: string,
  metadata: (typeof samples)[number]["metadata"]
): Promise<string> {
  const workspace = metadata.workspace
  if (workspace === undefined) {
    throw new Error(`sample ${metadata.id} has no project workspace`)
  }
  const packageDirectory = await mkdtemp(
    join(tmpdir(), `seseragi-sample-${metadata.id}-`)
  )
  const sourceRoot = resolve(packageDirectory, "src")
  try {
    for (const path of workspace.files) {
      const target = resolve(sourceRoot, path)
      await mkdir(dirname(target), { recursive: true })
      await writeFile(
        target,
        await readFile(resolve(sampleDirectory, path), "utf8")
      )
    }
    const entry = workspace.entry.replace(/\.ssrg$/u, "")
    await writeFile(
      resolve(packageDirectory, "seseragi.toml"),
      [
        "[package]",
        `name = "sample/${metadata.id}"`,
        'version = "0.0.0"',
        'language = ">=0.1.0 <0.2.0"',
        "",
        "[run]",
        `entry = ${JSON.stringify(entry)}`,
        'target = "test-js"',
        "",
      ].join("\n")
    )
    return packageDirectory
  } catch (error) {
    await rm(packageDirectory, { recursive: true, force: true })
    throw error
  }
}
