import assert from "node:assert/strict"
import { readFileSync } from "node:fs"
import { join } from "node:path"
import { highlightSeseragi } from "../../../playground/src/editor/seseragi-language"
import { playgroundUrlForSource } from "../../../playground/src/workspace/source-link"
import type { DocCorpus, Locale, PreparedPage } from "../content/model"
import type {
  NavigationSource,
  SidebarArea,
  SidebarPage,
  SiteBlock,
  SiteInput,
  SitePage,
} from "./model"

const key = (id: string, locale: Locale) => `${id}\0${locale}`

function slug(value: string): string {
  return value
    .toLowerCase()
    .normalize("NFKD")
    .replace(/[^a-z0-9]+/gu, "-")
    .replace(/^-|-$/gu, "")
}

function pageLink(page: PreparedPage, active: boolean): SidebarPage {
  return {
    title: page.metadata.title,
    route: page.metadata.route,
    active,
  }
}

function prepareSidebar(
  source: NavigationSource,
  pages: Map<string, PreparedPage>,
  locale: Locale,
  current: PreparedPage
): SidebarArea[] {
  return source.areas.map((area) => {
    const groups = area.groups.map((group) => {
      const sections = group.sections.map((section) => {
        const sectionPages = section.pages.map((id) => {
          const page = pages.get(key(id, locale))
          assert.ok(
            page,
            `Navigation references unknown page ${id} (${locale})`
          )
          return pageLink(page, page.metadata.id === current.metadata.id)
        })
        return {
          title: section.title[locale],
          active: sectionPages.some(({ active }) => active),
          pages: sectionPages,
        }
      })
      return {
        title: group.title[locale],
        active: sections.some(({ active }) => active),
        sections,
      }
    })
    const firstPage = groups.flatMap(({ sections }) =>
      sections.flatMap(({ pages }) => pages)
    )[0]
    return {
      id: area.id,
      title: area.title[locale],
      description: area.description[locale],
      route: firstPage?.route ?? "",
      active: groups.some(({ active }) => active),
      groups,
    }
  })
}

function breadcrumbs(
  sidebar: SidebarArea[],
  current: PreparedPage
): SidebarPage[] {
  if (current.metadata.kind === "home") return []
  if (current.metadata.kind === "landing")
    return [
      {
        title: current.metadata.title,
        route: current.metadata.route,
        active: true,
      },
    ]
  for (const area of sidebar)
    for (const group of area.groups)
      for (const section of group.sections)
        if (section.pages.some(({ active }) => active))
          return [
            { title: area.title, route: area.route, active: false },
            { title: group.title, route: "", active: false },
            { title: section.title, route: "", active: false },
            pageLink(current, true),
          ]
  return [pageLink(current, true)]
}

function siteBlock(
  block: PreparedPage["blocks"][number],
  playground: string
): SiteBlock {
  const code =
    block.kind === "example"
      ? block.text
      : block.kind === "api-reference"
        ? block.referenceSignature
        : ""
  return {
    ...block,
    anchorId:
      block.kind === "heading"
        ? slug(block.inlines.map(({ text }) => text).join(""))
        : "",
    signature: code
      ? highlightSeseragi(code).map(({ text, classes }) => ({
          text,
          className: classes,
        }))
      : [],
    playgroundUrl:
      block.kind === "example" ? playgroundUrlForSource(playground, code) : "",
  }
}

export function prepareSite(options: {
  corpus: DocCorpus
  contentRoot: string
  origin: string
  base: string
  playgroundUrl: string
}): SiteInput {
  const navigation = JSON.parse(
    readFileSync(join(options.contentRoot, "navigation.json"), "utf8")
  ) as NavigationSource
  assert.equal(navigation.schema, 1)
  const pages = new Map(
    options.corpus.pages.map((page) => [
      key(page.metadata.id, page.metadata.locale),
      page,
    ])
  )

  const prepared: SitePage[] = options.corpus.pages.map((page) => {
    const locale = page.metadata.locale
    const otherLocale: Locale = locale === "en" ? "ja" : "en"
    const alternate = pages.get(key(page.metadata.id, otherLocale))
    assert.ok(alternate, `Missing alternate locale for ${page.metadata.id}`)
    const sidebar = prepareSidebar(navigation, pages, locale, page)
    const previous = page.metadata.prerequisites.at(-1)
      ? pages.get(key(page.metadata.prerequisites.at(-1)!, locale))
      : undefined
    const next = page.metadata.next
      ? pages.get(key(page.metadata.next, locale))
      : undefined
    const blocks = page.blocks.map((block) =>
      siteBlock(block, options.playgroundUrl)
    )
    return {
      route: page.metadata.route,
      locale,
      kind: page.metadata.kind,
      title: page.metadata.title,
      summary: page.metadata.summary,
      alternateRoute: alternate.metadata.route,
      breadcrumbs: breadcrumbs(sidebar, page),
      sidebar,
      toc: blocks
        .filter(({ kind }) => kind === "heading")
        .map((block) => ({
          title: block.inlines.map(({ text }) => text).join(""),
          route: `#${block.anchorId}`,
          active: false,
        })),
      previousTitle: previous?.metadata.title ?? "",
      previousRoute: previous?.metadata.route ?? "",
      nextTitle: next?.metadata.title ?? "",
      nextRoute: next?.metadata.route ?? "",
      blocks,
    }
  })

  return {
    schema: 1,
    origin: options.origin,
    base: options.base,
    playgroundUrl: options.playgroundUrl,
    pages: prepared,
  }
}
