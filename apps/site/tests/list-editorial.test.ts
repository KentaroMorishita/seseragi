import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import {
  listConsumerCases,
  listEditorialCases,
  listEditorialExamples,
} from "../scripts/list-editorial"
import { compilerReferenceModules } from "../scripts/reference"
import {
  sequenceEditorialCases,
  sequenceEditorialExamples,
} from "../scripts/sequence-editorial"
import { assertReferenceLinkTitles, pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(240_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const examples = [
  ...listEditorialExamples("https://seseragi.vercel.app/"),
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

test("thirteen published List snippets execute normal, empty, and invalid-size cases", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-list-examples-"))
  try {
    for (const sample of [...listEditorialCases, ...listConsumerCases]) {
      const canonical = examples.find((x) => x.id === `list-${sample.slug}`)!
      const path = join(directory, `${sample.slug}.ssrg`)
      writeFileSync(path, canonical.source)
      const result = spawnSync(cli, ["run", path], {
        cwd: root,
        encoding: "utf8",
        timeout: 15_000,
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

test("identity overlay renders twelve List pages in both locales and preserves declarations", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-list-render-"))
  try {
    const allModules = compilerReferenceModules()
    const sourceModule = allModules.find((x) => x.specifier === "std/list")!
    const identities = new Set<string>(
      listEditorialCases.map((x) => x.identity)
    )
    const module = {
      ...sourceModule,
      items: sourceModule.items.filter(
        (x) =>
          identities.has(x.identity) ||
          ["std/list::Eq", "std/list::empty", "std/list::zip"].includes(
            x.identity
          )
      ),
    }
    expect(module.items).toHaveLength(14)
    for (const item of module.items) {
      const isControl = item.identity === "std/list::Eq"
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
    expect(output).toHaveLength(30) // Twelve edited identities and three controls, each in EN/JA.
    const pages = new Map(output.map((x) => [x.route, x.html]))
    for (const prefix of ["", "/ja"]) {
      const moduleHtml = pages.get(`${prefix}/docs/library/list/`)!
      expect(moduleHtml).toContain('id="working-with-lists"')
      const titles = new Map(
        output.map((page) => [page.route, pageTitle(page.html)])
      )
      expect(
        assertReferenceLinkTitles(
          `${moduleHtml.split('id="list-api-index"')[0]}</article>`,
          titles,
          `${prefix}/docs/library/list/`
        )
      ).toBeGreaterThanOrEqual(13)
      for (const name of ["empty", "zip"]) {
        const control = sequenceEditorialCases.find(
          (x) => x.module === "list" && x.name === name
        )!
        const controlHtml = pages.get(
          `${prefix}/docs/library/list/function/${name}/`
        )!
        expect(plain(controlHtml)).toContain(control.expected)
        expect(controlHtml).toContain('id="using-this-operation"')
      }
      expect(moduleHtml).toContain('id="public-api"')
      expect(moduleHtml).toContain('id="consume-maybe"')
      expect(moduleHtml).toContain('id="consume-either"')
      for (const consumer of listConsumerCases)
        expect(plain(moduleHtml)).toContain(consumer.expected)
      for (const sample of listEditorialCases) {
        const route = `${prefix}/docs/library/list/function/${sample.slug}/`
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
        const canonical = examples.find((x) => x.id === `list-${sample.slug}`)!
        const code = html.match(
          /<code class="seseragi-highlight">([\s\S]*?)<\/code>/u
        )?.[1]
        expect(plain(code ?? ""), route).toBe(canonical.source)
        expect(html).toContain(`href="${prefix}/docs/library/list/"`)
        expect(moduleHtml).toContain(`href="${route}"`)
        expect(html).not.toContain("Missing canonical example")
        if (["chunksOf", "windows"].includes(sample.name))
          expect(html).toContain(
            `href="${prefix}/docs/library/list/#consume-either"`
          )
        if (["last", "init", "groupBy"].includes(sample.name))
          expect(html).toContain(
            `href="${prefix}/docs/library/list/#consume-maybe"`
          )
        if (sample.name === "sortBy") {
          expect(text).toContain("nobody")
          expect(text).toContain("nobody")
          expect(text).toContain("score")
        }
        const ja = readFileSync(
          join(
            root,
            `apps/site/src/reference/editorial/list/${sample.slug}/ja.ssrg`
          ),
          "utf8"
        )
        const en = readFileSync(
          join(
            root,
            `apps/site/src/reference/editorial/list/${sample.slug}/en.ssrg`
          ),
          "utf8"
        )
        expect([...ja.matchAll(/^ {2}(\w+): /gm)].map((x) => x[1])).toEqual(
          [...en.matchAll(/^ {2}(\w+): /gm)].map((x) => x[1])
        )
      }
      const controlHtml = pages.get(
        `${prefix}/docs/library/list/instance/eq-list-a/`
      )!
      const controlItem = module.items.find(
        (x) => x.identity === "std/list::Eq"
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
        plain(pages.get(`${prefix}/docs/library/list/function/findindex/`)!)
      ).toContain("Nothing")
      expect(
        plain(pages.get(`${prefix}/docs/library/list/function/chunksof/`)!)
      ).toContain("Left")
    }
    if (process.env.LIST_EDITORIAL_RENDER_DIR) {
      const { mkdirSync } = require("node:fs")
      mkdirSync(process.env.LIST_EDITORIAL_RENDER_DIR, { recursive: true })
      for (const page of output) {
        const destination = join(
          process.env.LIST_EDITORIAL_RENDER_DIR,
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
  const directory = mkdtempSync(join(tmpdir(), "seseragi-list-identities-"))
  try {
    const all = compilerReferenceModules().find(
      (x) => x.specifier === "std/list"
    )!
    const items = listEditorialCases.map((sample) => {
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
    const mapped = [...dispatch.matchAll(/"(std\/list::[^"]+)" -> Just/g)].map(
      (x) => x[1]
    )
    expect(new Set(mapped).size).toBe(mapped.length)
    expect(mapped.sort()).toEqual(
      [
        ...listEditorialCases.map((x) => x.identity),
        ...sequenceEditorialCases
          .filter((x) => x.module === "list")
          .map((x) => x.identity),
      ].sort()
    )
    const first = items[0]
    const probes = [
      ...items,
      { ...first, identity: "std/list::Eq" },
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

test("List callbacks short-circuit, run in documented order and preserve stable ties", async () => {
  const lists = await import("../../../runtime/ts/src/list")
  const maps = await import("../../../runtime/ts/src/map")
  const sums = await import("../../../runtime/ts/src/sum")
  const values = lists.fromArray([1, 2, 0, 3])
  for (const operation of [lists.takeWhile, lists.dropWhile]) {
    const calls: number[] = []
    operation((n: number) => {
      calls.push(n)
      return n > 0
    }, values)
    expect(calls).toEqual([1, 2, 0])
  }
  const found: number[] = []
  expect(
    lists.findIndex((n: number) => {
      found.push(n)
      return n === 2
    }, values)
  ).toEqual(sums.Just(1))
  expect(found).toEqual([1, 2])
  const steps: number[] = []
  expect(
    lists.reduceRight(
      0,
      (n: number) => (acc: number) => {
        steps.push(n)
        return n - acc
      },
      lists.fromArray([1, 2, 3])
    )
  ).toBe(2)
  expect(steps).toEqual([3, 2, 1])
  expect(
    lists.reduceRight(
      10,
      (_: number) => (_acc: number) => {
        throw new Error("empty callback")
      },
      lists.empty()
    )
  ).toBe(10)
  const ord = {
    compare: (a: number) => (b: number) =>
      a < b ? sums.Less : a > b ? sums.Greater : sums.Equal,
  }
  const people = [
    { name: "Aki", score: 2 },
    { name: "Mio", score: 1 },
    { name: "Ren", score: 2 },
  ]
  const keyCalls: string[] = []
  expect(
    lists
      .toArray(
        lists.sortBy(
          ord,
          (p: (typeof people)[number]) => {
            keyCalls.push(p.name)
            return p.score
          },
          lists.fromArray(people)
        )
      )
      .map((p) => p.name)
  ).toEqual(["Mio", "Aki", "Ren"])
  expect(keyCalls).toEqual(["Aki", "Mio", "Ren"])
  expect(
    lists.toArray(
      lists.sortBy(
        ord,
        (_: number) => {
          throw new Error("empty key")
        },
        lists.empty()
      )
    )
  ).toEqual([])
  const calls: number[] = []
  const eq = { eq: (a: number) => (b: number) => a === b }
  const hash = { hash: (a: number) => a }
  const groups = lists.groupBy(
    eq,
    hash,
    (n: number) => {
      calls.push(n)
      return n % 2
    },
    lists.fromArray([3, 2, 5, 4, 7])
  )
  expect(calls).toEqual([3, 2, 5, 4, 7])
  expect(maps.keys(groups)).toEqual([1, 0])
  const odds = maps.get(eq, hash, 1, groups)
  expect(odds.tag).toBe("Just")
  if (odds.tag === "Just") expect(lists.toArray(odds.value)).toEqual([3, 5, 7])
})

test("ordinary TypeScript counterparts use the documented inputs, order and empty behavior", () => {
  const values = [1, 2, 0, 3]
  let boundary = 0
  while (boundary < values.length && values[boundary] > 0) boundary++
  expect(values.slice(0, boundary)).toEqual([1, 2])
  expect(values.slice(boundary)).toEqual([0, 3])
  expect([30, 40].findIndex((x) => x >= 30)).toBe(0)
  expect([10, 20].findIndex((x) => x >= 30)).toBe(-1)
  expect([10, 20, 30].slice(0, -1)).toEqual([10, 20])
  expect([10].slice(0, -1)).toEqual([])
  expect([1, 2, 3].reduceRight((total, value) => value - total, 0)).toBe(2)
  expect(
    ([] as number[]).reduceRight((total, value) => value - total, 10)
  ).toBe(10)
  const numbers = [30, 2, 10, 2]
  expect(numbers.slice().sort((a, b) => a - b)).toEqual([2, 2, 10, 30])
  expect(numbers).toEqual([30, 2, 10, 2])
  const people = [
    { name: "Aki", score: 2 },
    { name: "Mio", score: 1 },
    { name: "Ren", score: 2 },
  ]
  expect(
    people
      .slice()
      .sort((a, b) => a.score - b.score)
      .map((p) => p.name)
  ).toEqual(["Mio", "Aki", "Ren"])
  expect(people.map((p) => p.name)).toEqual(["Aki", "Mio", "Ren"])
  const groups = new Map<number, number[]>()
  for (const n of [3, 2, 5, 4, 7]) {
    const key = n % 2
    groups.set(key, [...(groups.get(key) ?? []), n])
  }
  expect([...groups.keys()]).toEqual([1, 0])
  expect(groups.get(1)).toEqual([3, 5, 7])
  expect(groups.get(9)).toBeUndefined()
  const last = (xs: number[]) =>
    xs.length === 0 ? "empty input" : `last: ${xs[xs.length - 1]}`
  expect(last([10, 20, 30])).toBe("last: 30")
  expect(last([])).toBe("empty input")
  const partition = (width: number, xs: number[], overlap: boolean) => {
    if (width <= 0) return { error: width }
    const groups: number[][] = []
    for (
      let i = 0;
      overlap ? i + width <= xs.length : i < xs.length;
      i += overlap ? 1 : width
    )
      groups.push(xs.slice(i, i + width))
    return { groups }
  }
  expect(partition(2, [10, 20, 30, 40, 50], false)).toEqual({
    groups: [[10, 20], [30, 40], [50]],
  })
  expect(partition(2, [10, 20, 30, 40], true)).toEqual({
    groups: [
      [10, 20],
      [20, 30],
      [30, 40],
    ],
  })
  expect(partition(5, [10, 20], true)).toEqual({ groups: [] })
  expect(partition(5, [10, 20], false)).toEqual({ groups: [[10, 20]] })
  expect(partition(2, [], false)).toEqual({ groups: [] })
  expect(partition(0, [], true)).toEqual({ error: 0 })
})
