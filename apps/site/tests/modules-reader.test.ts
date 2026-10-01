import { expect, setDefaultTimeout, test } from "bun:test"
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { dirname, join, resolve } from "node:path"
import { moduleProjectExamples } from "../scripts/module-examples"
import { assertReferenceLinkTitles, pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(180_000)
const root = resolve(import.meta.dir, "../../..")
const sourceRoot = join(root, "apps/site/src/pages/language")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const slugs = [
  "identity",
  "packages",
  "top-level",
  "visibility",
  "imports",
  "specifier-resolution",
  "re-exports",
  "namespaces-and-resolution",
  "dependency-graphs",
  "initialization",
  "entry-points",
] as const
const examples = moduleProjectExamples("https://seseragi.vercel.app/")
const quote = JSON.stringify
const fields = ["purpose", "reading", "rules", "mistakes", "related"] as const

function visible(html: string) {
  return html
    .replace(/<[^>]+>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
}
function title(path: string, locale: string) {
  const source = readFileSync(join(sourceRoot, path, `${locale}.ssrg`), "utf8")
  const literal = source.match(/\btitle:\s*("(?:[^"\\]|\\.)*")/u)?.[1]
  expect(literal, path).toBeDefined()
  return JSON.parse(literal ?? '""') as string
}
function primary(slug: string) {
  const guide = readFileSync(
    join(sourceRoot, "modules", slug, "guide.ssrg"),
    "utf8"
  )
  const group = guide.match(/ArticleExamples\s+\[([\s\S]*?)\]/u)?.[1]
  expect(group, slug).toBeDefined()
  return [...(group ?? "").matchAll(/"([^"]+)"/gu)].map((match) => match[1])
}

test("module articles select complete ordered project files without changing identities", () => {
  expect(slugs).toHaveLength(11)
  for (const slug of slugs) {
    const source = readFileSync(
      join(sourceRoot, "modules", slug, "page.ssrg"),
      "utf8"
    )
    expect(source).toContain(`"language.modules.${slug}"`)
    expect(source).toContain(`"/docs/language/modules/${slug}/"`)
    const ids = primary(slug)
    expect(ids.length).toBeGreaterThan(1)
    expect(new Set(ids).size).toBe(ids.length)
    for (const id of ids) {
      expect(source, slug).toContain(`"${id}"`)
      expect(
        examples.find((example) => example.id === id),
        id
      ).toBeDefined()
    }
  }
})

test("eleven module articles render whole canonical projects and paired local explanations", () => {
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-module-reader-"))
  try {
    const encodedExamples = examples
      .map(
        (example) => `ExampleSource {
  id: ${quote(example.id)}, sourcePath: ${quote(example.sourcePath)},
  source: ${quote(example.source)}, sha256: ${quote(example.sha256)},
  playgroundUrl: "", highlighted: [${example.highlighted
    .map(
      (part) =>
        `HighlightPart { text: ${quote(part.text)}, className: ${quote(part.className)} }`
    )
    .join(",")}]
}`
      )
      .join(",")
    const rendered = renderPageClosure<{
      pages: Array<{
        id: string
        locale: "en" | "ja"
        route: string
        title: string
        html: string
      }>
      counts: Array<{ id: string; english: number[]; japanese: number[] }>
    }>({
      directory: temporary,
      cli,
      timeoutMs: 150_000,
      modules: [
        ...slugs.map((slug) => `pages/language/modules/${slug}/page`),
        "render/document",
      ],
      entry: `import * as arrays from "std/array"
import * as json from "std/json"
import { BuildInput, ExampleSource, HighlightPart } from "./model/build"
import { En, Ja, localeCode, localizedRoute, translate } from "./model/locale"
import { SiteCatalog } from "./model/page"
import { ReaderCopy } from "./pages/language/reader-article"
${slugs
  .map(
    (
      slug,
      index
    ) => `import { page as page${index} } from "./pages/language/modules/${slug}/page"
import * as en${index} from "./pages/language/modules/${slug}/en"
import * as ja${index} from "./pages/language/modules/${slug}/ja"`
  )
  .join("\n")}
import { renderDocument } from "./render/document"
struct Rendered deriving JsonEncode { id: String, locale: String, route: String, title: String, html: String }
struct Counts deriving JsonEncode { id: String, english: Array<Int>, japanese: Array<Int> }
struct Report deriving JsonEncode { pages: Array<Rendered>, counts: Array<Counts> }
fn counts copy: ReaderCopy -> Array<Int> = [${fields.map((field) => `arrays.length copy.${field}`).join(",")}]
pub effect fn main = {
  let pages = [${slugs.map((_, index) => `page${index} ()`).join(",")}]
  let input = BuildInput { schema: 1, origin: "https://seseragi.example",
    playgroundUrl: "https://seseragi.vercel.app/", tourUrl: "https://seseragi.vercel.app/tour/",
    grammar: "", examples: [${encodedExamples}], referenceModules: [] }
  let site = SiteCatalog { home: page0 (), areas: [], pages }
  println (json.encodeString (Report {
    pages: [Rendered { id: page.id, locale: localeCode locale,
      route: localizedRoute locale page.path, title: translate locale page.title,
      html: renderDocument input site locale page } | locale <- [En, Ja], page <- pages],
    counts: [${slugs.map((slug, index) => `Counts { id: ${quote(slug)}, english: counts (en${index}.copy ()), japanese: counts (ja${index}.copy ()) }`).join(",")}]
  }))
}`,
    })
    expect(rendered.pages).toHaveLength(22)
    expect(new Set(rendered.pages.map((page) => page.route)).size).toBe(22)
    for (const counts of rendered.counts) {
      expect(counts.english, counts.id).toEqual(counts.japanese)
      expect(
        counts.english.every((count) => count > 0),
        counts.id
      ).toBe(true)
    }
    const titles = new Map<string, string>()
    for (const page of rendered.pages) {
      for (const [, destination] of page.html.matchAll(
        /href="((?:\/ja)?\/docs\/language\/[^"#]+\/)"/gu
      )) {
        const locale = destination.startsWith("/ja/") ? "ja" : "en"
        const path = destination
          .replace(/^\/(?:ja\/)?docs\/language\//u, "")
          .replace(/\/$/u, "")
        try {
          titles.set(destination, title(path, locale))
        } catch {
          // Section overviews are outside this eleven-page article closure.
          expect(path.endsWith("/overview"), destination).toBe(true)
        }
      }
    }
    let checkedLinks = 0
    for (const page of rendered.pages) {
      const slug = page.id.replace("language.modules.", "")
      const baseRoute = `/docs/language/modules/${slug}/`
      expect(page.route).toBe(
        `${page.locale === "ja" ? "/ja" : ""}${baseRoute}`
      )
      expect(pageTitle(page.html)).toBe(title(`modules/${slug}`, page.locale))
      expect(page.html).toContain(`<html lang="${page.locale}">`)
      expect(page.html).not.toContain("site-build-error")
      expect(page.html).not.toContain('class="playground-link"')
      expect(page.html).toContain(
        `href="${page.locale === "ja" ? "" : "/ja"}${baseRoute}"`
      )
      const headings = [
        ...page.html.matchAll(/<h2\b[^>]*\bid="([^"]+)"/gu),
      ].map((match) => match[1])
      expect(new Set(headings).size, page.route).toBe(headings.length)
      expect(headings).toContain("understand-this")
      expect(headings).toContain("related-rules")
      const panels = [
        ...page.html.matchAll(
          /<section class="code-panel">([\s\S]*?)<\/section>/gu
        ),
      ].map((match) => ({
        html: match[1],
        source: visible(
          match[1].match(
            /<code class="seseragi-highlight">([\s\S]*?)<\/code>/u
          )?.[1] ?? ""
        ),
      }))
      const expected = primary(slug).map(
        (id) => examples.find((example) => example.id === id)!
      )
      expect(
        panels.slice(0, expected.length).map((panel) => panel.source),
        page.route
      ).toEqual(expected.map((example) => example.source))
      for (const panel of panels) {
        const caption = visible(
          panel.html.match(
            /class="code-panel-title">([\s\S]*?)<\/span>/u
          )?.[1] ?? ""
        )
        expect(
          examples.some(
            (example) =>
              example.source === panel.source &&
              caption.endsWith(
                example.sourcePath.replace(
                  /^apps\/site\/examples\/projects\/modules\/[^/]+\//u,
                  ""
                )
              )
          ),
          `${page.route}: ${caption}: ${panel.source}`
        ).toBe(true)
      }
      expect(visible(page.html), page.route).toContain("seseragi lock update")
      expect(visible(page.html), page.route).toMatch(/seseragi (?:run|build)/u)
      checkedLinks += assertReferenceLinkTitles(page.html, titles, page.route)
      if (process.env.SESERAGI_MODULES_OUTPUT) {
        const destination = join(
          resolve(process.env.SESERAGI_MODULES_OUTPUT),
          page.route.slice(1),
          "index.html"
        )
        mkdirSync(dirname(destination), { recursive: true })
        writeFileSync(destination, page.html)
      }
    }
    expect(checkedLinks).toBeGreaterThanOrEqual(60)
    const find = (slug: string, locale: string) =>
      visible(
        rendered.pages.find(
          (page) =>
            page.id === `language.modules.${slug}` && page.locale === locale
        )!.html
      )
    expect(find("imports", "en")).toContain("export function greet")
    expect(find("imports", "ja")).toContain("Hello, Aki!")
    expect(find("packages", "en")).toContain("SES-N0104")
    expect(find("identity", "en")).toContain(
      "argument 1 expected UserId, received UserId"
    )
    expect(find("visibility", "en")).toContain("PrivateExport")
    expect(find("namespaces-and-resolution", "en")).toContain("SES-N0002")
    expect(find("initialization", "en")).toContain("SES-N0201")
    expect(find("dependency-graphs", "en")).toContain("SES-K0001")
    expect(find("entry-points", "en")).toContain("Unit")
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})
