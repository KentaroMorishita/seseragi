// Checks the Tour lessons and the formatting of the spec examples with the
// native CLI.
//
//   bun scripts/check-tour-cli.ts
//   bun scripts/check-tour-cli.ts --update-tour-diagnostics
//
// Each lesson that is not interactive must be formatted, run, and print its
// expected output (or fail with its expected error). Structured lessons also
// check their exercise and their diagnostic example.

import { readdir, readFile, writeFile } from "node:fs/promises"
import { resolve } from "node:path"
import { runSampleCommand } from "./sample-cli-process"
import { loadValidatedTourCurriculum } from "./tour-curriculum"
import { repositoryPath } from "./tour-lessons"

const repositoryRoot = resolve(import.meta.dir, "..")
const updateTourDiagnostics = process.argv.includes("--update-tour-diagnostics")
const { lessons: tourLessons } =
  await loadValidatedTourCurriculum(repositoryRoot)
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
    const formatCheck = await runSampleCommand(
      [executable, "format", "--check", lesson.sourcePath],
      { cwd: repositoryRoot, label: `Tour ${lesson.metadata.id} format` }
    )
    if (formatCheck.status !== 0) {
      throw new Error(
        `Tour lesson ${lesson.metadata.id} is not formatted:\n${formatCheck.stdout}${formatCheck.stderr}`
      )
    }
    const stdin = lesson.stdinPath
      ? await readFile(lesson.stdinPath, "utf8")
      : ""
    const expected = lesson.expectedOutputPath
      ? await readFile(lesson.expectedOutputPath, "utf8")
      : ""
    const { status, stdout, stderr } = await runSampleCommand(
      [executable, "run", lesson.sourcePath],
      {
        cwd: repositoryRoot,
        label: `Tour ${lesson.metadata.id} run`,
        stdin,
      }
    )
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
  const exerciseFormat = await runSampleCommand(
    [executable, "format", "--check", lesson.exercisePath],
    { cwd: repositoryRoot, label: `Tour ${lesson.metadata.id} exercise format` }
  )
  if (exerciseFormat.status !== 0) {
    throw new Error(
      `Tour lesson ${lesson.metadata.id} exercise is not formatted:\n${exerciseFormat.stdout}${exerciseFormat.stderr}`
    )
  }
  const exerciseRun = await runSampleCommand(
    [executable, "run", lesson.exercisePath],
    { cwd: repositoryRoot, label: `Tour ${lesson.metadata.id} exercise run` }
  )
  if (exerciseRun.status !== 0) {
    throw new Error(
      `Tour lesson ${lesson.metadata.id} exercise failed in CLI:\n${exerciseRun.stderr}`
    )
  }
  const expectedExercise = (
    await readFile(lesson.exerciseExpectedOutputPath, "utf8")
  ).replace(/\r?\n$/u, "")
  if (exerciseRun.stdout.replace(/\r?\n$/u, "") !== expectedExercise) {
    throw new Error(
      `Tour lesson ${lesson.metadata.id} exercise output mismatch\nexpected: ${JSON.stringify(expectedExercise)}\nactual: ${JSON.stringify(exerciseRun.stdout.replace(/\r?\n$/u, ""))}`
    )
  }
  checkedTourExercises += 1

  const diagnosticRun = await runSampleCommand(
    [
      executable,
      "run",
      repositoryPath(repositoryRoot, lesson.diagnosticExamplePath),
    ],
    { cwd: repositoryRoot, label: `Tour ${lesson.metadata.id} diagnostic run` }
  )
  if (diagnosticRun.status === 0 || diagnosticRun.stderr.trim() === "") {
    throw new Error(
      `Tour lesson ${lesson.metadata.id} diagnostic example did not fail`
    )
  }
  if (diagnosticRun.stdout !== "") {
    throw new Error(
      `Tour lesson ${lesson.metadata.id} diagnostic example wrote stdout`
    )
  }
  if (updateTourDiagnostics) {
    await writeFile(lesson.diagnosticOutputPath, diagnosticRun.stderr)
  } else {
    const expectedDiagnostic = await readFile(
      lesson.diagnosticOutputPath,
      "utf8"
    )
    if (diagnosticRun.stderr !== expectedDiagnostic) {
      throw new Error(
        `Tour lesson ${lesson.metadata.id} diagnostic snapshot is stale; run \`bun run tour:diagnostics:update\``
      )
    }
  }
  checkedTourDiagnostics += 1
}

// The spec lessons and Playground examples are read as model code, so they
// stay in the current formatter's layout. Diagnostic fixtures under
// examples/spec/fixtures are written broken on purpose and are not checked.
const formattedSpecDirectories = [
  "examples/spec/lessons",
  "examples/spec/playground",
]
let checkedSpecFormatting = 0
for (const directory of formattedSpecDirectories) {
  const sources = (await readdir(resolve(repositoryRoot, directory)))
    .filter((name) => name.endsWith(".ssrg"))
    .sort()
  for (const name of sources) {
    const sourcePath = resolve(repositoryRoot, directory, name)
    const formatCheck = await runSampleCommand(
      [executable, "format", "--check", sourcePath],
      { cwd: repositoryRoot, label: `${directory}/${name} format` }
    )
    if (formatCheck.status !== 0) {
      throw new Error(
        `${directory}/${name} is not formatted; run \`seseragi format\` on it:\n${formatCheck.stdout}${formatCheck.stderr}`
      )
    }
    checkedSpecFormatting += 1
  }
}

console.log(
  `Validated ${checkedTourLessons} Tour lessons, ${checkedTourExercises} exercises, ${checkedTourDiagnostics} Tour diagnostics and the formatting of ${checkedSpecFormatting} spec sources with the native Seseragi CLI (${tourLessons.length - checkedTourLessons} browser-interactive skipped).`
)
