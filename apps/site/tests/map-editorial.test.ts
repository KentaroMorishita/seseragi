import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import {
  apiCorrectionCases,
  apiCorrectionExamples,
} from "../scripts/api-corrections"
import {
  mapEditorialCases,
  mapEditorialExamples,
} from "../scripts/map-editorial"
import { compilerReferenceModules } from "../scripts/reference"
import { assertReferenceLinkTitles, pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(240_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const examples = [
  ...mapEditorialExamples("https://seseragi.vercel.app/"),
  ...apiCorrectionExamples("https://seseragi.vercel.app/"),
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

test("fourteen published Map pairs execute normal, empty, duplicate and missing-key cases", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-map-examples-"))
  try {
    for (const sample of mapEditorialCases) {
      const canonical = examples.find((x) => x.id === `map-${sample.slug}`)!
      const path = join(directory, `${sample.slug}.ssrg`)
      writeFileSync(path, canonical.source)
      const result = spawnSync(cli, ["run", path], {
        cwd: root,
        encoding: "utf8",
        timeout: 15_000,
      })
      expect(result.status, `${sample.name}: ${result.stderr}`).toBe(0)
      expect(result.stdout.trim(), sample.name).toBe(sample.expected)
      const comparison = examples.find((x) => x.id === `map-${sample.slug}-ts`)!
      const tsPath = join(directory, `${sample.slug}.ts`)
      writeFileSync(tsPath, comparison.source)
      const ts = spawnSync(process.execPath, [tsPath], {
        cwd: directory,
        encoding: "utf8",
        timeout: 15_000,
      })
      expect(ts.status, ts.stderr).toBe(0)
      expect(ts.stdout).toBe(result.stdout)
      expect(comparison.playgroundUrl).toBe("")
      expect(new URL(canonical.playgroundUrl).searchParams.get("source")).toBe(
        canonical.source
      )
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("identity overlay renders fifteen Map pages in both locales and preserves declarations", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-map-render-"))
  try {
    const allModules = compilerReferenceModules()
    const sourceModule = allModules.find((x) => x.specifier === "std/map")!
    const identities = new Set<string>(mapEditorialCases.map((x) => x.identity))
    const module = {
      ...sourceModule,
      items: sourceModule.items.filter(
        (x) =>
          identities.has(x.identity) ||
          ["std/map::filter", "std/map::isEmpty", "std/map::get"].includes(
            x.identity
          )
      ),
    }
    expect(module.items).toHaveLength(17)
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
    expect(output).toHaveLength(36) // Fifteen edited identities and three controls, each in EN/JA.
    const pages = new Map(output.map((x) => [x.route, x.html]))
    for (const prefix of ["", "/ja"]) {
      const moduleHtml = pages.get(`${prefix}/docs/library/map/`)!
      expect(moduleHtml).toContain('id="working-with-maps"')
      const titles = new Map(
        output.map((page) => [page.route, pageTitle(page.html)])
      )
      expect(
        assertReferenceLinkTitles(
          `${moduleHtml.split('id="map-api-index"')[0]}</article>`,
          titles,
          `${prefix}/docs/library/map/`
        )
      ).toBe(15)
      expect(moduleHtml).toContain('id="public-api"')
      const getCase = apiCorrectionCases.find(
        (x) => x.identity === "std/map::get"
      )!
      expect(
        plain(pages.get(`${prefix}/docs/library/map/function/get/`)!)
      ).toContain(getCase.expected)
      for (const sample of mapEditorialCases) {
        const route = `${prefix}/docs/library/map/function/${sample.slug}/`
        const html = pages.get(route)!
        const text = plain(html)
        // Check the complete authored leaf body, not only the module selector.
        // Generated declarations/index labels follow this explicit boundary.
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
        expect(html).toContain('id="typescript-comparison"')
        expect(text).toContain(item.signature)
        expect(text).not.toContain("A public standard-library item.")
        expect(text).not.toContain("標準ライブラリの公開項目です。")
        expect(text).toContain(sample.expected)
        const canonical = examples.find((x) => x.id === `map-${sample.slug}`)!
        const code = html.match(
          /<code class="seseragi-highlight">([\s\S]*?)<\/code>/u
        )?.[1]
        expect(plain(code ?? ""), route).toBe(canonical.source)
        expect(html).toContain(`href="${prefix}/docs/library/map/"`)
        expect(moduleHtml).toContain(`href="${route}"`)
        expect(html).not.toContain("Missing canonical example")
        const comparison = examples.find(
          (x) => x.id === `map-${sample.slug}-ts`
        )!
        const panels = [
          ...html.matchAll(
            /<code class="seseragi-highlight">([\s\S]*?)<\/code>/gu
          ),
        ]
        expect(plain(panels[1]?.[1] ?? ""), route).toBe(comparison.source)
        const ja = readFileSync(
          join(
            root,
            `apps/site/src/reference/editorial/map/${sample.slug}/ja.ssrg`
          ),
          "utf8"
        )
        const en = readFileSync(
          join(
            root,
            `apps/site/src/reference/editorial/map/${sample.slug}/en.ssrg`
          ),
          "utf8"
        )
        expect([...ja.matchAll(/^ {2}(\w+): /gm)].map((x) => x[1])).toEqual(
          [...en.matchAll(/^ {2}(\w+): /gm)].map((x) => x[1])
        )
      }
      for (const name of ["filter", "isEmpty"]) {
        const control = module.items.find(
          (x) => x.identity === `std/map::${name}`
        )!
        const html = pages.get(
          `${prefix}/docs/library/map/function/${name.toLowerCase()}/`
        )!
        expect(html).not.toContain('id="using-this-operation"')
        expect(plain(html)).toContain(
          prefix ? control.descriptionJa : control.descriptionEn
        )
      }
    }
    if (process.env.MAP_EDITORIAL_RENDER_DIR) {
      const { mkdirSync } = require("node:fs")
      mkdirSync(process.env.MAP_EDITORIAL_RENDER_DIR, { recursive: true })
      for (const page of output) {
        const destination = join(
          process.env.MAP_EDITORIAL_RENDER_DIR,
          page.route
        )
        mkdirSync(destination, { recursive: true })
        writeFileSync(join(destination, "index.html"), page.html)
      }
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("overlay identity inventory stays valid and ignores unrelated namespace, kind, module, and symbol", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-map-identities-"))
  try {
    const all = compilerReferenceModules().find(
      (x) => x.specifier === "std/map"
    )!
    const items = mapEditorialCases.map((sample) => {
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
      join(root, "apps/site/src/reference/editorial/map-catalog.ssrg"),
      "utf8"
    )
    const mapped = [...dispatch.matchAll(/"(std\/map::[^"]+)" -> Just/g)].map(
      (x) => x[1]
    )
    expect(new Set(mapped).size).toBe(mapped.length)
    expect(mapped.sort()).toEqual(
      mapEditorialCases.map((x) => x.identity).sort()
    )
    const first = items[0]
    const probes = [
      ...items,
      { ...first, identity: "std/map::filter" },
      { ...first, namespace: "type" },
      { ...first, itemKind: "constructor" },
      { ...first, module: "std/array" },
      { ...first, identity: "std/array::chunksOf" },
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
      modules: ["reference/editorial/map-catalog"],
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { ReferenceSymbol, decodeBuildInput } from "./model/build"
import { Editorial } from "./reference/editorial/model"
import { mapEditorialFor } from "./reference/editorial/map-catalog"
fn hasEditorial symbol: ReferenceSymbol -> Bool = match mapEditorialFor symbol {
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

test("existing Map runtime contracts cover callback counts, collisions and preserved versions", () => {
  const result = spawnSync(
    process.execPath,
    ["test", "runtime/ts/tests/map.test.ts"],
    { cwd: root, encoding: "utf8", timeout: 15000 }
  )
  expect(result.status, result.stderr).toBe(0)
  expect(result.stderr).toContain("0 fail")
})
