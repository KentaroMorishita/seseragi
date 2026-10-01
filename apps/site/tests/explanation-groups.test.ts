import { beforeAll, expect, setDefaultTimeout, test } from "bun:test"
import { mkdtempSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(120_000)

const sources = [
  { id: "model-日本", source: 'pub fn label -> String = "定義 < & 😀"\n' },
  {
    id: "consumer",
    source: 'import { label } from "./model"\nprintln (label ())\n',
  },
  { id: "unselected", source: 'println "Later café"\n' },
]
const input = {
  schema: 1,
  origin: "https://seseragi.example",
  playgroundUrl: "https://seseragi.vercel.app/",
  tourUrl: "https://seseragi.vercel.app/tour/",
  grammar: "",
  examples: sources.map(({ id, source }) => ({
    id,
    source,
    sourcePath: `${id}.ssrg`,
    sha256: "fixture",
    playgroundUrl:
      id === "consumer"
        ? ""
        : `https://seseragi.vercel.app/?source=${encodeURIComponent(source)}`,
    highlighted: [{ text: source, className: "fixture-token" }],
  })),
  referenceModules: [],
}

type Rendered = {
  name: string
  locale: "en" | "ja"
  html: string
  originalHtml: string
  legacyHtml: string
  validation: string
  selected: string[]
}
let rendered: Rendered[]

function text(html: string) {
  return html
    .replace(/<[^>]+>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
}
function article(html: string) {
  const content = html.match(
    /<article class="article-content">([\s\S]*?)<\/article>/u
  )?.[1]
  expect(content).toBeDefined()
  return content!
}
function panels(html: string) {
  return [
    ...article(html).matchAll(
      /<section class="code-panel">[\s\S]*?<\/section>/gu
    ),
  ].map((match) => match[0])
}
function code(panel: string) {
  return text(
    panel.match(/<code class="seseragi-highlight">([\s\S]*?)<\/code>/u)?.[1] ??
      ""
  )
}
function output(name: string, locale: "en" | "ja") {
  const found = rendered.find(
    (page) => page.name === name && page.locale === locale
  )
  expect(found, `${name}: ${locale}`).toBeDefined()
  return found!
}

beforeAll(() => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-explanation-groups-"))
  try {
    rendered = renderPageClosure<Rendered[]>({
      directory,
      modules: ["pages/language/explanation", "render/document"],
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { En, Ja, localized, localeCode } from "./model/locale"
import {
  Article, Block, CodeExample, DocumentationIndex, DocumentationLanding,
  FlatSection, Heading, Home, NavigationArea, NavigationGroup,
  NavigationSection, PageDefinition, PageKind, Paragraph, SiteCatalog,
  SiteLanding, Terminal, Words
} from "./model/page"
import {
  ArticleExample, ArticleExamples, ExampleSelection, Explanation,
  ExplanationCopy, WithoutExample, explainPage, makeExplanation,
  selectArticleExamples
} from "./pages/language/explanation"
import { renderDocument } from "./render/document"

// Direct legacy oracle: the pre-group helper's find/filter/layout behavior.
// It deliberately does not call the new validator or explainPage.
type LegacySelection =
  | LegacyWithoutExample
  | LegacyArticleExample String
fn legacyIsExample id: String -> block: Block -> Bool = match block {
  CodeExample (exampleId, _) -> exampleId == id
  _ -> False
}
fn legacyKeep id: LegacySelection -> block: Block -> Bool = match id {
  LegacyWithoutExample -> True
  LegacyArticleExample value -> !(legacyIsExample value block)
}
fn legacyExplain selection: LegacySelection -> explanation: Explanation
  -> page: PageDefinition -> PageDefinition = {
  let source = match selection {
    LegacyWithoutExample -> {
      let empty: Array<Block> = []
      empty
    }
    LegacyArticleExample id -> match arrays.find (legacyIsExample id) page.blocks {
      Nothing -> {
        let empty: Array<Block> = []
        empty
      }
      Just block -> [block]
    }
  }
  PageDefinition {
    ...page, summary: explanation.summary,
    blocks: arrays.concat [
      [Heading ("understand-this", explanation.question)], source,
      [Paragraph ([Words explanation.reading]), Paragraph ([Words explanation.result])],
      arrays.filter (legacyKeep selection) page.blocks
    ]
  }
}
fn explanation selection: ExampleSelection -> Explanation = makeExplanation selection
  (ExplanationCopy {
    question: "How do these files fit together?",
    summary: "Read the complete panels in their chosen order.",
    reading: "Read the definition, then its consumer.",
    result: "Both labels and source stay intact."
  })
  (ExplanationCopy {
    question: "このファイルはどうつながる？",
    summary: "選んだ順に完全なコードを読みます。",
    reading: "定義を読んで、呼び出しを確かめます。",
    result: "ラベルとソースはそのままです。"
  })
fn baseBlocks -> Array<Block> = [
  Paragraph ([Words (localized "Before the original examples." "元の例より前。")]),
  CodeExample ("model-日本", localized "model.ssrg — café 😀" "定義.ssrg — 日本語 😀"),
  Heading ("later", localized "Later material" "後の内容"),
  CodeExample ("unselected", localized "Kept in place" "その場に残す"),
  Terminal (localized "Terminal stays" "端末を残す", "seseragi run ."),
  CodeExample ("consumer", localized "consumer.ssrg — use" "呼び出し.ssrg — 使用"),
  Paragraph ([Words (localized "After the original examples." "元の例より後。")])
]
fn page id: String -> path: String -> kind: PageKind -> blocks: Array<Block> -> PageDefinition =
  PageDefinition { id, path, kind, title: localized id id,
    summary: localized "Original summary" "元の概要", blocks }
struct Case { name: String, selection: ExampleSelection, page: PageDefinition }
fn fixture name: String -> selection: ExampleSelection -> blocks: Array<Block> -> Case = Case {
  name, selection, page: page name ("/docs/language/" + name + "/") Article blocks
}
fn legacyFor name: String -> Maybe<LegacySelection> = match name {
  "legacy-single" -> Just (LegacyArticleExample "consumer")
  "legacy-missing" -> Just (LegacyArticleExample "absent-日本")
  "legacy-duplicate" -> Just (LegacyArticleExample "model-日本")
  "singleton" -> Just (LegacyArticleExample "model-日本")
  "legacy-without" -> Just LegacyWithoutExample
  "legacy-empty" -> Just LegacyWithoutExample
  "empty-group" -> Just LegacyWithoutExample
  _ -> Nothing
}
fn validation selection: ExampleSelection -> blocks: Array<Block> -> String = match selection {
  ArticleExamples ids -> match selectArticleExamples ids blocks {
    Left error -> error
    Right _ -> ""
  }
  _ -> ""
}
fn selected selection: ExampleSelection -> blocks: Array<Block> -> Array<String> = match selection {
  ArticleExamples ids -> match selectArticleExamples ids blocks {
    Left _ -> []
    Right chosen -> [match block { CodeExample (id, _) -> id; _ -> "not-an-example" } | block <- chosen]
  }
  _ -> []
}
struct Output deriving JsonEncode {
  name: String, locale: String, html: String, originalHtml: String,
  legacyHtml: String, validation: String, selected: Array<String>
}
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(input))} {
  Left _ -> println "[]"
  Right input -> {
    let blocks = baseBlocks ()
    let duplicate = arrays.concat [blocks, [CodeExample ("model-日本", localized "Second model label" "二つ目の定義ラベル")]]
    let unselectedDuplicate = arrays.concat [blocks, [CodeExample ("unselected", localized "Another unselected label" "未選択の別ラベル")]]
    let cases = [
      fixture "ordered" (ArticleExamples ["consumer", "model-日本"]) blocks,
      fixture "all" (ArticleExamples ["unselected", "consumer", "model-日本"]) blocks,
      fixture "singleton" (ArticleExamples ["model-日本"]) blocks,
      fixture "empty-group" (ArticleExamples []) blocks,
      fixture "missing" (ArticleExamples ["model-日本", "absent-日本"]) blocks,
      fixture "missing-empty" (ArticleExamples ["consumer"]) [],
      fixture "duplicate-request" (ArticleExamples ["model-日本", "consumer", "model-日本"]) blocks,
      fixture "duplicate-panel" (ArticleExamples ["consumer", "model-日本"]) duplicate,
      fixture "unselected-duplicate" (ArticleExamples ["consumer", "model-日本"]) unselectedDuplicate,
      fixture "legacy-single" (ArticleExample "consumer") blocks,
      fixture "legacy-without" WithoutExample blocks,
      fixture "legacy-empty" WithoutExample [],
      fixture "legacy-missing" (ArticleExample "absent-日本") blocks,
      fixture "legacy-duplicate" (ArticleExample "model-日本") duplicate
    ]
    let home = page "home" "/" Home []
    let docs = page "docs" "/docs/" DocumentationIndex []
    let firstRun = page "first-run" "/docs/first-run/" Article []
    let examples = page "examples" "/examples/" SiteLanding []
    let releases = page "releases" "/releases/" SiteLanding []
    let language = page "language" "/docs/language/" DocumentationLanding []
    let overview = page "overview" "/docs/language/overview/" Article []
    let pages = [item.page | item <- cases]
    let site = SiteCatalog {
      home, pages: arrays.concat [[home, docs, firstRun, examples, releases, language, overview], pages],
      areas: [NavigationArea {
        id: "language", title: language.title, description: language.summary, landing: language,
        groups: [NavigationGroup {
          id: "fixtures", title: localized "Fixtures" "テスト",
          sections: [NavigationSection {
            id: "explanations", title: localized "Explanations" "説明",
            kind: FlatSection, overview, pages
          }]
        }]
      }]
    }
    println (json.encodeString [Output {
      name: item.name, locale: localeCode locale,
      html: renderDocument input site locale (explainPage (explanation item.selection) item.page),
      originalHtml: renderDocument input site locale item.page,
      legacyHtml: match legacyFor item.name {
        Nothing -> ""
        Just legacy -> renderDocument input site locale (legacyExplain legacy (explanation item.selection) item.page)
      },
      validation: validation item.selection item.page.blocks,
      selected: selected item.selection item.page.blocks
    } | item <- cases, locale <- [En, Ja]])
  }
}`,
    })
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("grouped examples move complete panels in explicit ID order in both locales", () => {
  expect(rendered).toHaveLength(28)
  for (const locale of ["en", "ja"] as const) {
    const page = output("ordered", locale)
    const original = panels(page.originalHtml)
    expect(page.validation).toBe("")
    expect(page.selected).toEqual(["consumer", "model-日本"])
    expect(page.html).not.toContain("site-build-error")
    expect(panels(page.html)).toEqual([original[2], original[0], original[1]])
    expect(panels(page.html).map(code)).toEqual([
      sources[1].source,
      sources[0].source,
      sources[2].source,
    ])
    const body = article(page.html)
    const reading =
      locale === "en"
        ? "Read the definition, then its consumer."
        : "定義を読んで、呼び出しを確かめます。"
    const result =
      locale === "en"
        ? "Both labels and source stay intact."
        : "ラベルとソースはそのままです。"
    expect(body).toStartWith('<h2 id="understand-this">')
    expect(body.indexOf(original[0])).toBeLessThan(body.indexOf(reading))
    expect(body.indexOf(reading)).toBeLessThan(body.indexOf(result))
    expect(body.indexOf(result)).toBeLessThan(body.indexOf('id="later"'))
    expect(body.indexOf('id="later"')).toBeLessThan(body.indexOf(original[1]))
    expect(body).toContain(
      locale === "en" ? "model.ssrg — café 😀" : "定義.ssrg — 日本語 😀"
    )
    // Removing only the two selected panels leaves every original byte in place.
    expect(body.slice(body.indexOf(`</p>`, body.indexOf(result)) + 4)).toBe(
      article(page.originalHtml)
        .replace(original[0], "")
        .replace(original[2], "")
    )
    const all = output("all", locale)
    expect(all.selected).toEqual(["unselected", "consumer", "model-日本"])
    expect(panels(all.html)).toEqual([original[1], original[2], original[0]])
    expect(all.html).not.toContain("site-build-error")
  }
})

test("invalid groups report missing and duplicate IDs visibly without partial selection", () => {
  const errors = {
    missing: "ArticleExamples missing page CodeExample: absent-日本",
    "missing-empty": "ArticleExamples missing page CodeExample: consumer",
    "duplicate-request": "ArticleExamples duplicate requested ID: model-日本",
    "duplicate-panel": "ArticleExamples duplicate page CodeExample: model-日本",
  }
  for (const [name, error] of Object.entries(errors))
    for (const locale of ["en", "ja"] as const) {
      const page = output(name, locale)
      expect(page.validation).toBe(error)
      expect(page.selected).toEqual([])
      expect(page.html.match(/class="site-build-error"/gu)).toHaveLength(1)
      expect(text(article(page.html))).toContain(
        `Missing canonical example: __invalid-article-examples__/${name}/${error}`
      )
      // A broken group must not arbitrarily move or silently discard any panel.
      expect(panels(page.html)).toEqual(panels(page.originalHtml))
      expect(article(page.html)).toEndWith(article(page.originalHtml))
    }
})

test("group selection removes only selected IDs, preserving unrelated duplicate panels", () => {
  for (const locale of ["en", "ja"] as const) {
    const page = output("unselected-duplicate", locale)
    const original = panels(page.originalHtml)
    expect(page.validation).toBe("")
    expect(page.html).not.toContain("site-build-error")
    expect(panels(page.html)).toEqual([
      original[2],
      original[0],
      original[1],
      original[3],
    ])
  }
})

test("legacy selections remain byte-exact and singleton/empty groups are compatible", () => {
  for (const name of [
    "legacy-single",
    "legacy-without",
    "legacy-empty",
    "legacy-missing",
    "legacy-duplicate",
    "singleton",
    "empty-group",
  ])
    for (const locale of ["en", "ja"] as const) {
      const page = output(name, locale)
      expect(page.legacyHtml).not.toBe("")
      expect(page.html, `${name}: ${locale}`).toBe(page.legacyHtml)
      expect(page.html).not.toContain("site-build-error")
    }
})
