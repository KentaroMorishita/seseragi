import { expect, test } from "bun:test"
import { spawnSync } from "node:child_process"
import {
  existsSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import { discoverAuthoringFiles } from "../scripts/content/discover"
import { parseAuthoringFile } from "../scripts/content/parse"
import {
  prepareContent,
  repositoryContentRoot,
  toLegacyPages,
} from "../scripts/content/prepare"
import { validatePages } from "../scripts/content/validate"

const root = resolve(import.meta.dir, "../../..")
const contentRoot = resolve(import.meta.dir, "../content")
const corpus = () => prepareContent({ repositoryRoot: root, contentRoot })

test("file-per-page authoring prepares one bilingual semantic Doc AST", () => {
  const prepared = corpus()
  expect(prepared.schema).toBe(1)
  expect(prepared.defaultLocale).toBe("en")
  expect(prepared.locales).toEqual(["en", "ja"])
  expect(prepared.pages).toHaveLength(8)
  expect(new Set(prepared.pages.map(({ metadata }) => metadata.id))).toEqual(
    new Set([
      "home",
      "docs.overview",
      "language.syntax.function-application",
      "library.array.get",
    ])
  )
  for (const page of prepared.pages) {
    expect(page.spec.every(({ sha256 }) => sha256.length === 64)).toBe(true)
  }
  for (const page of prepared.pages.filter(
    ({ metadata }) => metadata.id === "language.syntax.function-application"
  )) {
    expect(page.spec.map(({ section }) => section)).toEqual([
      "1.4",
      "2.7",
      "3.1",
    ])
    const example = page.blocks.find(({ kind }) => kind === "example")
    expect(example?.source).toBe(
      "examples/spec/lessons/02-values-and-functions.ssrg"
    )
    expect(example?.sha256).toHaveLength(64)
    expect(example?.text).toContain("fn add left: Int")
  }
})

test("the compatibility adapter keeps the released renderer buildable", () => {
  const legacy = toLegacyPages(corpus(), "en")
  expect(legacy).toHaveLength(4)
  const functionPage = legacy.find(
    ({ route }) => route === "/docs/language/syntax/function-application/"
  )!
  expect(functionPage.blocks).toContainEqual({
    kind: "sample",
    source: "examples/spec/lessons/02-values-and-functions.ssrg",
  })
})

test("authoring rejects open directives, copied Seseragi code and raw HTML", () => {
  const metadata = JSON.stringify({})
  expect(() =>
    parseAuthoringFile(
      `---json\n${metadata}\n---\n# Page\n\n:::invented\nText\n:::\n`,
      "page.md"
    )
  ).toThrow("unknown directive invented")
  expect(() =>
    parseAuthoringFile(
      `---json\n${metadata}\n---\n# Page\n\n\`\`\`seseragi\nfn copied -> Int = 1\n\`\`\`\n`,
      "page.md"
    )
  ).toThrow("must use a canonical example directive")
  expect(() =>
    parseAuthoringFile(
      `---json\n${metadata}\n---\n# Page\n\n<script>bad</script>\n`,
      "page.md"
    )
  ).toThrow("raw HTML is not allowed")
})

test("locale pairs, graph edges, links and semantic provenance are closed", () => {
  const pagesRoot = join(contentRoot, "pages")
  const pages = discoverAuthoringFiles(contentRoot).map((path) =>
    parseAuthoringFile(readFileSync(path, "utf8"), path)
  )
  const english = pages.find(
    ({ metadata }) =>
      metadata.locale === "en" &&
      metadata.id === "language.syntax.function-application"
  )!
  const japanese = pages.find(
    ({ metadata }) =>
      metadata.locale === "ja" &&
      metadata.id === "language.syntax.function-application"
  )!
  const isolate = (page: typeof english) => ({
    ...page,
    metadata: {
      ...page.metadata,
      prerequisites: [],
      related: [],
      next: "",
    },
  })
  const isolatedEnglish = isolate(english)
  const isolatedJapanese = isolate(japanese)

  expect(() => validatePages([isolatedEnglish], pagesRoot)).toThrow(
    "missing locale file"
  )
  expect(() =>
    validatePages(
      [
        {
          ...isolatedEnglish,
          metadata: { ...isolatedEnglish.metadata, route: "/Docs/invalid/" },
        },
        isolatedJapanese,
      ],
      pagesRoot
    )
  ).toThrow("invalid route")

  expect(() =>
    validatePages(
      [
        {
          ...isolatedEnglish,
          metadata: { ...isolatedEnglish.metadata, related: ["missing.page"] },
        },
        {
          ...isolatedJapanese,
          metadata: { ...isolatedJapanese.metadata, related: ["missing.page"] },
        },
      ],
      pagesRoot
    )
  ).toThrow("unknown graph page missing.page")

  const unsafeLink = {
    ...isolatedEnglish,
    blocks: [
      ...isolatedEnglish.blocks,
      {
        ...isolatedEnglish.blocks[0],
        inlines: [
          { kind: "link" as const, text: "bad", href: "javascript:bad" },
        ],
      },
    ],
  }
  expect(() =>
    validatePages([unsafeLink, isolatedJapanese], pagesRoot)
  ).toThrow()

  const unknownExample = {
    ...isolatedEnglish,
    blocks: isolatedEnglish.blocks.map((block) =>
      block.kind === "example"
        ? { ...block, source: "examples/spec/missing.ssrg" }
        : block
    ),
  }
  expect(() =>
    validatePages([unknownExample, isolatedJapanese], pagesRoot)
  ).toThrow("example directive is absent from metadata")

  const unsafeExample = "examples/spec/../secret.ssrg"
  expect(() =>
    validatePages(
      [
        {
          ...isolatedEnglish,
          metadata: { ...isolatedEnglish.metadata, examples: [unsafeExample] },
          blocks: isolatedEnglish.blocks.map((block) =>
            block.kind === "example"
              ? { ...block, source: unsafeExample }
              : block
          ),
        },
        {
          ...isolatedJapanese,
          metadata: { ...isolatedJapanese.metadata, examples: [unsafeExample] },
          blocks: isolatedJapanese.blocks.map((block) =>
            block.kind === "example"
              ? { ...block, source: unsafeExample }
              : block
          ),
        },
      ],
      pagesRoot
    )
  ).toThrow("invalid canonical example path")
})

test("the Seseragi authoring model decodes the prepared corpus", () => {
  const cli =
    process.env.SESERAGI_BIN ?? resolve(root, "target", "debug", "seseragi")
  expect(existsSync(cli)).toBe(true)
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-doc-ast-"))
  try {
    const artifact = join(temporary, "artifact")
    const source = join(temporary, "main.ssrg")
    const model = readFileSync(
      join(root, "apps/docs/src/model/document.ssrg"),
      "utf8"
    )
    const probe = readFileSync(
      join(root, "apps/docs/src/authoring-probe.ssrg"),
      "utf8"
    )
      .replace(
        'import { DocCorpus, decodeCorpus } from "./model/document"\n',
        ""
      )
      .replace('import * as json from "std/json"\n', "")
    writeFileSync(source, `${model}\n${probe}`)
    const build = Bun.spawnSync(
      [
        cli,
        "build",
        source,
        "--target",
        "process",
        "--profile",
        "development",
        "--source-map",
        "omit",
        "--out-dir",
        artifact,
      ],
      { cwd: root, stderr: "pipe", stdout: "pipe" }
    )
    expect(build.exitCode, build.stderr.toString()).toBe(0)
    const manifest = JSON.parse(
      readFileSync(join(artifact, "artifact-manifest.json"), "utf8")
    )
    const run = spawnSync("bun", [manifest.entry], {
      cwd: artifact,
      input: `${JSON.stringify(corpus())}\n`,
      encoding: "utf8",
    })
    expect(run.status, run.stderr).toBe(0)
    expect(JSON.parse(run.stdout)).toEqual({ schema: 1, pages: 8 })
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})

test("repositoryContentRoot resolves the checked-in authoring tree", () => {
  expect(
    repositoryContentRoot(resolve(import.meta.dir, "../scripts/content"))
  ).toEqual({ repositoryRoot: root, contentRoot })
})
