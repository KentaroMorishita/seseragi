import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import { cpSync, mkdtempSync, readdirSync, readFileSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, relative, resolve } from "node:path"
import {
  type ModuleProject,
  moduleExampleId,
  moduleProjectExamples,
  moduleProjects,
  moduleTypescriptProject,
} from "../scripts/module-examples"

const root = resolve(import.meta.dir, "../../..")
const fixtureRoot = join(root, "apps/site/examples/projects/modules")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
setDefaultTimeout(120_000)

function run(command: string, args: string[], cwd: string) {
  return spawnSync(command, args, {
    cwd,
    encoding: "utf8",
    timeout: 90_000,
    maxBuffer: 16 * 1024 * 1024,
  })
}

function files(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      const path = join(directory, entry.name)
      return entry.isDirectory() ? files(path) : [path]
    })
    .sort()
}

function snapshot(directory: string) {
  return files(directory).map((path) => [
    relative(directory, path),
    readFileSync(path, "utf8"),
  ])
}

function copiedRun(project: ModuleProject, temporary: string) {
  const sourceDirectory = join(root, project.directory)
  const copiedDirectory = join(temporary, project.key)
  cpSync(sourceDirectory, copiedDirectory, { recursive: true })
  // Copy the whole directory, including both packages in dependency examples.
  // Verify bytes before lock/build commands create any temporary artifacts.
  expect(snapshot(copiedDirectory), project.key).toEqual(
    snapshot(sourceDirectory)
  )
  const entryDirectory = join(copiedDirectory, project.entry)
  const lock = run(cli, ["lock", "update", entryDirectory], copiedDirectory)
  expect(lock.status, lock.stderr || lock.error?.message).toBe(0)
  return run(cli, ["run", entryDirectory], copiedDirectory)
}

function expectOutput(project: ModuleProject, temporary: string) {
  if (!("output" in project.result))
    throw new Error(`Repair ${project.key} is not a valid program`)
  const result = copiedRun(project, temporary)
  expect(result.status, result.stderr || result.error?.message).toBe(0)
  expect(result.stderr, project.key).toBe("")
  expect(result.stdout, project.key).toBe(project.result.output)
  return result.stdout
}

test("module descriptors cover every complete canonical project file", () => {
  expect(new Set(moduleProjects.map((project) => project.key)).size).toBe(
    moduleProjects.length
  )
  const declaredPaths = [
    ...moduleProjects.flatMap((project) =>
      project.files.map((file) => join(root, project.directory, file.path))
    ),
    ...moduleTypescriptProject.files.map((file) =>
      join(root, moduleTypescriptProject.directory, file.path)
    ),
  ].sort()
  expect(declaredPaths).toEqual(files(fixtureRoot))
  for (const project of moduleProjects) {
    expect(project.files.map((file) => file.path)).toContain(
      project.entry === "." ? "seseragi.toml" : "app/seseragi.toml"
    )
    if ("diagnostics" in project.result) {
      const repairKey = project.result.repair
      const repair = moduleProjects.find(
        (candidate) => candidate.key === repairKey
      )
      expect(
        repair,
        `${project.key}: missing repair ${repairKey}`
      ).toBeDefined()
      expect(repair && "output" in repair.result, repairKey).toBe(true)
      expect(project.result.diagnostics.length).toBeGreaterThan(0)
    }
  }
})

test("registered multi-file sources have exact bytes and hashes and no single-file Playground link", () => {
  const examples = moduleProjectExamples("https://seseragi.vercel.app/")
  expect(examples).toHaveLength(files(fixtureRoot).length)
  expect(new Set(examples.map((example) => example.id)).size).toBe(
    examples.length
  )
  for (const example of examples) {
    const source = readFileSync(join(root, example.sourcePath), "utf8")
    expect(example.source, example.id).toBe(source)
    expect(example.sha256, example.id).toBe(
      createHash("sha256").update(source).digest("hex")
    )
    expect(
      example.highlighted.map((part) => part.text).join(""),
      example.id
    ).toBe(source)
    expect(example.playgroundUrl, example.id).toBe("")
    if (example.sourcePath.endsWith(".toml"))
      expect(example.highlighted).toEqual([{ text: source, className: "" }])
  }
  for (const project of moduleProjects) {
    for (const file of project.files) {
      expect(
        examples.find(
          (example) => example.id === moduleExampleId(project.key, file.key)
        )?.sourcePath
      ).toBe(`${project.directory}/${file.path}`)
    }
  }
})

for (const project of moduleProjects) {
  test(`native module project ${project.key} ${"output" in project.result ? "prints its stated output" : "rejects the mistake and executes its repair"}`, () => {
    const original = snapshot(fixtureRoot)
    const temporary = mkdtempSync(join(tmpdir(), "seseragi-module-project-"))
    try {
      if ("output" in project.result) {
        expectOutput(project, temporary)
      } else {
        const result = copiedRun(project, temporary)
        expect(result.status, result.stderr || result.error?.message).toBe(2)
        expect(result.stdout, project.key).toBe("")
        for (const diagnostic of project.result.diagnostics)
          expect(result.stderr, project.key).toContain(diagnostic)
        const repairKey = project.result.repair
        const repair = moduleProjects.find(
          (candidate) => candidate.key === repairKey
        )
        if (!repair) throw new Error(`Missing repair ${repairKey}`)
        expectOutput(repair, temporary)
      }
    } finally {
      rmSync(temporary, { recursive: true, force: true })
      // Locks and generated files belong only to isolated temporary copies.
      expect(snapshot(fixtureRoot), project.key).toEqual(original)
    }
  })
}

test("ordinary TypeScript export/import strict-checks and prints the same greeting as Seseragi", () => {
  const original = snapshot(fixtureRoot)
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-module-comparison-"))
  try {
    const sourceDirectory = join(root, moduleTypescriptProject.directory)
    const copiedDirectory = join(temporary, "typescript")
    cpSync(sourceDirectory, copiedDirectory, { recursive: true })
    expect(snapshot(copiedDirectory)).toEqual(snapshot(sourceDirectory))
    const main = join(copiedDirectory, "main.ts")
    const checked = run(
      join(root, "node_modules/.bin/tsc"),
      [
        "--strict",
        "--noEmit",
        "--skipLibCheck",
        "--types",
        "bun",
        "--typeRoots",
        join(root, "node_modules/@types"),
        "--target",
        "ES2022",
        "--module",
        "Preserve",
        "--moduleResolution",
        "Bundler",
        main,
      ],
      copiedDirectory
    )
    expect(checked.status, checked.stdout || checked.stderr).toBe(0)
    const typescript = run(process.execPath, [main], copiedDirectory)
    expect(typescript.status, typescript.stderr).toBe(0)
    expect(typescript.stderr).toBe("")
    expect(typescript.stdout).toBe(moduleTypescriptProject.output)
    const native = moduleProjects.find(
      (project) => project.key === "named-import"
    )
    if (!native) throw new Error("Missing paired Seseragi greeting")
    expect(typescript.stdout).toBe(expectOutput(native, temporary))
  } finally {
    rmSync(temporary, { recursive: true, force: true })
    expect(snapshot(fixtureRoot)).toEqual(original)
  }
})
