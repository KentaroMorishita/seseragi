import { expect, setDefaultTimeout, test } from "bun:test"
import assert from "node:assert/strict"
import { spawnSync } from "node:child_process"
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import { compilerReferenceModules } from "../scripts/reference"
import {
  textEditorialCases,
  textEditorialExamples,
  textTypeScriptOutput,
  textUnitCase,
} from "../scripts/text-editorial"
import { pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(120_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const examples = textEditorialExamples("https://seseragi.vercel.app/")
function plain(html: string) {
  return html
    .replace(/<[^>]*>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
}

function assertAuthoredPageTitles(
  html: string,
  titles: ReadonlyMap<string, string>,
  route: string
) {
  const article = html.match(/<article\b[^>]*>([\s\S]*?)<\/article>/u)?.[1]
  assert.ok(article, `${route}: missing article`)
  // Stop only at the explicit generated declaration/API-index boundary.
  const authored = article.split(
    /<h2 id="(?:canonical-declaration|text-api-index)"/u
  )[0]
  let checked = 0
  for (const link of authored.matchAll(
    /<a\b[^>]*\bhref="([^"]+)"[^>]*>([\s\S]*?)<\/a>/gu
  )) {
    const destination = plain(link[1])
    // Fragment links describe sections; external links include Playground actions.
    if (
      !/^\/(?:ja\/)?docs\/library\//u.test(destination) ||
      destination.includes("#")
    )
      continue
    assert.ok(
      titles.has(destination),
      `${route}: missing title for ${destination}`
    )
    assert.equal(
      plain(link[2]).trim(),
      titles.get(destination),
      `${route}: authored page-name link to ${destination} must match its heading`
    )
    checked++
  }
  return checked
}

test("fifteen published text snippets execute normal, empty, and invalid-size cases", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-text-examples-"))
  try {
    for (const sample of [...textEditorialCases, textUnitCase]) {
      const canonical = examples.find((x) => x.id === `text-${sample.slug}`)!
      const path = join(directory, `${sample.slug}.ssrg`)
      writeFileSync(path, canonical.source)
      const result = spawnSync(cli, ["run", path], {
        cwd: root,
        encoding: "utf8",
        timeout: 30_000,
      })
      expect(result.status, `${sample.name}: ${result.stderr}`).toBe(0)
      expect(result.stdout.trim(), sample.name).toBe(sample.expected)
      expect(new URL(canonical.playgroundUrl).searchParams.get("source")).toBe(
        canonical.source
      )
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("identity overlay renders fifteen real pages in both locales and preserves declarations", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-text-render-"))
  try {
    const allModules = compilerReferenceModules()
    const sourceModule = allModules.find((x) => x.specifier === "std/text")!
    const identities = new Set<string>(
      textEditorialCases.map((x) => x.identity)
    )
    const module = {
      ...sourceModule,
      items: sourceModule.items.filter(
        (x) => identities.has(x.identity) || x.identity === "std/text::isEmpty"
      ),
    }
    expect(module.items).toHaveLength(15)
    for (const item of module.items) {
      expect(item.namespace).toBe("value")
      expect(item.itemKind).toBe("function")
    }
    const input = {
      schema: 1,
      origin: "https://seseragi.example",
      playgroundUrl: "https://seseragi.vercel.app/",
      tourUrl: "https://seseragi.vercel.app/tour/",
      grammar: "",
      examples,
      referenceModules: [module],
    }
    const output = renderPageClosure<Array<{ route: string; html: string }>>({
      directory,
      modules: ["reference/catalog", "render/document"],
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
    let home = PageDefinition { id: "library", path: "/docs/library/", kind: Reference,
      title: localized "Library" "ライブラリ", summary: localized "Library" "ライブラリ", blocks: [] }
    let site = SiteCatalog { home, pages, areas: [NavigationArea {
      id: "library", title: home.title, description: home.summary, landing: home, groups
    }] }
    println (json.encodeString [Output { route: localizedRoute locale page.path,
      html: renderDocument input site locale page } | locale <- [En, Ja], page <- pages])
  }
}`,
    })
    expect(output).toHaveLength(32) // Fifteen editorial pages plus the untouched isEmpty API, each in EN/JA.
    const pages = new Map(output.map((x) => [x.route, x.html]))
    const titles = new Map(
      output.map((page) => [page.route, pageTitle(page.html)])
    )
    let authoredLinks = 0
    for (const prefix of ["", "/ja"]) {
      const moduleRoute = `${prefix}/docs/library/text/`
      const moduleHtml = pages.get(moduleRoute)!
      const moduleLinks = assertAuthoredPageTitles(
        moduleHtml,
        titles,
        moduleRoute
      )
      expect(moduleLinks).toBe(14)
      authoredLinks += moduleLinks
      const decorated = moduleHtml.replaceAll(
        ">concat</a>",
        ">concat: Join strings</a>"
      )
      expect(decorated).not.toBe(moduleHtml)
      expect(() =>
        assertAuthoredPageTitles(decorated, titles, moduleRoute)
      ).toThrow("must match its heading")
      for (const sample of textEditorialCases) {
        const route = `${prefix}/docs/library/text/function/${sample.slug}/`
        const html = pages.get(route)!
        const count = assertAuthoredPageTitles(html, titles, route)
        expect(count, route).toBe(1)
        authoredLinks += count
        expect(html).toContain(
          prefix === "/ja"
            ? ">std/text</a><span>：ほかの文字列操作を選ぶ。必要な結果から操作を探せます。</span>"
            : ">std/text</a><span>: Choose another text operation. Find the operation for the result you need.</span>"
        )
        const stale = html.replaceAll(
          ">std/text</a>",
          ">Choose another text operation</a>"
        )
        expect(stale).not.toBe(html)
        expect(() => assertAuthoredPageTitles(stale, titles, route)).toThrow(
          "must match its heading"
        )
      }
    }
    expect(authoredLinks).toBe(56)
    for (const prefix of ["", "/ja"]) {
      const moduleHtml = pages.get(`${prefix}/docs/library/text/`)!
      expect(moduleHtml).toContain('id="working-with-text"')
      expect(moduleHtml).toContain('id="public-api"')
      expect(moduleHtml).toContain('id="consume-maybe"')
      expect(moduleHtml).toContain('id="consume-either"')
      expect(plain(moduleHtml)).toContain(textUnitCase.expected)
      expect(plain(moduleHtml)).toContain(textTypeScriptOutput)
      const modulePanels = [
        ...moduleHtml.matchAll(
          /<code class="seseragi-highlight">([\s\S]*?)<\/code>/gu
        ),
      ].map((x) => plain(x[1]))
      expect(modulePanels).toEqual(
        [
          "text-units",
          "text-units-typescript",
          "text-scalarat",
          "text-slicescalars",
        ].map((id) => examples.find((x) => x.id === id)!.source)
      )
      for (const name of ["scalarAt", "sliceScalars"])
        expect(plain(moduleHtml)).toContain(
          textEditorialCases.find((x) => x.name === name)!.expected
        )

      for (const sample of textEditorialCases) {
        const route = `${prefix}/docs/library/text/function/${sample.slug}/`
        const html = pages.get(route)!
        const text = plain(html)
        const item = module.items.find((x) => x.identity === sample.identity)!
        expect(html).toContain('id="using-this-operation"')
        expect(html).toContain('id="operation-rules"')
        expect(html).toContain('id="operation-mistakes"')
        expect(text).toContain(item.signature)
        expect(text).not.toContain("A public standard-library item.")
        expect(text).not.toContain("標準ライブラリの公開項目です。")
        expect(text).toContain(sample.expected)
        const canonical = examples.find((x) => x.id === `text-${sample.slug}`)!
        const code = html.match(
          /<code class="seseragi-highlight">([\s\S]*?)<\/code>/u
        )?.[1]
        expect(plain(code ?? ""), route).toBe(canonical.source)
        expect(html).toContain(`href="${prefix}/docs/library/text/"`)
        expect(moduleHtml).toContain(`href="${route}"`)
        expect(html).not.toContain("Missing canonical example")
        if (sample.name === "trim") {
          expect(text).not.toContain("The final input remains U+FEFF")
          expect(text).not.toContain("最後の入力はU+FEFF")
          expect(text).toContain(
            prefix === "/ja" ? "U+FEFFを含む例" : "The input containing U+FEFF"
          )
        }
        if (sample.name === "concat") expect(text).toContain("std/array")

        if (
          [
            "lengthScalars",
            "lengthBytes",
            "scalarAt",
            "sliceScalars",
            "split",
            "replaceAll",
          ].includes(sample.name)
        )
          expect(html).toContain(
            `href="${prefix}/docs/library/text/#text-units"`
          )
        const ja = readFileSync(
          join(
            root,
            `apps/site/src/reference/editorial/text/${sample.slug}/ja.ssrg`
          ),
          "utf8"
        )
        const en = readFileSync(
          join(
            root,
            `apps/site/src/reference/editorial/text/${sample.slug}/en.ssrg`
          ),
          "utf8"
        )
        expect([...ja.matchAll(/^ {2}(\w+): /gm)].map((x) => x[1])).toEqual(
          [...en.matchAll(/^ {2}(\w+): /gm)].map((x) => x[1])
        )
      }
      const controlHtml = pages.get(
        `${prefix}/docs/library/text/function/isempty/`
      )!
      const controlItem = module.items.find(
        (x) => x.identity === "std/text::isEmpty"
      )!
      expect(controlHtml).not.toContain('id="using-this-operation"')
      expect(plain(controlHtml)).toContain(
        prefix === "/ja" ? controlItem.descriptionJa : controlItem.descriptionEn
      )
      expect(
        plain(pages.get(`${prefix}/docs/library/text/function/scalarat/`)!)
      ).toContain("Nothing")
      expect(
        plain(pages.get(`${prefix}/docs/library/text/function/slicescalars/`)!)
      ).toContain("Left")
    }
    if (process.env.TEXT_EDITORIAL_RENDER_DIR) {
      const { mkdirSync } = require("node:fs")
      mkdirSync(process.env.TEXT_EDITORIAL_RENDER_DIR, { recursive: true })
      for (const page of output)
        writeFileSync(
          join(
            process.env.TEXT_EDITORIAL_RENDER_DIR,
            `${page.route.replaceAll("/", "_")}.html`
          ),
          page.html
        )
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("overlay identity inventory stays valid and ignores unrelated namespace, kind, module, and symbol", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-text-identities-"))
  try {
    const all = compilerReferenceModules().find(
      (x) => x.specifier === "std/text"
    )!
    const items = textEditorialCases.map((sample) => {
      const matches = all.items.filter(
        (x) =>
          x.identity === sample.identity &&
          x.namespace === "value" &&
          x.itemKind === "function"
      )
      expect(matches, sample.identity).toHaveLength(1)
      return matches[0]
    })
    const dispatch = readFileSync(
      join(root, "apps/site/src/reference/editorial/catalog.ssrg"),
      "utf8"
    )
    const mapped = [...dispatch.matchAll(/"(std\/text::[^"]+)" -> Just/g)].map(
      (x) => x[1]
    )
    expect(new Set(mapped).size).toBe(mapped.length)
    expect(mapped.sort()).toEqual(
      textEditorialCases.map((x) => x.identity).sort()
    )
    const first = items[0]
    const probes = [
      ...items,
      { ...first, identity: "std/text::isEmpty" },
      { ...first, namespace: "type" },
      { ...first, itemKind: "constructor" },
      { ...first, module: "std/array" },
      { ...first, identity: "std/array::concat" },
    ]
    const input = {
      schema: 1,
      origin: "",
      playgroundUrl: "",
      tourUrl: "",
      grammar: "",
      examples: [],
      referenceModules: [{ ...all, items: probes }],
    }
    const result = renderPageClosure<boolean[]>({
      directory,
      modules: ["reference/editorial/catalog"],
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { ReferenceSymbol, decodeBuildInput } from "./model/build"
import { Editorial } from "./reference/editorial/model"
import { editorialFor } from "./reference/editorial/catalog"
fn hasEditorial symbol: ReferenceSymbol -> Bool = match editorialFor symbol {
  Just _ -> True
  Nothing -> False
}
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(input))} {
  Left _ -> println "[]"
  Right input -> println (json.encodeString (arrays.concat [[hasEditorial symbol | symbol <- module.items] | module <- input.referenceModules]))
}`,
    })
    expect(result).toEqual([...Array(14).fill(true), ...Array(5).fill(false)])
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("TypeScript Unicode comparison typechecks and executes the same concrete string", () => {
  const source = join(root, "apps/site/examples/src/api-text/units.ts")
  const check = spawnSync(
    join(root, "node_modules/.bin/tsc"),
    [
      "--noEmit",
      "--target",
      "ES2022",
      "--module",
      "ESNext",
      "--moduleResolution",
      "bundler",
      "--lib",
      "ES2022,ES2022.Intl,DOM",
      "--skipLibCheck",
      source,
    ],
    { cwd: root, encoding: "utf8", timeout: 30_000 }
  )
  expect(check.status, check.stderr || check.stdout).toBe(0)
  const execution = spawnSync(process.execPath, [source], {
    cwd: root,
    encoding: "utf8",
    timeout: 30_000,
  })
  expect(execution.status, execution.stderr).toBe(0)
  expect(execution.stdout.trim()).toBe(textTypeScriptOutput)
  const descriptor = examples.find((x) => x.id === "text-units-typescript")!
  expect(descriptor.source).toBe(readFileSync(source, "utf8"))
  expect(descriptor.playgroundUrl).toBe("")
})
