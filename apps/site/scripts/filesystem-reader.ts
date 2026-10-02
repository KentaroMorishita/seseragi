import { canonicalExample } from "./canonical-example"

// These process programs own their temporary files. A web build alone does not
// establish a FileSystem provider, so only the three pure Path cases have seeds.
export const filesystemReaderCases = [
  {
    slug: "path-basics",
    filesystem: false,
    output: "Saving reports/report.txt\n",
  },
  {
    slug: "parse-inputs",
    filesystem: false,
    output:
      "accepted: reports/./report.txt\nrejected: EmptyPath\nrejected: PathContainsBackslash { offset: 2 }\n",
  },
  {
    slug: "child-filenames",
    filesystem: false,
    output:
      "reports/report.txt\nreports/report.v2.txt\nrejected: ../report.txt\nrejected: \nrejected: .\nrejected: ..\n",
    typescript: "child-filenames",
    typescriptOutput:
      "reports/report.txt\nreports/report.v2.txt\nrejected: ../report.txt\nrejected: \nrejected: .\nrejected: ..\n",
  },
  {
    slug: "report-roundtrip",
    filesystem: true,
    output: "café\nこんにちは\n",
  },
  {
    slug: "write-modes",
    filesystem: true,
    output:
      "before: False\nduplicate: FileAlreadyExists\noriginal: original\nupdated: OK!\n",
    typescript: "write-modes",
    typescriptOutput:
      "before: false\nduplicate: EEXIST\noriginal: original\nupdated: OK!\n",
  },
  {
    slug: "create-new",
    filesystem: true,
    output:
      "operation: WriteFile\nkind: FileAlreadyExists\npath matches report: True\nother path: none\ncontents: original\n",
  },
  { slug: "replace", filesystem: true, output: "OK\n" },
  { slug: "append", filesystem: true, output: "first\nsecond\ntail\n" },
  {
    slug: "strict-utf8",
    filesystem: true,
    output:
      "access: FileNotFound\ndecode: InvalidUtf8 { offset: 1 }\nBOM preserved: True\n",
    typescript: "strict-utf8",
    typescriptOutput:
      "access: ENOENT\ndecode: TypeError (no byte offset)\nBOM preserved: true\n",
  },
  {
    slug: "temporary-workspace",
    filesystem: true,
    output: "success: ready\ncallback failure: report rejected\n",
    typescript: "temporary-workspace",
    typescriptOutput: "success: ready\ncallback failure: report rejected\n",
  },
] as const

// Exact compiler-owned identities, not new routes or renamed opaque types.
export const filesystemReaderRoutes = [
  ["std/path", "module", "module", "path-module", "path-basics", ""],
  ["std/path::Path", "type", "opaque-type", "path", "path-basics", ""],
  ["std/path::parse", "value", "function", "parse", "parse-inputs", ""],
  ["std/path::render", "value", "function", "render", "path-basics", ""],
  [
    "std/path::child",
    "value",
    "function",
    "child",
    "child-filenames",
    "child-filenames",
  ],
  ["std/fs", "module", "module", "fs-module", "report-roundtrip", ""],
  [
    "std/fs::FileSystem",
    "type",
    "opaque-type",
    "filesystem",
    "report-roundtrip",
    "",
  ],
  [
    "std/fs::FileSystemError",
    "type",
    "struct",
    "filesystemerror",
    "create-new",
    "",
  ],
  [
    "std/fs::FileTextError",
    "type",
    "opaque-type",
    "filetexterror",
    "strict-utf8",
    "strict-utf8",
  ],
  [
    "std/fs::WriteMode",
    "type",
    "opaque-type",
    "writemode",
    "write-modes",
    "write-modes",
  ],
  ["std/fs::CreateNew", "value", "constructor", "createnew", "create-new", ""],
  ["std/fs::Replace", "value", "constructor", "replace", "replace", ""],
  ["std/fs::Append", "value", "constructor", "append", "append", ""],
  [
    "std/fs::readTextUtf8",
    "value",
    "effect-function",
    "readtextutf8",
    "report-roundtrip",
    "",
  ],
  [
    "std/fs::writeTextUtf8",
    "value",
    "effect-function",
    "writetextutf8",
    "report-roundtrip",
    "",
  ],
  [
    "std/fs::withTemporaryDirectory",
    "value",
    "effect-function",
    "withtemporarydirectory",
    "temporary-workspace",
    "temporary-workspace",
  ],
].map(([identity, namespace, kind, slug, example, comparison]) => {
  const module = identity!.split("::")[0]!
  const name = identity!.split("::")[1] ?? ""
  const modulePath = module.slice("std/".length)
  return {
    identity: identity!,
    module,
    name,
    namespace: namespace!,
    kind: kind!,
    slug: slug!,
    example: example!,
    comparison: comparison!,
    route:
      namespace === "module"
        ? `/docs/library/${modulePath}/`
        : `/docs/library/${modulePath}/${kind}/${name.toLowerCase()}/`,
  }
})

export const filesystemReaderFailures = [
  {
    slug: "string-as-path",
    diagnostics: ["SES-T0101", "This position requires an Effect value"],
    correction: "report-roundtrip",
  },
  {
    slug: "string-as-path-annotation",
    diagnostics: ["SES-T0101", "Binding annotation type mismatch"],
    correction: "path-basics",
  },
] as const

export function filesystemReaderExamples(playgroundUrl: string) {
  return filesystemReaderCases.flatMap((item) => {
    const source = canonicalExample(
      `filesystem-reader-${item.slug}`,
      `apps/site/examples/src/filesystem-reader/${item.slug}.ssrg`,
      playgroundUrl,
      !item.filesystem
    )
    return "typescript" in item
      ? [
          source,
          canonicalExample(
            `filesystem-reader-${item.typescript}-ts`,
            `apps/site/examples/comparisons/filesystem-reader/${item.typescript}.ts`,
            playgroundUrl,
            false,
            "typescript"
          ),
        ]
      : [source]
  })
}
