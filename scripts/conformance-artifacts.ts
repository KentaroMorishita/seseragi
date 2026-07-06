import { existsSync, readdirSync, readFileSync } from "node:fs"
import { join, relative, resolve } from "node:path"

type Phase = "frontend" | "interface" | "runtime" | "stage" | "execution"

type Check = {
  name: string
  status: "pass" | "fail"
  message?: string
}

type CaseResult = {
  id: string
  phase: Phase
  path: string
  status: "pass" | "fail"
  checks: Check[]
}

const repoRoot = resolve(import.meta.dir, "..")
const artifactsRoot = resolve(repoRoot, "examples/spec/artifacts")
const args = new Set(Bun.argv.slice(2))
const outputJson = args.has("--json")
const listOnly = args.has("--list")

const relativeToRepo = (path: string): string => relative(repoRoot, path)

const readJson = <T>(path: string, checks: Check[], name: string): T | null => {
  if (!existsSync(path)) {
    checks.push({
      name,
      status: "fail",
      message: `${relativeToRepo(path)} is missing`,
    })
    return null
  }
  try {
    const parsed = JSON.parse(readFileSync(path, "utf8")) as T
    checks.push({ name, status: "pass" })
    return parsed
  } catch (error) {
    checks.push({
      name,
      status: "fail",
      message: `${relativeToRepo(path)} is not valid JSON: ${error}`,
    })
    return null
  }
}

const requireFile = (path: string, checks: Check[], name: string): void => {
  checks.push(
    existsSync(path)
      ? { name, status: "pass" }
      : {
          name,
          status: "fail",
          message: `${relativeToRepo(path)} is missing`,
        }
  )
}

const caseStatus = (checks: Check[]): "pass" | "fail" =>
  checks.every((check) => check.status === "pass") ? "pass" : "fail"

const artifactDirectories = (schema: string): string[] => {
  const directory = join(artifactsRoot, schema)
  if (!existsSync(directory)) return []
  return readdirSync(directory, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => join(directory, entry.name))
    .sort()
}

const checkSchemaOneCase = (directory: string): CaseResult => {
  const checks: Check[] = []
  for (const file of [
    "main.ssrg",
    "tokens.json",
    "cst.json",
    "diagnostics.json",
    "interface.json",
  ]) {
    requireFile(join(directory, file), checks, file)
  }

  const moduleInterface = readJson<{
    schema?: unknown
    source?: unknown
    exports?: unknown
  }>(join(directory, "interface.json"), checks, "interface envelope")
  if (
    moduleInterface &&
    (moduleInterface.schema !== 1 ||
      typeof moduleInterface.source !== "string" ||
      !Array.isArray(moduleInterface.exports))
  ) {
    checks.push({
      name: "interface shape",
      status: "fail",
      message: "interface.json must use schema 1 with source and exports",
    })
  } else if (moduleInterface) {
    checks.push({ name: "interface shape", status: "pass" })
  }

  const hasEmitterArtifacts = existsSync(
    join(directory, "generated-module.json")
  )
  if (hasEmitterArtifacts) {
    for (const file of [
      "surface-ast.json",
      "resolved-ast.json",
      "typed-hir.json",
      "core-ir.json",
      "typescript-ir.json",
      "generated-module.json",
      "main.ts",
      "main.ts.map",
    ]) {
      requireFile(join(directory, file), checks, file)
    }
  }

  return {
    id: `schema-1/${relative(join(artifactsRoot, "schema-1"), directory)}`,
    phase: "frontend",
    path: relativeToRepo(directory),
    status: caseStatus(checks),
    checks,
  }
}

const checkInterfaceCase = (directory: string): CaseResult => {
  const checks: Check[] = []
  for (const file of ["main.ssrg", "interface.json"]) {
    requireFile(join(directory, file), checks, file)
  }
  const moduleInterface = readJson<{
    schema?: unknown
    module?: unknown
    dependencies?: unknown
    exports?: unknown
  }>(join(directory, "interface.json"), checks, "interface envelope")
  if (
    moduleInterface &&
    (moduleInterface.schema !== 1 ||
      typeof moduleInterface.module !== "string" ||
      !Array.isArray(moduleInterface.dependencies) ||
      !Array.isArray(moduleInterface.exports))
  ) {
    checks.push({
      name: "interface shape",
      status: "fail",
      message: "interface artifact must expose module, dependencies, exports",
    })
  } else if (moduleInterface) {
    checks.push({ name: "interface shape", status: "pass" })
  }

  return {
    id: `interface-schema-1/${relative(
      join(artifactsRoot, "interface-schema-1"),
      directory
    )}`,
    phase: "interface",
    path: relativeToRepo(directory),
    status: caseStatus(checks),
    checks,
  }
}

