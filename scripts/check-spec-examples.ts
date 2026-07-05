import { existsSync, readdirSync, readFileSync } from "node:fs"
import { basename, dirname, join, resolve } from "node:path"

const root = resolve(import.meta.dir, "../examples/spec")
const specDir = resolve(import.meta.dir, "../docs/spec")
const lessonsDir = join(root, "lessons")
const fixturesDir = join(root, "fixtures")
const errors: string[] = []
const specSections = new Set<string>()
const diagnosticRegistry = new Map<string, string>()

for (const name of readdirSync(specDir).filter((name) =>
  name.endsWith(".md")
)) {
  const source = readFileSync(join(specDir, name), "utf8")
  for (const match of source.matchAll(/^## (\d+\.\d+)\b/gm)) {
    specSections.add(match[1])
  }
}

const toolingSpec = readFileSync(join(specDir, "12-tooling.md"), "utf8")
for (const match of toolingSpec.matchAll(
  /^\|\s*(SES-[PNTIEKFL]\d{4})\s*\|\s*(Error|Warning|Information|Hint)\s*\|/gm
)) {
  diagnosticRegistry.set(match[1], match[2])
}

if (diagnosticRegistry.size === 0) {
  errors.push("docs/spec/12-tooling.md: diagnostic registry is empty")
}

const lessons = readdirSync(lessonsDir)
  .filter((name) => /^\d{2}-.*\.ssrg$/.test(name))
  .sort()

lessons.forEach((name, index) => {
  const expectedNumber = String(index + 1).padStart(2, "0")
  const actualNumber = name.slice(0, 2)
  const path = join(lessonsDir, name)
  const source = readFileSync(path, "utf8")
  const lines = source.split("\n")

  if (actualNumber !== expectedNumber) {
    errors.push(`${name}: expected lesson number ${expectedNumber}`)
  }
  if (!lines[0]?.startsWith(`// Lesson ${actualNumber}:`)) {
    errors.push(`${name}: first line must declare its lesson number and goal`)
  }
  if (!lines[1]?.startsWith("// 前提:")) {
    errors.push(`${name}: second line must declare prerequisites in Japanese`)
  }

  const prerequisites = [...(lines[1]?.matchAll(/\b(\d{2})\b/g) ?? [])].map(
    (match) => Number(match[1])
  )
  if (!lines[1]?.includes("なし") && prerequisites.length === 0) {
    errors.push(`${name}: prerequisites must name an earlier lesson or なし`)
  }
  if (prerequisites.some((lesson) => lesson >= Number(actualNumber))) {
    errors.push(`${name}: prerequisites must refer only to earlier lessons`)
  }

  const hasJapaneseExplanation = lines
    .slice(2)
    .some((line) => /^\s*\/\/ .*[ぁ-んァ-ヶ一-龠]/.test(line))
  if (!hasJapaneseExplanation) {
    errors.push(`${name}: missing a Japanese learning comment`)
  }

  const marker = lines.findIndex((line) =>
    line.startsWith("// Expected stdout:")
  )
  if (marker < 0) {
    errors.push(`${name}: missing Expected stdout marker`)
    return
  }

  const snapshot = lines[marker]?.slice("// Expected stdout:".length).trim()
  if (snapshot) {
    const snapshotPath = resolve(dirname(path), snapshot)
    if (!existsSync(snapshotPath)) {
      errors.push(`${name}: missing stdout snapshot ${snapshot}`)
    }
  } else if (!lines.slice(marker + 1).some((line) => line.startsWith("// "))) {
    errors.push(`${name}: inline Expected stdout is empty`)
  }
})

const lessonReadme = readFileSync(join(lessonsDir, "README.md"), "utf8")
for (const name of lessons) {
  const number = basename(name).slice(0, 2)
  if (!new RegExp(`\\|\\s*${number}\\s*\\|`).test(lessonReadme)) {
    errors.push(`${name}: missing from lessons/README.md learning path`)
  }
}

type ExpectedRange = {
  start: number
  end: number
  text: string
}

type ExpectedDiagnostic = {
  code: string
  severity: string
  primary: ExpectedRange
}

type FixtureExpectation = {
  schema: number
  kind: string
  spec: string[]
  diagnostics?: ExpectedDiagnostic[]
}

type ProjectExpectation = {
  schema: number
  kind: string
  phase: string
  spec: string[]
  lock: string
  stdout?: string
}

let fixtureCount = 0
for (const fixtureKind of ["compile", "diagnostics"] as const) {
  const directory = join(fixturesDir, fixtureKind)
  const sources = readdirSync(directory)
    .filter((name) => name.endsWith(".ssrg"))
    .sort()
  const sidecars = new Set(
    readdirSync(directory).filter((name) => name.endsWith(".expect.json"))
  )

  for (const name of sources) {
    fixtureCount += 1
    const sourcePath = join(directory, name)
    const source = readFileSync(sourcePath, "utf8")
    const expectedKind = fixtureKind === "compile" ? "compile" : "diagnostic"
    const sidecarName = name.replace(/\.ssrg$/, ".expect.json")
    const sidecarPath = join(directory, sidecarName)

    if (!existsSync(sidecarPath)) {
      errors.push(`${fixtureKind}/${name}: missing ${sidecarName}`)
      continue
    }

    if (source.includes("\r")) {
      errors.push(`${fixtureKind}/${name}: fixture source must use LF`)
    }
    if (!source.endsWith("\n")) {
      errors.push(`${fixtureKind}/${name}: fixture source needs final newline`)
    }
    sidecars.delete(sidecarName)

    let expectation: FixtureExpectation
    try {
      const parsed: unknown = JSON.parse(readFileSync(sidecarPath, "utf8"))
      if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
        errors.push(`${fixtureKind}/${sidecarName}: root must be an object`)
        continue
      }
      expectation = parsed as FixtureExpectation
    } catch (error) {
      errors.push(`${fixtureKind}/${sidecarName}: invalid JSON: ${error}`)
      continue
    }

    if (expectation.schema !== 1) {
      errors.push(`${fixtureKind}/${sidecarName}: schema must be 1`)
    }
    if (expectation.kind !== expectedKind) {
      errors.push(`${fixtureKind}/${sidecarName}: kind must be ${expectedKind}`)
    }
    if (
      !Array.isArray(expectation.spec) ||
      expectation.spec.length === 0 ||
      expectation.spec.some((section) => !/^\d+\.\d+$/.test(section))
    ) {
      errors.push(
        `${fixtureKind}/${sidecarName}: spec must contain section numbers`
      )
    } else {
      for (const section of expectation.spec) {
        if (!specSections.has(section)) {
          errors.push(
            `${fixtureKind}/${sidecarName}: unknown spec section ${section}`
          )
        }
      }
    }

    if (expectedKind === "compile") {
      if (expectation.diagnostics !== undefined) {
        errors.push(
          `${fixtureKind}/${sidecarName}: compile fixture cannot declare diagnostics`
        )
      }
      continue
    }

    if (
      !Array.isArray(expectation.diagnostics) ||
      expectation.diagnostics.length === 0
    ) {
      errors.push(
        `${fixtureKind}/${sidecarName}: diagnostic fixture needs diagnostics`
      )
      continue
    }

    const bytes = Buffer.from(source, "utf8")
    for (const diagnostic of expectation.diagnostics) {
      if (
        !diagnostic ||
        typeof diagnostic !== "object" ||
        Array.isArray(diagnostic)
      ) {
        errors.push(
          `${fixtureKind}/${sidecarName}: diagnostic must be an object`
        )
        continue
      }
      if (!/^SES-[PNTIEKFL]\d{4}$/.test(diagnostic.code)) {
        errors.push(
          `${fixtureKind}/${sidecarName}: invalid code ${diagnostic.code}`
        )
      } else if (!diagnosticRegistry.has(diagnostic.code)) {
        errors.push(
          `${fixtureKind}/${sidecarName}: unregistered code ${diagnostic.code}`
        )
      }
      if (
        !["Error", "Warning", "Information", "Hint"].includes(
          diagnostic.severity
        )
      ) {
        errors.push(
          `${fixtureKind}/${sidecarName}: invalid severity ${diagnostic.severity}`
        )
      } else if (
        diagnosticRegistry.get(diagnostic.code) !== diagnostic.severity
      ) {
        errors.push(
          `${fixtureKind}/${sidecarName}: severity for ${diagnostic.code} must match registry ${diagnosticRegistry.get(diagnostic.code)}`
        )
      }

      const range = diagnostic.primary
      if (
        !range ||
        typeof range !== "object" ||
        typeof range.text !== "string" ||
        !Number.isInteger(range.start) ||
        !Number.isInteger(range.end) ||
        range.start < 0 ||
        range.end < range.start ||
        range.end > bytes.length
      ) {
        errors.push(`${fixtureKind}/${sidecarName}: invalid primary byte range`)
        continue
      }

      const actual = bytes.subarray(range.start, range.end).toString("utf8")
      if (actual !== range.text) {
        errors.push(
          `${fixtureKind}/${sidecarName}: primary text expected ${JSON.stringify(range.text)}, got ${JSON.stringify(actual)}`
        )
      }
      const startsOnBoundary = Buffer.from(
        bytes.subarray(0, range.start).toString("utf8"),
        "utf8"
      ).equals(bytes.subarray(0, range.start))
      const endsOnBoundary = Buffer.from(
        bytes.subarray(0, range.end).toString("utf8"),
        "utf8"
      ).equals(bytes.subarray(0, range.end))
      if (!startsOnBoundary || !endsOnBoundary) {
        errors.push(
          `${fixtureKind}/${sidecarName}: primary range is not on UTF-8 boundaries`
        )
      }
    }
  }

  for (const orphan of [...sidecars].sort()) {
    errors.push(`${fixtureKind}/${orphan}: no matching .ssrg source`)
  }
}

