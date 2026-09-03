import { expect, test } from "bun:test"
import {
  cpSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"
import { UNICODE_VERSION } from "../src/unicode-version"

const runtime = fileURLToPath(new URL("..", import.meta.url))
const fixture = fileURLToPath(
  new URL(
    "../../../examples/spec/artifacts/project-schema-1/imported-unicode/",
    import.meta.url
  )
)

/** Run the real compiler-produced, imported modules, not a handwritten guard. */
function importArtifacts(runtimeVersion: string, dependencyVersion?: string) {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-unicode-artifact-"))
  try {
    const packageDirectory = join(directory, "node_modules/@seseragi/runtime")
    mkdirSync(packageDirectory, { recursive: true })
    cpSync(join(runtime, "src"), join(packageDirectory, "src"), {
      recursive: true,
    })
    cpSync(
      join(runtime, "package.json"),
      join(packageDirectory, "package.json")
    )
    // Only the version payload changes; the shipped check and Unicode tables
    // remain untouched, so this also tests the actual runtime import path.
    writeFileSync(
      join(packageDirectory, "src/unicode-version-data.ts"),
      `export const UNICODE_VERSION = ${JSON.stringify(runtimeVersion)}\n`
    )
    const project = JSON.parse(
      readFileSync(join(fixture, "project.json"), "utf8")
    )
    for (const module of project.modules) {
      const metadata = JSON.parse(
        readFileSync(
          join(fixture, module.artifacts, "generated-module.json"),
          "utf8"
        )
      )
      expect(metadata.runtime.unicodeVersion).toBe(UNICODE_VERSION)
      let source = readFileSync(
        join(fixture, module.artifacts, "main.ts"),
        "utf8"
      )
      const guard = `$ssrg$assertUnicodeVersion(${JSON.stringify(UNICODE_VERSION)})`
      expect(source.split(guard)).toHaveLength(2)
      const guardIndex = source.indexOf(guard)
      expect(guardIndex).toBeLessThan(source.indexOf("export const "))
      const operations = module.id.endsWith("::operations")
      if (operations)
        expect(guardIndex).toBeLessThan(source.indexOf("const canonicalize"))
      // Mark the first source initializer of each compiled module. This is
      // test-only instrumentation, never a replacement for the generated guard.
      const actualGuard =
        operations && dependencyVersion
          ? `$ssrg$assertUnicodeVersion(${JSON.stringify(dependencyVersion)})`
          : guard
      source = source.replace(
        guard,
        `${actualGuard}\nconsole.log(${JSON.stringify(module.id)})`
      )
      const output = join(directory, module.output.replace(/\.js$/, ".ts"))
      mkdirSync(dirname(output), { recursive: true })
      writeFileSync(output, source)
    }
    const result = Bun.spawnSync(
      [process.execPath, join(directory, "dist/imported-unicode/main.ts")],
      { cwd: directory, stdout: "pipe", stderr: "pipe" }
    )
    return {
      exitCode: result.exitCode,
      stdout: result.stdout.toString(),
      stderr: result.stderr.toString(),
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
}

test("matching runtime evaluates the imported and entry source initializers", () => {
  const result = importArtifacts(UNICODE_VERSION)
  expect(result.exitCode).toBe(0)
  expect(result.stderr).toBe("")
  expect(result.stdout).toBe(
    "fixture/imported-unicode::operations\nfixture/imported-unicode::main\n"
  )
})

test("older and newer runtimes reject actual compiled imports before source initialization", () => {
  for (const version of ["16.0.0", "18.0.0"]) {
    const result = importArtifacts(version)
    expect(result.exitCode).not.toBe(0)
    expect(result.stdout).toBe("")
    expect(result.stderr).toContain(
      `runtime ABI mismatch: artifact requires Unicode ${UNICODE_VERSION}, runtime provides ${version}`
    )
  }
})

test("a dependency with another Unicode requirement rejects a matching entry runtime", () => {
  const result = importArtifacts(UNICODE_VERSION, "18.0.0")
  expect(result.exitCode).not.toBe(0)
  expect(result.stdout).toBe("")
  expect(result.stderr).toContain(
    `runtime ABI mismatch: artifact requires Unicode 18.0.0, runtime provides ${UNICODE_VERSION}`
  )
})
