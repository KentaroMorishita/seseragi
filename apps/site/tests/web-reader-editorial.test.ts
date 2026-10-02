import { expect, setDefaultTimeout, test } from "bun:test"
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import { compilerReferenceModules } from "../scripts/reference"
import {
  webReaderCases,
  webReaderExamples,
  webReaderRoutes,
} from "../scripts/web-reader"
import { assertAuthoredLibraryTitles } from "./library-titles"
import { pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(240_000)
const root = resolve(import.meta.dir, "../../..")
const examples = webReaderExamples("https://seseragi.vercel.app/")
function plain(html: string) {
  return html
    .replace(/<[^>]*>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
}
function buildInput(
  referenceModules: ReturnType<typeof compilerReferenceModules>
) {
  return {
    schema: 1,
    origin: "https://seseragi.example",
    playgroundUrl: "https://seseragi.vercel.app/",
    tourUrl: "",
    grammar: "",
    examples,
    referenceModules,
  }
}

test("Web editorial guards exact identity, module, namespace and kind, leaving unselected symbols untouched", () => {
  const all = compilerReferenceModules()
  const leaves = webReaderRoutes.filter((x) => x.kind !== "module")
  const symbols = leaves.map((item) => {
    const matches = all
      .find((m) => m.specifier === item.module)!
      .items.filter(
        (x) =>
          x.identity === item.identity &&
          x.namespace === item.namespace &&
          x.itemKind === item.kind
      )
    expect(matches).toHaveLength(1)
    return matches[0]
  })
  expect(symbols).toHaveLength(13)
  const probes = symbols.flatMap((symbol) => [
    symbol,
    { ...symbol, module: "std/web/svg" },
    { ...symbol, namespace: "wrong" },
    { ...symbol, itemKind: "wrong" },
    { ...symbol, identity: `${symbol.identity}-wrong` },
  ])
  const controls = all
    .filter((m) =>
      ["std/web/html", "std/web/dom", "std/signal"].includes(m.specifier)
    )
    .flatMap((m) =>
      m.items.filter((s) => !symbols.some((x) => x.identity === s.identity))
    )
  const input = buildInput([{ ...all[0], items: [...probes, ...controls] }])
  const directory = mkdtempSync(join(tmpdir(), "seseragi-web-guards-"))
  try {
    const result = renderPageClosure<Array<[boolean, boolean]>>({
      directory,
      modules: [
        "reference/editorial/catalog",
        "reference/editorial/web-reader/catalog",
      ],
      timeoutMs: 180_000,
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { Editorial } from "./reference/editorial/model"
import { editorialFor, moduleEditorial } from "./reference/editorial/catalog"
import { webReaderEditorialFor, webReaderModuleBlocks } from "./reference/editorial/web-reader/catalog"
fn present value: Maybe<Editorial> -> Bool = match value { Just _ -> True; Nothing -> False }
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(input))} {
  Left _ -> println "[]"
  Right input -> println (json.encodeString (arrays.concat [
    arrays.concat [[(present (webReaderEditorialFor symbol), present (editorialFor symbol)) | symbol <- module.items] | module <- input.referenceModules],
    [(arrays.length (webReaderModuleBlocks name) > 0, arrays.length (moduleEditorial name) > 0) | name <- ["std/web/html", "std/web/dom", "std/web/svg", "std/web/html-extra", "std/web/dom::app"]]
  ]))
}`,
    })
    expect(result).toHaveLength(probes.length + controls.length + 5)
    for (let i = 0; i < probes.length; i++)
      expect(result[i]).toEqual(i % 5 === 0 ? [true, true] : [false, false])
    for (let i = probes.length; i < probes.length + controls.length; i++)
      expect(result[i]).toEqual([false, false])
    expect(result.slice(-5)).toEqual([
      [true, true],
      [true, true],
      [false, false],
      [false, false],
      [false, false],
    ])
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("all thirty Web locale bodies retain exact prose, examples, output, declarations and purpose-labelled destination titles", () => {
  const all = compilerReferenceModules()
  const identities = new Set<string>(webReaderRoutes.map((x) => x.identity))
  const needed = new Set([
    "std/web/html::html",
    "std/web/html::head",
    "std/web/html::body",
    "std/web/html::div",
    "std/web/dom::query",
    "std/web/dom::run",
    "std/web/dom::mount",
  ])
  const modules = all
    .filter((m) =>
      ["std/web/html", "std/web/dom", "std/signal"].includes(m.specifier)
    )
    .map((m) => ({
      ...m,
      items: m.items.filter(
        (s) => identities.has(s.identity) || needed.has(s.identity)
      ),
    }))
  const input = buildInput(modules)
  const directory = mkdtempSync(join(tmpdir(), "seseragi-web-render-"))
  try {
    const result = renderPageClosure<Array<{ route: string; html: string }>>({
      directory,
      modules: ["reference/catalog", "render/document"],
      timeoutMs: 180_000,
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { En, Ja, localized, localizedRoute } from "./model/locale"
import { Reference, SiteCatalog, PageDefinition, NavigationArea } from "./model/page"
import { referenceGroups } from "./reference/catalog"
import { renderDocument } from "./render/document"
struct Output deriving JsonEncode { route: String, html: String }
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(input))} {
  Left _ -> println "[]"
  Right input -> {
    let groups = referenceGroups input.referenceModules
    let sections = arrays.concat [group.sections | group <- groups]
    let pages = arrays.concat [arrays.concat [[section.overview], section.pages] | section <- sections]
    let home = PageDefinition { id: "library", path: "/docs/library/", kind: Reference, title: localized "Library" "ライブラリ", summary: localized "Library" "ライブラリ", blocks: [] }
    let site = SiteCatalog { home, pages, areas: [NavigationArea { id: "library", title: home.title, description: home.summary, landing: home, groups }] }
    println (json.encodeString [Output {route: localizedRoute locale page.path, html: renderDocument input site locale page} | locale <- [En, Ja], page <- pages])
  }
}`,
    })
    const outputDir = process.env.WEB_READER_RENDER_DIR
    if (outputDir) {
      mkdirSync(outputDir, { recursive: true })
      writeFileSync(
        join(outputDir, "all-rendered.json"),
        JSON.stringify(result, null, 2)
      )
      for (const page of result) {
        const destination = join(outputDir, page.route.slice(1))
        mkdirSync(destination, { recursive: true })
        writeFileSync(join(destination, "index.html"), page.html)
      }
    }
    const pages = new Map(result.map((x) => [x.route, x.html]))
    const titles = new Map(result.map((x) => [x.route, pageTitle(x.html)]))
    let titleLinks = 0
    for (const item of webReaderRoutes)
      for (const locale of ["en", "ja"]) {
        const prefix = locale === "en" ? "" : "/ja",
          route = prefix + item.route
        const html = pages.get(route)!,
          body = html.match(/<article\b[\s\S]*?<\/article>/u)![0],
          text = plain(body)
        expect(html, route).toBeDefined()
        for (const anchor of [
          "using-this-operation",
          "typescript-comparison",
          "operation-rules",
          "choose-next",
        ])
          expect(body).toContain(`id="${anchor}"`)
        expect(body).not.toContain("Missing canonical example")
        expect(body).not.toContain("/docs/applications/web/")
        if (item.example !== "release-card-app") {
          expect(text).toContain(
            locale === "en"
              ? "Choose Open in Playground, then Run."
              : "「Playgroundで開く」からRunを選びます。"
          )
          expect(text).toContain(
            locale === "en" ? "In Output, select Text" : "OutputのText"
          )
        }
        if (["html-module", "p", "fragment"].includes(item.slug)) {
          expect(text).toContain(
            'let label: html.Html<Unit> = html.text "Release"'
          )
          expect(text).toContain("0.61.19")
        }
        if (item.slug === "elementprops") {
          expect(text).toContain("hidden?: Bool")
          expect(text).toContain("children: C")
        }
        if (item.slug === "intochildren")
          expect(text).toContain(
            locale === "en"
              ? "In an element function’s declaration"
              : "要素を作る関数の宣言で"
          )
        if (item.slug === "h2")
          expect(text).toContain(
            locale === "en"
              ? "Pass the result of heading ()"
              : "heading ()の戻り値を"
          )

        const copy = (lang: string) =>
          readFileSync(
            join(
              root,
              `apps/site/src/reference/editorial/web-reader/${item.slug}/${lang}.ssrg`
            ),
            "utf8"
          )
        const strings = (source: string) =>
          [...source.matchAll(/^ {4}("(?:[^"\\]|\\.)*")/gmu)].map(
            (m) => JSON.parse(m[1]) as string
          )
        const own = copy(locale),
          other = copy(locale === "en" ? "ja" : "en")
        expect(strings(own)).toHaveLength(strings(other).length)
        for (const paragraph of strings(own))
          expect(text, `${route}: paragraph`).toContain(paragraph)
        for (const paragraph of strings(other))
          expect(text, `${route}: locale leakage`).not.toContain(paragraph)
        for (const [, field, value] of own.matchAll(
          /^ {2}(\w+): ("(?:[^"\\]|\\.)*")/gmu
        ))
          if (field !== "summary" || item.kind !== "module")
            expect(plain(html), `${route}: ${field}`).toContain(
              JSON.parse(value)
            )
        const continuation =
          item.slug === "html-module"
            ? "cards"
            : ["app", "dom-module"].includes(item.slug)
              ? "release-card-view"
              : ""
        const comparison =
          item.slug === "html-module"
            ? "cards-ts"
            : ["app", "dom-module"].includes(item.slug)
              ? "release-card-app-ts"
              : ""
        const expectedSourceIds = [
          item.example,
          continuation,
          comparison,
        ].filter(Boolean)
        const panels = [
          ...body.matchAll(
            /<code class="seseragi-highlight">([\s\S]*?)<\/code>/gu
          ),
        ].map((m) => plain(m[1]))
        for (const id of expectedSourceIds)
          expect(
            panels.filter(
              (x) =>
                x === examples.find((e) => e.id === `web-reader-${id}`)!.source
            ),
            `${route}: source ${id}`
          ).toHaveLength(1)
        const expectedOutputs = [
          item.example,
          continuation,
          item.slug === "html-module" ? "cards" : "",
        ]
          .filter(Boolean)
          .map((slug) => webReaderCases.find((x) => x.slug === slug)!)
          .filter((x) => "output" in x)
          .map((x) => ("output" in x ? x.output.trimEnd() : ""))
        const terminals = [
          ...body.matchAll(
            /<section class="code-panel terminal-panel">[\s\S]*?<pre><code>([\s\S]*?)<\/code><\/pre><\/section>/gu
          ),
        ].map((m) => plain(m[1]))
        expect(terminals, `${route}: outputs`).toEqual(
          item.example === "release-card-app"
            ? [
                "seseragi build release-card-app.ssrg --target web --out-dir release-card-app-web",
                ...expectedOutputs,
              ]
            : expectedOutputs
        )
        const seeds = [...body.matchAll(/href="([^"]+)"/gu)]
          .map((m) => new URL(plain(m[1]), input.origin))
          .filter(
            (u) =>
              u.origin === "https://seseragi.vercel.app" &&
              u.searchParams.has("source")
          )
          .map((u) => u.searchParams.get("source"))
        expect(seeds).toEqual(
          [item.example, continuation]
            .filter(Boolean)
            .map(
              (id) => examples.find((e) => e.id === `web-reader-${id}`)!.source
            )
        )
        if (item.kind !== "module") {
          const symbol = modules
            .find((m) => m.specifier === item.module)!
            .items.find((s) => s.identity === item.identity)!
          expect(text).toContain(symbol.signature)
          expect(text).toContain(
            locale === "en" ? symbol.reading.en : symbol.reading.ja
          )
          expect(body).toContain('id="canonical-declaration"')
          const topicPrefix = `${prefix}/docs/library/${item.module.slice(4)}/`
          const sequence = [
            ...html.matchAll(
              /class="reference-(?:previous|next)" href="([^"]+)"/gu
            ),
          ]
          for (const [, destination] of sequence) {
            expect(destination, `${route}: same-topic sequence`).toStartWith(
              topicPrefix
            )
            expect(
              pages.has(destination),
              `${route}: rendered sequence destination`
            ).toBe(true)
          }
          expect(html).toContain(
            `class="reference-topic-link" href="${topicPrefix}"`
          )
          expect(body).toContain(
            `href="${prefix}/docs/library/${item.module.slice(4)}/"`
          )
          expect(
            pages.get(`${prefix}/docs/library/${item.module.slice(4)}/`)
          ).toContain(`href="${route}"`)
        }
        const entry = {
          route,
          kind: item.kind === "module" ? ("module" as const) : ("api" as const),
        }
        const count = assertAuthoredLibraryTitles(html, titles, entry)
        expect(count).toBeGreaterThanOrEqual(2)
        titleLinks += count
        const mutated = html.replace(
          /(<p><a\b[^>]*href="[^"#]+">)([^<]+)(<\/a><span>：|<\/a><span>: )/u,
          "$1wrong title$3"
        )
        expect(mutated).not.toBe(html)
        expect(() =>
          assertAuthoredLibraryTitles(mutated, titles, entry)
        ).toThrow()
        expect(html).toContain(`href="${prefix ? "" : "/ja"}${item.route}"`)
      }
    expect(titleLinks).toBe(116)
    for (const route of [
      "/docs/library/web/html/function/div/",
      "/docs/library/web/dom/function/query/",
    ]) {
      expect(pages.get(route)).not.toContain('id="typescript-comparison"')
    }
    if (outputDir)
      writeFileSync(
        join(outputDir, "selected-bodies.json"),
        JSON.stringify(
          webReaderRoutes.flatMap((item) =>
            ["", "/ja"].map((prefix) => ({
              route: prefix + item.route,
              body: pages
                .get(prefix + item.route)!
                .match(/<article\b[\s\S]*?<\/article>/u)![0],
            }))
          ),
          null,
          2
        )
      )
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