const projectsDir = join(fixturesDir, "projects")
const projects = readdirSync(projectsDir, { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .sort()

for (const name of projects) {
  const directory = join(projectsDir, name)
  const manifestPath = join(directory, "seseragi.toml")
  const expectationPath = join(directory, "project.expect.json")

  if (!existsSync(manifestPath)) {
    errors.push(`projects/${name}: missing seseragi.toml`)
  }
  if (!existsSync(expectationPath)) {
    errors.push(`projects/${name}: missing project.expect.json`)
    continue
  }

  let expectation: ProjectExpectation
  try {
    const parsed: unknown = JSON.parse(readFileSync(expectationPath, "utf8"))
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      errors.push(`projects/${name}: expectation root must be an object`)
      continue
    }
    expectation = parsed as ProjectExpectation
  } catch (error) {
    errors.push(`projects/${name}: invalid project.expect.json: ${error}`)
    continue
  }

  if (expectation.schema !== 1) {
    errors.push(`projects/${name}: schema must be 1`)
  }
  if (expectation.kind !== "project") {
    errors.push(`projects/${name}: kind must be project`)
  }
  if (
    !["compile", "diagnostic", "run", "test", "convert", "tooling"].includes(
      expectation.phase
    )
  ) {
    errors.push(`projects/${name}: invalid phase ${expectation.phase}`)
  }
  if (
    !Array.isArray(expectation.spec) ||
    expectation.spec.length === 0 ||
    expectation.spec.some((section) => !specSections.has(section))
  ) {
    errors.push(`projects/${name}: spec must reference known sections`)
  }
  if (!["generate", "fixture"].includes(expectation.lock)) {
    errors.push(`projects/${name}: lock must be generate or fixture`)
  } else if (
    expectation.lock === "fixture" &&
    !existsSync(join(directory, "seseragi.lock"))
  ) {
    errors.push(`projects/${name}: fixture lock requires seseragi.lock`)
  }

  if (expectation.stdout !== undefined) {
    const stdoutPath = resolve(directory, expectation.stdout)
    if (!existsSync(stdoutPath)) {
      errors.push(`projects/${name}: missing stdout ${expectation.stdout}`)
    } else {
      const stdout = readFileSync(stdoutPath, "utf8")
      if (stdout.includes("\r")) {
        errors.push(`projects/${name}: stdout snapshot must use LF`)
      }
      if (!stdout.endsWith("\n")) {
        errors.push(`projects/${name}: stdout snapshot needs final newline`)
      }
    }
  }
}

if (errors.length > 0) {
  console.error(errors.join("\n"))
  process.exit(1)
}

console.log(
  `Spec lessons: ${lessons.length} checked; fixtures: ${fixtureCount} checked; projects: ${projects.length} checked`
)
