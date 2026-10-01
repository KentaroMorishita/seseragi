import { createHash } from "node:crypto"
import { readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { canonicalExample } from "./canonical-example"

const root = resolve(import.meta.dir, "../../..")
const fixtureRoot = "apps/site/examples/projects/modules"

export type ModuleProjectFile = {
  key: string
  path: string
}

type ModuleProjectResult =
  | { output: string }
  | { diagnostics: readonly string[]; repair: string }

export type ModuleProject = {
  key: string
  directory: string
  entry: string
  files: readonly ModuleProjectFile[]
  result: ModuleProjectResult
}

function project(
  key: string,
  sources: Record<string, string>,
  result: ModuleProjectResult,
  entry = "."
): ModuleProject {
  return {
    key,
    directory: `${fixtureRoot}/${key}`,
    entry,
    files: Object.entries(
      entry === "." ? { manifest: "seseragi.toml", ...sources } : sources
    ).map(([key, path]) => ({ key, path })),
    result,
  }
}

const greetingFiles = {
  greeting: "src/greeting.ssrg",
  main: "src/main.ssrg",
}
const domainFiles = { domain: "src/domain.ssrg", main: "src/main.ssrg" }
const cycleFiles = {
  "shared-a": "src/shared-a.ssrg",
  "shared-b": "src/shared-b.ssrg",
  main: "src/main.ssrg",
}
const packageFiles = {
  "app-manifest": "app/seseragi.toml",
  "library-manifest": "greetings-library/seseragi.toml",
  greeting: "greetings-library/src/lib.ssrg",
  secret: "greetings-library/src/internal/secret.ssrg",
  main: "app/src/main.ssrg",
}
const greeting = "Hello, Aki!\n"

// Complete executable projects. Rejected cases stay isolated from valid graphs;
// every repair names a complete project with its own asserted output.
export const moduleProjects: readonly ModuleProject[] = [
  project("named-import", greetingFiles, { output: greeting }),
  project(
    "re-export-greeting",
    {
      greeting: "src/greeting.ssrg",
      public: "src/public.ssrg",
      main: "src/main.ssrg",
    },
    { output: greeting }
  ),
  project(
    "re-export-private",
    {
      greeting: "src/greeting.ssrg",
      public: "src/public.ssrg",
      main: "src/main.ssrg",
    },
    {
      diagnostics: ["PrivateExport", 'name: "punctuate"'],
      repair: "re-export-greeting",
    }
  ),
  project("import-forms", greetingFiles, { output: greeting.repeat(3) }),
  project("namespace", greetingFiles, { output: greeting }),
  project("namespace-unqualified", greetingFiles, {
    diagnostics: ["SES-N0001", "Name could not be resolved", "greet"],
    repair: "namespace",
  }),
  project("identity-greeting", greetingFiles, { output: greeting.repeat(2) }),
  project("identity", domainFiles, { output: "42\n" }),
  project(
    "copied-identity",
    {
      domain: "src/domain.ssrg",
      "copied-domain": "src/copied-domain.ssrg",
      main: "src/main.ssrg",
    },
    {
      diagnostics: [
        "SES-T0101",
        "Argument type does not match the parameter type",
        "argument 1 expected UserId, received UserId",
      ],
      repair: "identity",
    }
  ),
  project("package-public", packageFiles, { output: greeting }, "app"),
  project(
    "package-private",
    packageFiles,
    {
      diagnostics: ["SES-N0104", "does not export `internal/secret`"],
      repair: "package-public",
    },
    "app"
  ),
  project(
    "initialization",
    { summary: "src/summary.ssrg", main: "src/main.ssrg" },
    { output: "42\n" }
  ),
  project("private-helper", greetingFiles, { output: greeting }),
  project("private-import", greetingFiles, {
    diagnostics: ["PrivateExport", 'name: "punctuate"'],
    repair: "private-helper",
  }),
  project("opaque", domainFiles, { output: "42\n" }),
  project("opaque-access", domainFiles, {
    diagnostics: ["SES-T0101", "record type has no field `value`"],
    repair: "opaque",
  }),
  project(
    "re-exports",
    {
      domain: "src/domain.ssrg",
      public: "src/public.ssrg",
      main: "src/main.ssrg",
    },
    { output: "42\n" }
  ),
  project("namespaces", { main: "src/main.ssrg" }, { output: "42, 1\n" }),
  project(
    "duplicate-name",
    { main: "src/main.ssrg" },
    {
      diagnostics: ["SES-N0002", "Name duplicate definition", "let value = 42"],
      repair: "namespaces",
    }
  ),
  project("cycle", cycleFiles, {
    diagnostics: [
      "SES-K0001",
      "Cycle",
      'ModulePath("shared-a")',
      'ModulePath("shared-b")',
    ],
    repair: "cycle-repair",
  }),
  project(
    "type-cycle",
    { user: "src/user.ssrg", order: "src/order.ssrg", main: "src/main.ssrg" },
    {
      diagnostics: [
        "SES-K0001",
        "Cycle",
        'ModulePath("user")',
        'ModulePath("order")',
      ],
      repair: "cycle-repair",
    }
  ),
  project(
    "cycle-repair",
    { common: "src/common.ssrg", ...cycleFiles },
    {
      output: "21, 22\n",
    }
  ),
  project(
    "early-read",
    { main: "src/main.ssrg" },
    {
      diagnostics: [
        "SES-N0201",
        "Top-level initializer reads a value before it is initialized",
        "later is not initialized before this call",
      ],
      repair: "ordered-read",
    }
  ),
  project("ordered-read", { main: "src/main.ssrg" }, { output: "42\n" }),
  project(
    "entry-int",
    { main: "src/main.ssrg" },
    {
      diagnostics: ["invalid entry point: `main` must succeed with Unit"],
      repair: "named-import",
    }
  ),
  project(
    "explicit-path",
    { greeting: "src/words/index.ssrg", main: "src/main.ssrg" },
    { output: greeting }
  ),
  project(
    "implicit-index",
    { greeting: "src/words/index.ssrg", main: "src/main.ssrg" },
    {
      diagnostics: ["SES-K0001", "src/words.ssrg", "entry does not exist"],
      repair: "explicit-path",
    }
  ),
  project(
    "operator",
    { operators: "src/operators.ssrg", main: "src/main.ssrg" },
    { output: "9\n" }
  ),
]

export const moduleTypescriptProject = {
  directory: `${fixtureRoot}/typescript`,
  files: [
    { key: "greeting", path: "greeting.ts" },
    { key: "main", path: "main.ts" },
  ],
  output: greeting,
} as const

export function moduleExampleId(projectKey: string, fileKey: string) {
  return `modules-project-${projectKey}-${fileKey}`
}

// TOML is an exact plain-text ExampleSource. It deliberately does not use the
// Seseragi highlighter, and a single-file Playground cannot run these projects.
function manifestExample(id: string, sourcePath: string) {
  const source = readFileSync(join(root, sourcePath), "utf8")
  return {
    id,
    sourcePath,
    source,
    sha256: createHash("sha256").update(source).digest("hex"),
    playgroundUrl: "",
    highlighted: [{ text: source, className: "" }],
  }
}

export function moduleProjectExamples(playgroundUrl: string) {
  return [
    ...moduleProjects.flatMap((project) =>
      project.files.map((file) => {
        const id = moduleExampleId(project.key, file.key)
        const sourcePath = `${project.directory}/${file.path}`
        return file.path.endsWith(".toml")
          ? manifestExample(id, sourcePath)
          : canonicalExample(id, sourcePath, playgroundUrl, false)
      })
    ),
    ...moduleTypescriptProject.files.map((file) =>
      canonicalExample(
        moduleExampleId("typescript", file.key),
        `${moduleTypescriptProject.directory}/${file.path}`,
        playgroundUrl,
        false,
        "typescript"
      )
    ),
  ]
}
