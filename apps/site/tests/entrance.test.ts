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
import { comparisonExamples, entranceComparison } from "../scripts/comparisons"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(120_000)
const root = resolve(import.meta.dir, "../../..")
const examples = comparisonExamples("https://seseragi.vercel.app/")
const quote = JSON.stringify
const encodedExamples = examples
  .map(
    (example) => `ExampleSource {
    id: ${quote(example.id)}, sourcePath: ${quote(example.sourcePath)},
    source: ${quote(example.source)}, sha256: ${quote(example.sha256)},
    playgroundUrl: ${quote(example.playgroundUrl)},
    highlighted: [${example.highlighted
      .map(
        (part) => `HighlightPart {
      text: ${quote(part.text)}, className: ${quote(part.className)}
    }`
      )
      .join(",")}]
  }`
  )
  .join(",")

function text(html: string) {
  return html
    .replace(/<[^>]+>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
}
function panels(html: string) {
  return [
    ...html.matchAll(/<section class="code-panel">[\s\S]*?<\/section>/gu),
  ].map((match) => match[0])
}
function code(html: string) {
  return text(
    html.match(/<code class="seseragi-highlight">([\s\S]*?)<\/code>/u)?.[1] ??
      ""
  )
}

test("documentation titles wrap long identifiers without truncation", () => {
  const docs = readFileSync(join(root, "apps/site/styles/docs.css"), "utf8")
  const heading = docs.match(/\n\.page-intro h1\s*\{([^}]+)\}/u)?.[1] ?? ""
  // withTemporaryDirectory overflows the 500px layout unless the title can
  // break an otherwise unbreakable identifier. Keep ordinary word wrapping.
  expect(heading).toMatch(/overflow-wrap:\s*anywhere\s*;/u)
  expect(heading).not.toMatch(/word-break:\s*break-all\s*;/u)
  expect(heading).not.toMatch(/white-space:\s*nowrap\s*;/u)
  expect(heading).not.toMatch(/text-overflow:\s*ellipsis\s*;/u)
  expect(heading).not.toMatch(/overflow(?:-x)?:\s*(?:hidden|clip)\s*;/u)
})

test("article prose wraps long identifiers without changing code blocks", () => {
  const article = readFileSync(
    join(root, "apps/site/styles/article.css"),
    "utf8"
  )
  const prose =
    article.match(
      /\.article-content p,\s*\.article-content li\s*\{([^}]+)\}/u
    )?.[1] ?? ""
  // The Japanese Bytes Node note is plain <p><span> prose, not <code>.
  // Its Buffer.from(content).toString("hex") run overflowed at 500px.
  // Inline code also inherits this emergency wrapping from its paragraph.
  expect(prose).toMatch(/overflow-wrap:\s*anywhere\s*;/u)
  for (const rule of [prose, article]) {
    expect(rule).not.toMatch(/word-break:\s*break-all\s*;/u)
    expect(rule).not.toMatch(/white-space:\s*nowrap\s*;/u)
    expect(rule).not.toMatch(/text-overflow:\s*ellipsis\s*;/u)
    expect(rule).not.toMatch(/overflow(?:-x)?:\s*(?:hidden|clip)\s*;/u)
  }
  const code = readFileSync(join(root, "apps/site/styles/code.css"), "utf8")
  const pre =
    code.match(/\.code-panel pre,\s*\.api-reference pre\s*\{([^}]+)\}/u)?.[1] ??
    ""
  const block =
    code.match(
      /\.code-panel pre > code,\s*\.api-reference pre > code\s*\{([^}]+)\}/u
    )?.[1] ?? ""
  expect(pre).toMatch(/overflow:\s*auto\s*;/u)
  expect(block).toMatch(/min-width:\s*max-content\s*;/u)
  for (const rule of [pre, block]) {
    expect(rule).not.toMatch(/overflow-wrap:\s*(?:anywhere|break-word)\s*;/u)
    expect(rule).not.toMatch(/white-space:\s*(?:normal|pre-wrap|pre-line)\s*;/u)
    expect(rule).not.toMatch(/word-break:\s*break-all\s*;/u)
  }
})

