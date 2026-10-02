import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import {
  arrayConsumerCases,
  arrayEditorialCases,
  arrayEditorialExamples,
} from "../scripts/array-editorial"
import { compilerReferenceModules } from "../scripts/reference"
import {
  sequenceEditorialCases,
  sequenceEditorialExamples,
} from "../scripts/sequence-editorial"
import { signatureReading } from "../scripts/signature-reading"
import { assertReferenceLinkTitles, pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(120_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const examples = [
  ...arrayEditorialExamples("https://seseragi.vercel.app/"),
  ...sequenceEditorialExamples("https://seseragi.vercel.app/"),
]
function plain(html: string) {
  return html
    .replace(/<[^>]*>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
}

test("thirteen published Array snippets execute normal, empty, and invalid-size cases", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-array-examples-"))
  try {
    for (const sample of [...arrayEditorialCases, ...arrayConsumerCases]) {
      const canonical = examples.find((x) => x.id === `array-${sample.slug}`)!
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

test("identity overlay renders twelve real pages in both locales and preserves declarations", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-array-render-"))
  try {
    const allModules = compilerReferenceModules()
    const sourceModule = allModules.find((x) => x.specifier === "std/array")!
    const identities = new Set<string>(
      [
        ...arrayEditorialCases,
        ...sequenceEditorialCases.filter((x) => x.module === "array"),
      ].map((x) => x.identity)
    )
    const module = {
      ...sourceModule,
      items: sourceModule.items.filter(
        (x) => identities.has(x.identity) || x.identity === "std/array::Eq"
      ),
    }
    expect(module.items).toHaveLength(18)
    for (const item of module.items) {
      const isControl = item.identity === "std/array::Eq"
      expect(item.namespace).toBe(isControl ? "instance" : "value")
      expect(item.itemKind).toBe(isControl ? "instance" : "function")
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
    expect(output).toHaveLength(38) // Full Array overlay inventory, module and untouched Eq instance control.
    const pages = new Map(output.map((x) => [x.route, x.html]))
    const titles = new Map(output.map((x) => [x.route, pageTitle(x.html)]))
    for (const prefix of ["", "/ja"]) {
      const moduleHtml = pages.get(`${prefix}/docs/library/array/`)!
      expect(moduleHtml).toContain('id="working-with-arrays"')
      expect(
        assertReferenceLinkTitles(
          `${moduleHtml.split('id="array-api-index"')[0]}</article>`,
          titles,
          `${prefix}/docs/library/array/`
        )
      ).toBe(17)
      expect(moduleHtml).toContain('id="public-api"')
      expect(moduleHtml).toContain('id="consume-maybe"')
      expect(moduleHtml).toContain('id="consume-either"')
      for (const consumer of arrayConsumerCases)
        expect(plain(moduleHtml)).toContain(consumer.expected)
      for (const sample of arrayEditorialCases) {
        const route = `${prefix}/docs/library/array/function/${sample.slug}/`
        const html = pages.get(route)!
        const text = plain(html)
        expect(
          assertReferenceLinkTitles(
            `${html.split('id="canonical-declaration"')[0]}</article>`,
            titles,
            route
          )
        ).toBe(1)
        const item = module.items.find((x) => x.identity === sample.identity)!
        expect(html).toContain('id="using-this-operation"')
        expect(html).toContain('id="operation-rules"')
        expect(html).toContain('id="operation-mistakes"')
        expect(text).toContain(item.signature)
        expect(text).not.toContain("A public standard-library item.")
        expect(text).not.toContain("標準ライブラリの公開項目です。")
        expect(text).toContain(sample.expected)
        const canonical = examples.find((x) => x.id === `array-${sample.slug}`)!
        const code = html.match(
          /<code class="seseragi-highlight">([\s\S]*?)<\/code>/u
        )?.[1]
        expect(plain(code ?? ""), route).toBe(canonical.source)
        expect(html).toContain(`href="${prefix}/docs/library/array/"`)
        expect(moduleHtml).toContain(`href="${route}"`)
        expect(html).not.toContain("Missing canonical example")
        if (["chunksOf", "windows"].includes(sample.name))
          expect(html).toContain(
            `href="${prefix}/docs/library/array/#consume-either"`
          )
        if (["last", "init", "groupBy"].includes(sample.name))
          expect(html).toContain(
            `href="${prefix}/docs/library/array/#consume-maybe"`
          )
        if (sample.name === "sortBy") {
          expect(text).toContain("nobody")
          expect(text).not.toContain("<Int> specifies")
          expect(text).not.toContain("空配列を作る<Int>")
        }
        const ja = readFileSync(
          join(
            root,
            `apps/site/src/reference/editorial/array/${sample.slug}/ja.ssrg`
          ),
          "utf8"
        )
        const en = readFileSync(
          join(
            root,
            `apps/site/src/reference/editorial/array/${sample.slug}/en.ssrg`
          ),
          "utf8"
        )
        expect([...ja.matchAll(/^ {2}(\w+): /gm)].map((x) => x[1])).toEqual(
          [...en.matchAll(/^ {2}(\w+): /gm)].map((x) => x[1])
        )
      }
      const controlHtml = pages.get(
        `${prefix}/docs/library/array/instance/eq-array-a/`
      )!
      const controlItem = module.items.find(
        (x) => x.identity === "std/array::Eq"
      )!
      expect(controlHtml).toBeDefined()
      expect(controlItem).toBeDefined()
      expect(controlItem.namespace).toBe("instance")
      expect(controlItem.itemKind).toBe("instance")
      expect(controlHtml).not.toContain('id="using-this-operation"')
      const reading =
        prefix === "/ja" ? controlItem.reading.ja : controlItem.reading.en
      expect(reading.trim()).not.toBe("")
      expect(plain(controlHtml)).toContain(reading)
      expect(plain(controlHtml)).toContain(controlItem.signature)
      expect(
        plain(pages.get(`${prefix}/docs/library/array/function/findindex/`)!)
      ).toContain("Nothing")
      expect(
        plain(pages.get(`${prefix}/docs/library/array/function/chunksof/`)!)
      ).toContain("Left")
    }
    if (process.env.ARRAY_EDITORIAL_RENDER_DIR) {
      const { mkdirSync } = require("node:fs")
      mkdirSync(process.env.ARRAY_EDITORIAL_RENDER_DIR, { recursive: true })
      for (const page of output)
        writeFileSync(
          join(
            process.env.ARRAY_EDITORIAL_RENDER_DIR,
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
  const directory = mkdtempSync(join(tmpdir(), "seseragi-array-identities-"))
  try {
    const all = compilerReferenceModules().find(
      (x) => x.specifier === "std/array"
    )!
    const items = arrayEditorialCases.map((sample) => {
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
    const mapped = [...dispatch.matchAll(/"(std\/array::[^"]+)" -> Just/g)].map(
      (x) => x[1]
    )
    expect(new Set(mapped).size).toBe(mapped.length)
    expect(mapped.sort()).toEqual(
      [
        ...arrayEditorialCases.map((x) => x.identity),
        ...sequenceEditorialCases
          .filter((x) => x.module === "array")
          .map((x) => x.identity),
      ].sort()
    )
    const first = items[0]
    const probes = [
      ...items,
      { ...first, identity: "std/array::Eq" },
      { ...first, namespace: "type" },
      { ...first, itemKind: "constructor" },
      { ...first, module: "std/list" },
      { ...first, identity: "std/list::chunksOf" },
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
    expect(result).toEqual([...Array(11).fill(true), ...Array(5).fill(false)])
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("signature walkthrough uses singular and plural type-parameter grammar", () => {
  expect(
    signatureReading(
      "last<A> values: Array<A> -> Maybe<A>",
      "function",
      ["A"],
      []
    ).en
  ).toContain("A names a type parameter.")
  expect(
    signatureReading(
      "map<A, B> values: Array<A> -> Array<B>",
      "function",
      ["A", "B"],
      []
    ).en
  ).toContain("A, B name type parameters.")
})
