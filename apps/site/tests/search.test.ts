import { expect, test } from "bun:test"
import { prepareSearch, type SearchEntry } from "../client/search-engine"
import { compilerReferenceModules } from "../scripts/reference"
import { searchIndex } from "../scripts/search-index"

const entries: SearchEntry[] = [
  {
    url: "/docs/signals/transactions/",
    title: "transaction — publish changes together",
    context: "Docs",
    description: "A progress label",
    body: "planUpdate applies changes in order",
  },
  {
    url: "/docs/api/signal/#map",
    title: "map",
    context: "std/signal",
    description: "(A -> B) -> Signal<A> -> Signal<B>",
    body: "std/signal::map value function",
  },
  {
    url: "/docs/api/array/#map",
    title: "map",
    context: "std/array",
    description: "(A -> B) -> Array<A> -> Array<B>",
    body: "std/array::map value function",
  },
  {
    url: "/docs/api/signal/#transaction",
    title: "transaction",
    context: "std/signal",
    description: "Array<SignalChange> -> Task<Unit>",
    body: "std/signal::transaction value function",
  },
  {
    url: "/docs/types/variants/",
    title: "match — 状態に応じて処理を分ける",
    context: "Docs",
    description: "状態を型で分ける",
    body: "網羅性を確かめる",
  },
  {
    url: "/docs/composition/apply/",
    title: "<*> — combine arguments",
    context: "Docs",
    description: "Applicative",
    body: "<$> maps the first argument",
  },
]

test("search ranks exact names, distinguishes modules, and matches prose, Japanese, and operators", () => {
  const find = prepareSearch(entries)
  expect(find("transaction").entries.map(({ url }) => url)).toEqual([
    entries[3].url,
    entries[0].url,
  ])
  expect(find("std/signal map").entries).toEqual([entries[1]])
  expect(find("std/array::map").entries).toEqual([entries[2]])
  expect(find("  ＭＡＰ   Signal  ").entries).toEqual([entries[1]])
  expect(find("網羅性").entries).toEqual([entries[4]])
  expect(find("<*>").entries).toEqual([entries[5]])
  expect(find("planUpdate").entries).toEqual([entries[0]])
  expect(find("SignalChange Never").total).toBe(0)
  expect(find("   ").total).toBe(0)
  expect(find("<img src=x onerror=alert(1)>").total).toBe(0)
})

test("search caps results without hiding the total and keeps tie order stable", () => {
  const many = Array.from({ length: 70 }, (_, index) => ({
    ...entries[1],
    url: `/item-${index}/`,
  }))
  const result = prepareSearch(many)("map")
  expect(result.total).toBe(70)
  expect(result.entries).toEqual(many.slice(0, 40))
})

test("search indexes the active locale and real declaration anchors, excluding navigation markup", () => {
  const module = compilerReferenceModules().find(
    ({ specifier }) => specifier === "std/signal"
  )
  if (!module) throw new Error("Missing std/signal")
  const item = module.items.find(({ name }) => name === "transaction")
  if (!item) throw new Error("Missing transaction")
  const modules = [{ ...module, items: [item] }]
  const article = (prefix: string) => ({
    route: `${prefix}/docs/composition/apply/`,
    html: '<nav>Unpublished navigation words</nav><meta name="description" content="A &amp; B"><main><h1>Use &lt;*&gt;</h1><article class="article-content"><h2>Combine</h2><p>First &#65;</p><p>Then &#x42;</p></article></main>',
  })
  const api = (prefix: string) => ({
    route: `${prefix}/docs/api/signal/`,
    html: `<main><h1>std/signal</h1><section id="${item.anchor}"></section></main>`,
  })
  const search = (prefix: string) => ({
    route: `${prefix}/docs/search/`,
    html: "<main><h1>Search</h1></main>",
  })
  const pages = [
    article(""),
    api(""),
    search(""),
    article("/ja"),
    api("/ja"),
    search("/ja"),
  ]
  const index = searchIndex(pages, modules, "ja")
  expect(index).toHaveLength(3)
  expect(index.every(({ url }) => url.startsWith("/ja/"))).toBe(true)
  const indexedArticle = index.find(({ url }) =>
    url.endsWith("/docs/composition/apply/")
  )
  expect(indexedArticle?.title).toBe("Use <*>")
  expect(indexedArticle?.description).toBe("A & B")
  expect(indexedArticle?.body).toBe("Combine First A Then B")
  expect(index).toEqual(searchIndex([...pages].reverse(), modules, "ja"))
  expect(prepareSearch(index)("Unpublished").total).toBe(0)
  expect(index[2].url).toBe(`/ja/docs/api/signal/#${item.anchor}`)
  expect(index[2].description).toBe(item.signature)
  expect(() => searchIndex([article("/ja")], modules, "ja")).toThrow(
    "Search module page missing"
  )
  expect(() =>
    searchIndex(
      [article("/ja"), { ...api("/ja"), html: "<h1>std/signal</h1>" }],
      modules,
      "ja"
    )
  ).toThrow("Search declaration anchor missing")
})
