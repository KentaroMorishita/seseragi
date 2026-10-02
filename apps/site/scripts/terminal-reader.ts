import { canonicalExample } from "./canonical-example"

// Process startup inputs need a native host; only the three Prelude programs
// have browser Playground seeds. All paired TypeScript runs target Node/Bun.
export const terminalReaderCases = [
  { slug: "println-status", portable: true, output: "Saved 3 files\n" },
  { slug: "print-progress", portable: true, output: "Status: saved\n" },
  { slug: "printvalue-count", portable: true, output: "Files: 3" },
  { slug: "environment", portable: false, output: "Mode: [preview]\n" },
  {
    slug: "arguments",
    portable: false,
    output:
      "Argument: [alpha]\nArgument: [two words]\nArgument: []\nArgument: [--preview]\nArgument: [設定]\n",
  },
  {
    slug: "current-directory",
    portable: false,
    output: "Directory: /tmp/seseragi-process-demo\n",
  },
  {
    slug: "process-failure",
    portable: false,
    output:
      "Use a nonempty setting name without a NUL character\nUse a nonempty setting name without a NUL character\n",
  },
] as const

// Preserve the compiler-owned namespace, kind and spelling of all nine routes.
export const terminalReaderRoutes = [
  [
    "std/prelude::println",
    "value",
    "function",
    "prelude-println",
    "println-status",
  ],
  [
    "std/prelude::print",
    "value",
    "function",
    "prelude-print",
    "print-progress",
  ],
  [
    "std/prelude::printValue",
    "value",
    "function",
    "prelude-printvalue",
    "printvalue-count",
  ],
  ["std/process", "module", "module", "process-module", "environment"],
  [
    "std/process::Process",
    "type",
    "opaque-type",
    "process-type",
    "environment",
  ],
  [
    "std/process::ProcessError",
    "type",
    "opaque-type",
    "process-error",
    "process-failure",
  ],
  [
    "std/process::environment",
    "value",
    "effect-function",
    "environment",
    "environment",
  ],
  [
    "std/process::arguments",
    "value",
    "effect-function",
    "arguments",
    "arguments",
  ],
  [
    "std/process::currentDirectory",
    "value",
    "effect-function",
    "current-directory",
    "current-directory",
  ],
].map(([identity, namespace, kind, slug, example]) => {
  const module = identity!.split("::")[0]!
  const name = identity!.split("::")[1] ?? module
  const modulePath = module.slice("std/".length)
  return {
    identity: identity!,
    module,
    namespace: namespace!,
    kind: kind!,
    name,
    slug: slug!,
    example: example!,
    route:
      namespace === "module"
        ? `/docs/library/${modulePath}/`
        : `/docs/library/${modulePath}/${kind}/${name.toLowerCase()}/`,
  }
})

export function terminalReaderExamples(playgroundUrl: string) {
  return terminalReaderCases.flatMap((item) => [
    canonicalExample(
      `terminal-reader-${item.slug}`,
      `apps/site/examples/src/terminal-reader/${item.slug}.ssrg`,
      playgroundUrl,
      item.portable
    ),
    canonicalExample(
      `terminal-reader-${item.slug}-ts`,
      `apps/site/examples/src/terminal-reader/${item.slug}.ts`,
      playgroundUrl,
      false,
      "typescript"
    ),
  ])
}