test("the home hero starts both columns without inherited article spacing", () => {
  const home = readFileSync(join(root, "apps/site/styles/home.css"), "utf8")
  const responsive = readFileSync(
    join(root, "apps/site/styles/responsive.css"),
    "utf8"
  )
  const hero = home.match(/\.home-hero\s*\{([^}]+)\}/u)?.[1] ?? ""
  expect(hero).toMatch(/align-items:\s*start\s*;/u)
  expect(hero).toMatch(/grid-template-columns:\s*minmax\([^;]+\)\s*;/u)
  const article =
    home.match(/\.home-hero-content \.article-content\s*\{([^}]+)\}/u)?.[1] ??
    ""
  expect(article).toMatch(/padding-top:\s*0\s*;/u)
  const firstHeading =
    home.match(
      /\.home-hero-content \.article-content > h2:first-child\s*\{([^}]+)\}/u
    )?.[1] ?? ""
  expect(firstHeading).toMatch(/margin-top:\s*0\s*;/u)
  expect(firstHeading).toMatch(/padding-top:\s*0\s*;/u)
  expect(responsive).toMatch(
    /@media \(max-width: 960px\)[\s\S]*?\.home-hero\s*\{[^}]*grid-template-columns:\s*1fr\s*;/u
  )
  for (const [, rule] of responsive.matchAll(/\.home-hero\s*\{([^}]+)\}/gu))
    expect(rule).not.toMatch(/align-items:\s*center\s*;/u)
})

