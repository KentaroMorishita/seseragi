import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { dirname, join, resolve } from "node:path"
import { generatorInput } from "../scripts/build"
import {
  typeComparisons,
  typeReaderExamples,
  typeReaders,
} from "../scripts/type-readers"

const typeCases = typeReaders.map((entry) => ({
  ...entry,
  path: `types/${entry.path}`,
  id:
    entry.path === "type-identity-and-coercion"
      ? "language.types.identity-coercion"
      : `language.types.${entry.path}`,
  source: `type-reader-${entry.example}`,
  example: `type-reader-${entry.example}`,
  invalid: `type-reader-${entry.example}`,
  invalidId: `type-reader-${entry.example}-invalid`,
}))

import { renderPageClosure } from "./render-page-closure"

const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const validRoot = "apps/site/examples/src/language"
const invalidRoot = "apps/site/examples/invalid/src/language"
const playground = "https://seseragi.vercel.app/"
setDefaultTimeout(240_000)

function run(command: string, args: string[], cwd: string) {
  return spawnSync(command, args, {
    cwd,
    encoding: "utf8",
    timeout: 90_000,
    maxBuffer: 16 * 1024 * 1024,
  })
}
function text(html: string) {
  return html
    .replace(/<[^>]+>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
}

test("ten type-reuse questions keep their existing page identities and local bilingual explanations", () => {
  expect(typeCases).toHaveLength(10)
  for (const item of typeCases) {
    const directory = join(root, "apps/site/src/pages/language", item.path)
    const source = readFileSync(join(directory, "page.ssrg"), "utf8")
    expect(source).toContain(`"${item.id}"`)
    expect(source).toContain(`"/docs/language/${item.path}/"`)
    expect(source).toContain("let page = detailed ()")
    expect(source).toContain("page.blocks")
    for (const locale of ["en", "ja"]) {
      const copy = readFileSync(join(directory, `${locale}.ssrg`), "utf8")
      for (const field of ["question", "summary", "reading", "result"])
        expect(copy).toMatch(new RegExp(`  ${field}: "[^\"]`))
      for (const field of [
        "readerBridgeOne",
        "readerBridgeTwo",
        "readerMistake",
      ])
        expect(copy).toContain(`pub fn ${field} -> String`)
    }
  }
})

test("complete type-reuse examples execute independently and rejected cases report the observed diagnostic", () => {
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-type-examples-"))
  try {
    for (const item of typeCases) {
      const source = readFileSync(
        join(root, validRoot, `${item.source}.ssrg`),
        "utf8"
      )
      writeFileSync(join(temporary, "main.ssrg"), source)
      const valid = run(cli, ["run", "main.ssrg"], temporary)
      expect(valid.status, `${item.path}: ${valid.stderr}`).toBe(0)
      expect(valid.stderr).toBe("")
      expect(valid.stdout, item.path).toBe(`${item.output}\n`)
      const invalidSource = readFileSync(
        join(root, invalidRoot, `${item.invalid}.ssrg`),
        "utf8"
      )
      writeFileSync(join(temporary, "main.ssrg"), invalidSource)
      const invalid = run(cli, ["lint", "main.ssrg"], temporary)
      expect(invalid.status, item.path).not.toBe(0)
      expect(invalid.stderr, item.path).toContain(item.diagnostic)
    }

    for (const item of typeComparisons) {
      const tsSource = readFileSync(
        join(
          root,
          "apps/site/examples/comparisons",
          item.directory,
          "typescript.ts"
        ),
        "utf8"
      )
      const tsFile = join(temporary, "comparison.ts")
      writeFileSync(tsFile, tsSource)
      const tsc = join(root, "node_modules/.bin/tsc")
      const args = [
        "--noEmit",
        "--skipLibCheck",
        "--strict",
        "--target",
        "ES2022",
        "--module",
        "Preserve",
        "--moduleResolution",
        "Bundler",
        tsFile,
      ]
      const checked = run(tsc, args, root)
      expect(
        checked.status,
        `${item.directory}: ${checked.stdout || checked.stderr}`
      ).toBe(0)
      const execution = run(process.execPath, [tsFile], temporary)
      expect(execution.status, execution.stderr).toBe(0)
      expect(execution.stdout).toBe(`${item.output}\n`)
      writeFileSync(tsFile, `${tsSource}${item.mutation}`)
      const rejected = run(tsc, args, root)
      expect(rejected.status).not.toBe(0)
      expect(rejected.stdout).toContain(item.diagnostic)
    }
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})

test("twenty type-reuse locale pages render canonical first examples and preserve detailed sections", () => {
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-type-pages-"))
  try {
    const published = new Map(
      generatorInput(playground).examples.map((example) => [
        example.id,
        example,
      ])
    )
    const sources = new Map(
      typeReaderExamples(playground).map((example) => {
        const registered = published.get(example.id)
        expect(registered, example.id).toEqual(example)
        return [example.id, registered ?? example]
      })
    )
    for (const item of typeCases) {
      const pageSource = readFileSync(
        join(root, "apps/site/src/pages/language", item.path, "page.ssrg"),
        "utf8"
      )
      const ids = [
        item.example,
        item.invalidId,
        ...[...pageSource.matchAll(/CodeExample\s*\(\s*"([^"]+)"/gu)].map(
          (match) => match[1]
        ),
      ]
      for (const id of ids) {
        if (sources.has(id)) continue
        const example = published.get(id)
        expect(example, id).toBeDefined()
        if (example) sources.set(id, example)
      }
    }
    const examples = [...sources.values()]
      .map(
        (example) => `ExampleSource {
      id: ${JSON.stringify(example.id)}, sourcePath: ${JSON.stringify(example.sourcePath)}, source: ${JSON.stringify(example.source)},
      sha256: ${JSON.stringify(example.sha256)}, playgroundUrl: ${JSON.stringify(example.playgroundUrl)},
      highlighted: [${example.highlighted.map((part) => `HighlightPart { text: ${JSON.stringify(part.text)}, className: ${JSON.stringify(part.className)} }`).join(",")}]
    }`
      )
      .join(",")
    const pages = renderPageClosure<Array<{ route: string; html: string }>>({
      directory: temporary,
      timeoutMs: 180_000,
      modules: [
        ...typeCases.map((item) => `pages/language/${item.path}/page`),
        "render/document",
      ],
      entry: `import * as json from "std/json"
import { BuildInput, ExampleSource, HighlightPart } from "./model/build"
import { En, Ja, localizedRoute } from "./model/locale"
import { SiteCatalog } from "./model/page"
import { renderDocument } from "./render/document"
${typeCases.map((item, i) => `import { page as page${i} } from "./pages/language/${item.path}/page"`).join("\n")}
struct Output deriving JsonEncode { route: String, html: String }
pub effect fn main = {
  let pages = [${typeCases.map((_, i) => `page${i} ()`).join(", ")}]
  let input = BuildInput {
    schema: 1, origin: "https://seseragi.example", playgroundUrl: "${playground}",
    tourUrl: "https://seseragi.vercel.app/tour/", grammar: "", referenceModules: [], examples: [${examples}]
  }
  let site = SiteCatalog { home: page0 (), areas: [], pages }
  println (json.encodeString [Output { route: localizedRoute locale page.path, html: renderDocument input site locale page }
    | locale <- [En, Ja], page <- site.pages])
}`,
    })
    expect(pages).toHaveLength(20)
    for (const item of typeCases) {
      for (const prefix of ["", "/ja"]) {
        const route = `${prefix}/docs/language/${item.path}/`
        const html = pages.find((page) => page.route === route)?.html ?? ""
        expect(html, route).not.toContain("site-build-error")
        const firstCode =
          html.match(
            /<code class="seseragi-highlight">([\s\S]*?)<\/code>/u
          )?.[1] ?? ""
        const panels = [
          ...html.matchAll(/<section class="code-panel">[\s\S]*?<\/section>/gu),
        ].map((match) => match[0])
        expect(panels[0], route).toContain('class="playground-link"')
        const fragmentIds = [
          "types-system",
          "syntax-optional-record-field",
          "types-generic-aliases",
          "types-newtypes",
          "types-generic-functions",
          "types-polymorphism",
          "types-scope",
          "types-generic-structs",
          "types-generic-adts",
        ]
        for (const fragmentId of fragmentIds) {
          const fragment = published.get(fragmentId)
          if (!fragment) continue
          const panel = panels.find(
            (candidate) =>
              text(
                candidate.match(
                  /<code class="seseragi-highlight">([\s\S]*?)<\/code>/u
                )?.[1] ?? ""
              ) === fragment.source
          )
          if (panel) {
            expect(fragment.playgroundUrl).toBe("")
            expect(panel).not.toContain('class="playground-link"')
          }
        }
        expect(text(firstCode), route).toBe(
          readFileSync(join(root, validRoot, `${item.source}.ssrg`), "utf8")
        )
        for (const id of [
          item.example,
          item.invalidId,
          ...typeComparisons
            .filter(
              (comparison) =>
                comparison.example === item.example.replace("type-reader-", "")
            )
            .map(
              (comparison) => `type-reader-${comparison.example}-typescript`
            ),
        ]) {
          const expected = sources.get(id)
          expect(expected, id).toBeDefined()
          expect(
            panels.some(
              (panel) =>
                text(
                  panel.match(
                    /<code class="seseragi-highlight">([\s\S]*?)<\/code>/u
                  )?.[1] ?? ""
                ) === expected?.source
            ),
            `${route}: ${id}`
          ).toBe(true)
        }
        expect(text(html), route).toContain(item.output)
        expect(html).toContain('id="related-rules"')
        const oldSections = [
          "rule",
          "identity-and-representation",
          "checking-and-inference",
          "diagnostics",
          "declaration-example",
        ]
        for (const section of oldSections)
          expect(html, route).toContain(`id="${section}"`)
        const alternate = `${prefix ? "" : "/ja"}/docs/language/${item.path}/`
        expect(html).toContain(`href="${alternate}"`)
        if (process.env.SESERAGI_TYPES_OUTPUT) {
          const destination = join(
            resolve(process.env.SESERAGI_TYPES_OUTPUT),
            route,
            "index.html"
          )
          mkdirSync(dirname(destination), { recursive: true })
          writeFileSync(destination, html)
        }
      }
    }
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})
