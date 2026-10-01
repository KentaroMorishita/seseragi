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
import { dataChoiceExamples, dataChoices } from "../scripts/data-choices"
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

test("ten data questions keep their existing page identities and local bilingual explanations", () => {
  expect(dataChoices).toHaveLength(10)
  for (const item of dataChoices) {
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

test("complete data examples execute independently and rejected cases report the observed diagnostic", () => {
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-data-examples-"))
  try {
    for (const item of dataChoices) {
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
    const tsSource = readFileSync(
      join(root, "apps/site/examples/comparisons/delivery-state/typescript.ts"),
      "utf8"
    )
    const tsFile = join(temporary, "delivery.ts")
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
    expect(checked.status, checked.stdout || checked.stderr).toBe(0)
    const execution = run(process.execPath, [tsFile], temporary)
    expect(execution.status, execution.stderr).toBe(0)
    expect(execution.stdout).toBe("not shipped\ntracking: JP42\n")
    // The fair TS counterpart also catches the wrong payload and an omitted
    // branch under its explicit return type + strict settings.
    writeFileSync(
      tsFile,
      `${tsSource}\nlabel({ kind: "shipped", tracking: 42 })\n`
    )
    const wrongPayload = run(tsc, args, root)
    expect(wrongPayload.status).not.toBe(0)
    expect(wrongPayload.stdout).toContain("TS2322")
    writeFileSync(
      tsFile,
      tsSource.replace(/ {4}case "pending":\n {6}return "not shipped"\n/u, "")
    )
    const missingBranch = run(tsc, args, root)
    expect(missingBranch.status).not.toBe(0)
    expect(missingBranch.stdout).toContain("TS2366")
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})

test("twenty data locale pages render canonical first examples and preserve detailed sections", () => {
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-data-pages-"))
  try {
    const published = new Map(
      generatorInput(playground).examples.map((example) => [
        example.id,
        example,
      ])
    )
    const sources = new Map(
      dataChoiceExamples(playground).map((example) => {
        const registered = published.get(example.id)
        expect(registered, example.id).toEqual(example)
        return [example.id, registered ?? example]
      })
    )
    for (const item of dataChoices) {
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
        ...dataChoices.map((item) => `pages/language/${item.path}/page`),
        "render/document",
      ],
      entry: `import * as json from "std/json"
import { BuildInput, ExampleSource, HighlightPart } from "./model/build"
import { En, Ja, localizedRoute } from "./model/locale"
import { SiteCatalog } from "./model/page"
import { renderDocument } from "./render/document"
${dataChoices.map((item, i) => `import { page as page${i} } from "./pages/language/${item.path}/page"`).join("\n")}
struct Output deriving JsonEncode { route: String, html: String }
pub effect fn main = {
  let pages = [${dataChoices.map((_, i) => `page${i} ()`).join(", ")}]
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
    for (const item of dataChoices) {
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
          "types-constructors",
          "types-nominal-structural",
          "types-requirement-merge",
          "expressions-conditionals",
          "data-algebraic-data-types",
          "patterns-binding-rules",
          "patterns-irrefutable",
          "patterns-match",
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
        expect(text(html), route).toContain(item.output)
        expect(html).toContain('id="related-rules"')
        const oldSections = ["data/records", "data/structs"].includes(item.path)
          ? ["purpose", "reading-the-example", "rules", "mistakes"]
          : item.path.startsWith("types/")
            ? [
                "rule",
                "identity-and-representation",
                "checking-and-inference",
                "diagnostics",
              ]
            : ["rule", "typing", "evaluation", "diagnostics"]
        for (const section of oldSections)
          expect(html, route).toContain(`id="${section}"`)
        const alternate = `${prefix ? "" : "/ja"}/docs/language/${item.path}/`
        expect(html).toContain(`href="${alternate}"`)
        if (process.env.SESERAGI_DATA_OUTPUT) {
          const destination = join(
            resolve(process.env.SESERAGI_DATA_OUTPUT),
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