test("entrance pages render verified comparisons and a Docs-first starting path in both locales", () => {
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-entrance-"))
  try {
    const rendered = renderPageClosure<Array<{ route: string; html: string }>>({
      directory: temporary,
      modules: [
        "pages/home/page",
        "pages/docs/overview/page",
        "pages/language/model/what-is-seseragi/page",
        "pages/language/overview/page",
        "pages/library/overview/page",
        "render/document",
      ],
      entry: `import * as json from "std/json"
import { BuildInput, ExampleSource, HighlightPart } from "./model/build"
import { En, Ja, localized, localizedRoute } from "./model/locale"
import { FlatSection, NavigationArea, NavigationGroup, NavigationSection, SiteCatalog } from "./model/page"
import { page as homePage } from "./pages/home/page"
import { page as docsPage } from "./pages/docs/overview/page"
import { page as introPage } from "./pages/language/model/what-is-seseragi/page"
import { page as languagePage } from "./pages/language/overview/page"
import { page as libraryPage } from "./pages/library/overview/page"
import { renderDocument } from "./render/document"
struct Output deriving JsonEncode { route: String, html: String }
pub effect fn main = {
  let home = homePage ()
  let docs = docsPage ()
  let intro = introPage ()
  let language = languagePage ()
  let library = libraryPage ()
  let input = BuildInput {
    schema: 1, origin: "https://seseragi.example",
    playgroundUrl: "https://seseragi.vercel.app/",
    tourUrl: "https://seseragi.vercel.app/tour/", grammar: "",
    examples: [${encodedExamples}], referenceModules: []
  }
  let groups = [NavigationGroup {
    id: "understand", title: localized "Understand Seseragi" "Seseragiを知る",
    sections: [NavigationSection {
      id: "model", title: intro.title, kind: FlatSection,
      overview: intro, pages: []
    }]
  }]
  let site = SiteCatalog {
    home,
    areas: [
      NavigationArea { id: "language", title: language.title,
        description: language.summary, landing: language, groups },
      NavigationArea { id: "library", title: library.title,
        description: library.summary, landing: library, groups: [] }
    ], pages: [home, docs, intro, language]
  }
  println (json.encodeString [Output {
    route: localizedRoute locale page.path,
    html: renderDocument input site locale page
  } | locale <- [En, Ja], page <- site.pages])
}`,
    })
    const pages = Object.fromEntries(
      rendered.map(({ route, html }) => [route, html])
    )
    expect(rendered).toHaveLength(8)
    const ts = readFileSync(join(root, entranceComparison.typescript), "utf8")
    const seseragi = readFileSync(
      join(root, entranceComparison.seseragi),
      "utf8"
    )
    for (const prefix of ["", "/ja"]) {
      const home = pages[`${prefix}/`]
      const docs = pages[`${prefix}/docs/`]
      const intro = pages[`${prefix}/docs/language/model/what-is-seseragi/`]
      const language = pages[`${prefix}/docs/language/`]
      for (const html of [home, docs, intro, language]) {
        expect(html).not.toContain("site-build-error")
        expect(html).toContain(`href="${prefix}/docs/first-run/"`)
      }
      expect(panels(home)).toHaveLength(1)
      expect(code(panels(home)[0])).toBe(seseragi)
      const pair = panels(intro)
      expect(pair).toHaveLength(2)
      expect(code(pair[0])).toBe(ts)
      expect(code(pair[1])).toBe(seseragi)
      expect(pair[0]).not.toContain('class="playground-link"')
      expect(pair[1]).toContain('class="playground-link"')
      expect(text(intro)).toContain('standardTotal "3200"')
      expect(text(intro)).not.toContain('standardTotal \\"3200')
      expect(text(intro)).toContain("3700, 5000")
      expect(text(intro)).toContain("1,000,000")
      expect(intro.indexOf('id="read-function"')).toBeLessThan(
        intro.indexOf('id="reuse-conditions"')
      )
      expect(intro.indexOf('id="reuse-conditions"')).toBeLessThan(
        intro.indexOf('id="consequences"')
      )
      expect(docs).not.toContain("docs-sidebar")
      expect(docs.match(/class="reference-area-card"/gu)).toHaveLength(2)
      expect(docs.indexOf('id="start-here"')).toBeLessThan(
        docs.indexOf('class="reference-area-grid"')
      )
      expect(docs.indexOf('id="first-run"')).toBeLessThan(
        docs.indexOf('id="learn-by-doing"')
      )
      expect(language.indexOf('id="values-and-functions"')).toBeLessThan(
        language.indexOf('class="reference-directory"')
      )
      for (const suffix of [
        "model/immutable-by-default",
        "types/annotations-and-inference",
        "syntax/function-application",
      ]) {
        expect(intro).toContain(`href="${prefix}/docs/language/${suffix}/"`)
        expect(language).toContain(`href="${prefix}/docs/language/${suffix}/"`)
      }
      expect(docs).toContain(
        `href="${prefix}/docs/language/model/what-is-seseragi/"`
      )
      const other = prefix ? "" : "/ja"
      for (const route of [
        "/",
        "/docs/",
        "/docs/language/",
        "/docs/language/model/what-is-seseragi/",
      ])
        expect(pages[`${prefix}${route}`]).toContain(`href="${other}${route}"`)
    }
    expect(text(pages["/docs/"])).toContain("The optional Tour")
    expect(text(pages["/ja/docs/"])).toContain("このドキュメントだけで読めます")
    expect(text(pages["/docs/language/model/what-is-seseragi/"])).toContain(
      "not a complete payment implementation"
    )
    expect(text(pages["/ja/docs/language/model/what-is-seseragi/"])).toContain(
      "完成した決済処理ではありません"
    )
    if (process.env.SESERAGI_ENTRANCE_OUTPUT) {
      for (const { route, html } of rendered) {
        const destination = join(
          resolve(process.env.SESERAGI_ENTRANCE_OUTPUT),
          route,
          "index.html"
        )
        mkdirSync(dirname(destination), { recursive: true })
        writeFileSync(destination, html)
      }
    }
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})