const checkRuntimeCase = (): CaseResult => {
  const checks: Check[] = []
  const path = join(artifactsRoot, "runtime-schema-1/core/abi.json")
  const abi = readJson<{
    schema?: unknown
    identity?: unknown
    abiMajor?: unknown
    features?: unknown
  }>(path, checks, "runtime ABI")
  if (
    abi &&
    (abi.schema !== 1 ||
      abi.identity !== "@seseragi/runtime" ||
      abi.abiMajor !== 1 ||
      !Array.isArray(abi.features))
  ) {
    checks.push({
      name: "runtime ABI shape",
      status: "fail",
      message: "runtime ABI must use schema 1 and expose feature registry",
    })
  } else if (abi) {
    checks.push({ name: "runtime ABI shape", status: "pass" })
  }

  return {
    id: "runtime-schema-1/core",
    phase: "runtime",
    path: relativeToRepo(path),
    status: caseStatus(checks),
    checks,
  }
}

const checkStageCase = (directory: string): CaseResult => {
  const checks: Check[] = []
  for (const file of [
    "main.ssrg",
    "interface.json",
    "surface-ast.json",
    "resolved-ast.json",
    "typed-hir.json",
    "core-ir.json",
    "typescript-ir.json",
    "generated-module.json",
    "main.ts",
    "main.ts.map",
  ]) {
    requireFile(join(directory, file), checks, file)
  }
  for (const stage of [
    "surface-ast",
    "resolved-ast",
    "typed-hir",
    "core-ir",
    "typescript-ir",
  ]) {
    const artifact = readJson<{ schema?: unknown; stage?: unknown }>(
      join(directory, `${stage}.json`),
      checks,
      `${stage} envelope`
    )
    if (artifact && (artifact.schema !== 1 || artifact.stage !== stage)) {
      checks.push({
        name: `${stage} shape`,
        status: "fail",
        message: `${stage}.json must declare schema 1 and matching stage`,
      })
    } else if (artifact) {
      checks.push({ name: `${stage} shape`, status: "pass" })
    }
  }

  return {
    id: `stage-schema-1/${relative(
      join(artifactsRoot, "stage-schema-1"),
      directory
    )}`,
    phase: "stage",
    path: relativeToRepo(directory),
    status: caseStatus(checks),
    checks,
  }
}

const checkExecutionCase = (directory: string): CaseResult => {
  const checks: Check[] = []
  const run = readJson<{
    schema?: unknown
    case?: unknown
    entry?: { compiledModule?: unknown }
    expected?: { stdout?: unknown; stderr?: unknown }
  }>(join(directory, "run.json"), checks, "run envelope")
  if (
    run &&
    (run.schema !== 1 ||
      run.case !==
        relative(join(artifactsRoot, "execution-schema-1"), directory))
  ) {
    checks.push({
      name: "run shape",
      status: "fail",
      message: "run.json must use schema 1 and match its directory name",
    })
  } else if (run) {
    checks.push({ name: "run shape", status: "pass" })
  }
  if (run?.entry && typeof run.entry.compiledModule === "string") {
    requireFile(
      resolve(directory, run.entry.compiledModule),
      checks,
      "compiled module reference"
    )
  }
  if (run?.expected && typeof run.expected.stdout === "string") {
    requireFile(join(directory, run.expected.stdout), checks, "stdout snapshot")
  }
  if (run?.expected && typeof run.expected.stderr === "string") {
    requireFile(join(directory, run.expected.stderr), checks, "stderr snapshot")
  }

  return {
    id: `execution-schema-1/${relative(
      join(artifactsRoot, "execution-schema-1"),
      directory
    )}`,
    phase: "execution",
    path: relativeToRepo(directory),
    status: caseStatus(checks),
    checks,
  }
}

const results: CaseResult[] = [
  ...artifactDirectories("schema-1").map(checkSchemaOneCase),
  ...artifactDirectories("interface-schema-1").map(checkInterfaceCase),
  checkRuntimeCase(),
  ...artifactDirectories("stage-schema-1").map(checkStageCase),
  ...artifactDirectories("execution-schema-1").map(checkExecutionCase),
].sort((left, right) => left.id.localeCompare(right.id))

const failures = results.flatMap((result) =>
  result.checks
    .filter((check) => check.status === "fail")
    .map((check) => `${result.id}: ${check.name}: ${check.message}`)
)

if (listOnly) {
  for (const result of results) {
    console.log(`${result.phase}\t${result.id}\t${result.path}`)
  }
} else if (outputJson) {
  console.log(
    JSON.stringify(
      {
        schema: 1,
        root: relativeToRepo(artifactsRoot),
        summary: {
          total: results.length,
          passed: results.filter((result) => result.status === "pass").length,
          failed: results.filter((result) => result.status === "fail").length,
        },
        cases: results,
      },
      null,
      2
    )
  )
} else {
  console.log(`Conformance artifact cases: ${results.length}`)
  for (const phase of [
    "frontend",
    "interface",
    "runtime",
    "stage",
    "execution",
  ] satisfies Phase[]) {
    const phaseResults = results.filter((result) => result.phase === phase)
    const passed = phaseResults.filter(
      (result) => result.status === "pass"
    ).length
    console.log(`  ${phase}: ${passed}/${phaseResults.length} passed`)
  }
  if (failures.length > 0) {
    console.error("")
    for (const failure of failures) {
      console.error(failure)
    }
  }
}

if (failures.length > 0) {
  process.exit(1)
}
