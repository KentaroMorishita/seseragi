import assert from "node:assert/strict"
import { dirname, relative, sep } from "node:path"
import {
  availabilityStates,
  type DocInline,
  type Locale,
  locales,
  type PageMetadata,
  type ParsedPage,
  pageKinds,
} from "./model"

const metadataKeys = [
  "availability",
  "examples",
  "id",
  "kind",
  "locale",
  "next",
  "prerequisites",
  "referenceIdentities",
  "related",
  "route",
  "schema",
  "spec",
  "summary",
  "title",
]

function text(value: unknown, label: string): string {
  assert.equal(typeof value, "string", `${label} must be text`)
  assert.ok((value as string).trim().length > 0, `${label} must not be empty`)
  return value as string
}

function textArray(value: unknown, label: string): string[] {
  assert.ok(Array.isArray(value), `${label} must be an array`)
  const values = value.map((entry) => text(entry, label))
  assert.equal(
    new Set(values).size,
    values.length,
    `${label} contains duplicates`
  )
  return values
}

function validateMetadata(metadata: PageMetadata, path: string): void {
  assert.ok(
    metadata && typeof metadata === "object",
    `${path}: metadata required`
  )
  assert.deepEqual(
    Object.keys(metadata).sort(),
    metadataKeys,
    `${path}: metadata keys`
  )
  assert.equal(metadata.schema, 1, `${path}: unsupported schema`)
  assert.match(
    metadata.id,
    /^[a-z0-9]+(?:[.-][a-z0-9]+)*$/u,
    `${path}: invalid id`
  )
  assert.ok(locales.includes(metadata.locale), `${path}: invalid locale`)
  assert.ok(pageKinds.includes(metadata.kind), `${path}: invalid page kind`)
  assert.ok(
    availabilityStates.includes(metadata.availability),
    `${path}: invalid availability`
  )
  assert.match(
    metadata.route,
    /^\/(?:[a-z0-9]+(?:-[a-z0-9]+)*\/)+$/u,
    `${path}: invalid route`
  )
  text(metadata.title, `${path}: title`)
  text(metadata.summary, `${path}: summary`)
  assert.ok(metadata.summary.length <= 180, `${path}: summary is too long`)
  assert.ok(
    textArray(metadata.spec, `${path}: spec`).length > 0,
    `${path}: spec required`
  )
  for (const example of textArray(metadata.examples, `${path}: examples`)) {
    assert.match(
      example,
      /^examples\/spec\/[a-zA-Z0-9_./-]+\.ssrg$/u,
      `${path}: invalid canonical example path`
    )
    assert.ok(
      !example.split("/").includes(".."),
      `${path}: invalid canonical example path`
    )
  }
  textArray(metadata.prerequisites, `${path}: prerequisites`)
  textArray(metadata.related, `${path}: related`)
  textArray(metadata.referenceIdentities, `${path}: referenceIdentities`)
  assert.equal(typeof metadata.next, "string", `${path}: next must be text`)
}

function linkInlines(page: ParsedPage): DocInline[] {
  return page.blocks
    .flatMap((block) => [...block.inlines, ...block.items.flat()])
    .filter((inline) => inline.kind === "link")
}

function sharedMetadata(metadata: PageMetadata) {
  const {
    locale: _locale,
    route: _route,
    title: _title,
    summary: _summary,
    ...shared
  } = metadata
  return shared
}

export function validateGlossaries(glossaries: Record<Locale, unknown>): void {
  const keys: string[][] = []
  for (const locale of locales) {
    const glossary = glossaries[locale] as Record<string, unknown>
    assert.ok(
      glossary && typeof glossary === "object",
      `${locale} glossary required`
    )
    assert.deepEqual(Object.keys(glossary).sort(), [
      "locale",
      "schema",
      "terms",
    ])
    assert.equal(glossary.schema, 1)
    assert.equal(glossary.locale, locale)
    assert.ok(glossary.terms && typeof glossary.terms === "object")
    const terms = glossary.terms as Record<string, unknown>
    const ids = Object.keys(terms).sort()
    assert.ok(ids.length > 0, `${locale} glossary is empty`)
    for (const id of ids) {
      assert.match(id, /^[a-z0-9]+(?:-[a-z0-9]+)*$/u)
      text(terms[id], `${locale} glossary ${id}`)
    }
    keys.push(ids)
  }
  assert.deepEqual(
    keys[1],
    keys[0],
    "Locale glossaries must use the same term ids"
  )
}

export function validatePages(pages: ParsedPage[], pagesRoot: string): void {
  const byId = new Map<string, ParsedPage[]>()
  const routes = new Set<string>()

  for (const page of pages) {
    validateMetadata(page.metadata, page.path)
    assert.equal(
      page.documentTitle,
      page.metadata.title,
      `${page.path}: H1 must match metadata title`
    )
    const relativeDirectory = relative(pagesRoot, dirname(page.path))
      .split(sep)
      .join(".")
    assert.equal(
      relativeDirectory,
      page.metadata.id,
      `${page.path}: directory must match page identity`
    )
    assert.ok(
      !routes.has(page.metadata.route),
      `Duplicate route ${page.metadata.route}`
    )
    routes.add(page.metadata.route)
    const entries = byId.get(page.metadata.id) ?? []
    entries.push(page)
    byId.set(page.metadata.id, entries)
  }

  for (const [id, entries] of byId) {
    assert.deepEqual(
      entries.map(({ metadata }) => metadata.locale).sort(),
      [...locales],
      `${id}: missing locale file`
    )
    const english = entries.find(({ metadata }) => metadata.locale === "en")!
    const japanese = entries.find(({ metadata }) => metadata.locale === "ja")!
    assert.equal(
      japanese.metadata.route,
      `/ja${english.metadata.route}`,
      `${id}: Japanese route must mirror English under /ja/`
    )
    assert.deepEqual(
      sharedMetadata(japanese.metadata),
      sharedMetadata(english.metadata),
      `${id}: locale-independent metadata differs`
    )
  }

  for (const page of pages) {
    const graphIds = [
      ...page.metadata.prerequisites,
      ...page.metadata.related,
      ...(page.metadata.next ? [page.metadata.next] : []),
    ]
    for (const id of graphIds) {
      assert.notEqual(id, page.metadata.id, `${page.path}: self graph edge`)
      assert.ok(byId.has(id), `${page.path}: unknown graph page ${id}`)
    }
    for (const inline of linkInlines(page)) {
      if (inline.href.startsWith("doc:")) {
        const id = inline.href.slice(4)
        assert.ok(byId.has(id), `${page.path}: unknown document link ${id}`)
      } else if (inline.href.startsWith("#")) {
        assert.match(inline.href, /^#[a-z0-9]+(?:-[a-z0-9]+)*$/u)
      } else {
        const url = new URL(inline.href)
        assert.equal(
          url.protocol,
          "https:",
          `${page.path}: links must use HTTPS`
        )
        assert.equal(
          url.username,
          "",
          `${page.path}: link credentials forbidden`
        )
        assert.equal(
          url.password,
          "",
          `${page.path}: link credentials forbidden`
        )
      }
    }
    for (const block of page.blocks) {
      if (block.kind === "example")
        assert.ok(
          page.metadata.examples.includes(block.source),
          `${page.path}: example directive is absent from metadata`
        )
      if (block.kind === "api-reference")
        assert.ok(
          page.metadata.referenceIdentities.includes(block.reference),
          `${page.path}: API identity is absent from metadata`
        )
    }
  }
}
