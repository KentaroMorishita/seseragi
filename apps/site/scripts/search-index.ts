import assert from "node:assert/strict"
import type { SearchEntry } from "../client/search-engine"
import type { compilerReferenceModules } from "./reference"
import type { RenderedPage } from "./render-generator"

type ReferenceModules = ReturnType<typeof compilerReferenceModules>

// These are the entities emitted by the typed HTML renderer. Decode after
// extracting text so escaped prose never becomes markup or navigation.
function text(value: string) {
  const entities: Record<string, string> = {
    amp: "&",
    lt: "<",
    gt: ">",
    quot: '"',
    apos: "'",
  }
  return value
    .replace(/&(#x[0-9a-f]+|#[0-9]+|amp|lt|gt|quot|apos);/giu, (_, entity) => {
      if (!entity.startsWith("#")) return entities[entity.toLowerCase()]
      return String.fromCodePoint(
        entity.toLowerCase().startsWith("#x")
          ? Number.parseInt(entity.slice(2), 16)
          : Number.parseInt(entity.slice(1), 10)
      )
    })
    .replace(/\s+/gu, " ")
    .trim()
}

export function searchIndex(
  pages: RenderedPage[],
  modules: ReferenceModules,
  locale: "en" | "ja"
): SearchEntry[] {
  const prefix = locale === "ja" ? "/ja" : ""
  const selected = pages
    .filter(({ route }) => route.startsWith("/ja/") === (locale === "ja"))
    .sort((a, b) => (a.route < b.route ? -1 : a.route > b.route ? 1 : 0))
  const ids = new Map<string, Set<string>>()
  const entries: SearchEntry[] = []
  for (const { route, html } of selected) {
    const title: string[] = []
    const body: string[] = []
    const fragments = new Set<string>()
    let description = ""
    let headings = 0
    new HTMLRewriter()
      .on("h1", {
        element() {
          headings++
        },
        text(chunk) {
          title.push(chunk.text)
        },
      })
      .on('meta[name="description"]', {
        element(element) {
          description = element.getAttribute("content") ?? ""
        },
      })
      .on("main .article-content p, main h2, main h3", {
        element() {
          body.push(" ")
        },
        text(chunk) {
          body.push(chunk.text)
        },
      })
      .on("[id]", {
        element(element) {
          fragments.add(element.getAttribute("id") ?? "")
        },
      })
      .transform(html)
    ids.set(route, fragments)
    assert.equal(headings, 1, `Search needs one title: ${route}`)
    if (route === `${prefix}/docs/search/`) continue
    const isModule = route.startsWith(`${prefix}/docs/api/`)
    entries.push({
      url: route,
      title: text(title.join("")),
      context: isModule ? "API Reference" : "Docs",
      description: text(description),
      body: text(body.join("")),
    })
  }
  for (const module of modules) {
    const route = `${prefix}/docs/api/${module.specifier.slice(4)}/`
    assert.ok(ids.has(route), `Search module page missing: ${route}`)
    for (const item of module.items) {
      assert.ok(
        ids.get(route)?.has(item.anchor),
        `Search declaration anchor missing: ${item.identity}`
      )
      entries.push({
        url: `${route}#${item.anchor}`,
        title: item.name,
        context: module.specifier,
        description: item.signature,
        body: [item.identity, item.namespace, item.itemKind].join(" "),
      })
    }
  }
  assert.equal(new Set(entries.map(({ url }) => url)).size, entries.length)
  return entries
}
