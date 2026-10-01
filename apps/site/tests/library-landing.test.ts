import { expect, setDefaultTimeout, test } from "bun:test"
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { dirname, join, resolve } from "node:path"
import { compilerReferenceModules } from "../scripts/reference"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(180_000)
const root = resolve(import.meta.dir, "../../..")
const modules = compilerReferenceModules().map((module) => ({
  ...module,
  items: [],
}))
const core = [
  "data/tuples-arrays-and-lists",
  "effects/either",
  "types/built-in-types",
  "effects/effect-type",
  "effects/environment-requirements",
  "effects/runtime-boundaries",
  "syntax/function-application",
]
const anchors = [
  "choose-by-task",
  "collections",
  "text-and-json",
  "numbers-and-bytes",
  "state-and-concurrency",
  "files-and-processes",
  "web-and-network",
  "targets-and-prerequisites",
]
function text(html: string) {
  return html
    .replace(/<[^>]+>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
}
function escapeHtml(value: string) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll("'", "&#39;")
}
function title(directory: string, locale: string) {
  const source = readFileSync(
    join(root, "apps/site/src/pages", directory, `${locale}.ssrg`),
    "utf8"
  )
  const literal =
    source.match(/\btitle:\s*("(?:[^"\\]|\\.)*")/u)?.[1] ??
    source.match(/pub fn title -> String =\s*("(?:[^"\\]|\\.)*")/u)?.[1]
  if (!literal) throw new Error(`Missing title ${directory}/${locale}`)
  return JSON.parse(literal) as string
}
function purposes(locale: string) {
  const source = readFileSync(
    join(root, "apps/site/src/reference/module-copy.ssrg"),
    "utf8"
  )
  return new Map(
    [
      ...source.matchAll(
        /("std\/[^"\n]+") -> localized ("(?:[^"\\]|\\.)*") ("(?:[^"\\]|\\.)*")/gu
      ),
    ].map((match) => [
      JSON.parse(match[1]),
      JSON.parse(match[locale === "en" ? 2 : 3]),
    ])
  )
}
type Section = { id: string; title: string; path: string }
type Group = { id: string; title: string; sections: Section[] }
type Output = {
  route: string
  locale: string
  html: string
  generic: string
  groups: Group[]
}

test("Library landing renders six task choices and all 63 exact purpose/target pairs in both locales", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-library-landing-"))
  try {
    const input = {
      schema: 1,
      origin: "https://seseragi.example",
      playgroundUrl: "https://seseragi.vercel.app/",
      tourUrl: "https://seseragi.vercel.app/tour/",
      grammar: "",
      examples: [],
      referenceModules: modules,
    }
    const output = renderPageClosure<Output[]>({
      directory,
      timeoutMs: 150_000,
      modules: [
        "pages/library/overview/page",
        "reference/catalog",
        "render/document",
      ],
      entry: `import * as json from "std/json"
import { decodeBuildInput } from "./model/build"
import { En, Ja, localeCode, localized, localizedRoute, translate } from "./model/locale"
import { DocumentationLanding, NavigationArea, PageDefinition, SiteCatalog } from "./model/page"
import { page } from "./pages/library/overview/page"
import { referenceGroups } from "./reference/catalog"
import { renderDocument } from "./render/document"
struct SectionOutput deriving JsonEncode { id: String, title: String, path: String }
struct GroupOutput deriving JsonEncode { id: String, title: String, sections: Array<SectionOutput> }
struct Output deriving JsonEncode { route: String, locale: String, html: String, generic: String, groups: Array<GroupOutput> }
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(input))} {
  Left _ -> println "[]"
  Right input -> {
    let library = page ()
    let groups = referenceGroups input.referenceModules
    let site = SiteCatalog { home: library, pages: [library], areas: [NavigationArea {
      id: "library", title: library.title, description: library.summary, landing: library, groups
    }] }
    let generic = PageDefinition { id: "other.overview", path: "/docs/other/", kind: DocumentationLanding,
      title: localized "Other" "その他", summary: localized "Other" "その他", blocks: [] }
    let otherSite = SiteCatalog { home: generic, pages: [generic], areas: [NavigationArea {
      id: "other", title: generic.title, description: generic.summary, landing: generic, groups
    }] }
    println (json.encodeString [Output {
      route: localizedRoute locale library.path, locale: localeCode locale,
      html: renderDocument input site locale library,
      generic: renderDocument input otherSite locale generic,
      groups: [GroupOutput { id: group.id, title: translate locale group.title,
        sections: [SectionOutput { id: section.id, title: translate locale section.title,
          path: localizedRoute locale section.overview.path } | section <- group.sections]
      } | group <- groups]
    } | locale <- [En, Ja]])
  }
}`,
    })
    expect(output).toHaveLength(2)
    expect(modules).toHaveLength(63)
    for (const page of output) {
      const prefix = page.locale === "ja" ? "/ja" : ""
      expect(page.route).toBe(`${prefix}/docs/library/`)
      if (process.env.SESERAGI_LIBRARY_LANDING_OUTPUT) {
        const path = join(
          resolve(process.env.SESERAGI_LIBRARY_LANDING_OUTPUT),
          page.route.slice(1),
          "index.html"
        )
        mkdirSync(dirname(path), { recursive: true })
        writeFileSync(path, page.html)
      }
      expect(page.html).toContain(`<html lang="${page.locale}">`)
      expect(page.html).not.toContain("site-build-error")
      expect(page.html.match(/<h1>/gu)).toHaveLength(1)
      expect(page.html).toContain(
        `href="${page.locale === "ja" ? "" : "/ja"}/docs/library/"`
      )
      const headingIds = [
        ...page.html.matchAll(/<h2\b[^>]*\bid="([^"]+)"/gu),
      ].map((match) => match[1])
      expect(new Set(headingIds).size).toBe(headingIds.length)
      for (const anchor of anchors) expect(headingIds).toContain(anchor)
      expect(page.groups).toHaveLength(12)
      for (const group of page.groups)
        expect(headingIds).toContain(`directory-${group.id}`)
      const start = page.html.indexOf('<div class="reference-directory">')
      const directoryHtml = page.html.slice(
        start,
        page.html.indexOf("</main>", start)
      )
      const cards = [
        ...directoryHtml.matchAll(
          /<div class="directory-entry">([\s\S]*?)<\/div>/gu
        ),
      ]
      expect(cards).toHaveLength(63)
      expect(directoryHtml.match(/class="directory-link"/gu)).toHaveLength(63)
      const found = new Set<string>()
      const expectedPurposes = purposes(page.locale)
      expect(expectedPurposes.size).toBe(63)
      if (page.locale === "en")
        expect(expectedPurposes.get("std/list")).toBe(
          "Build and transform immutable lists whose tails can be shared."
        )
      else
        expect(expectedPurposes.get("std/web/storage")).toContain(
          "キーがない場合や読み書きの失敗"
        )
      for (const [, card] of cards) {
        const link = card.match(
          /<a\b[^>]*href="([^"]+)"[^>]*>([\s\S]*?)<\/a>/u
        )!
        const specifier = text(link[2])
        expect(found.has(specifier)).toBe(false)
        found.add(specifier)
        const module = modules.find(
          (candidate) => candidate.specifier === specifier
        )
        expect(module, specifier).toBeDefined()
        if (!module) continue
        expect(link[1]).toBe(`${prefix}/docs/library/${specifier.slice(4)}/`)
        const purpose = text(
          card.match(/<p class="directory-purpose">([\s\S]*?)<\/p>/u)![1]
        )
        const targets = text(
          card.match(/<p class="directory-targets">([\s\S]*?)<\/p>/u)![1]
        )
        expect(purpose, specifier).toBe(expectedPurposes.get(specifier))
        expect(targets, specifier).toBe(
          `${page.locale === "en" ? "Targets: " : "対象: "}${module.targets.join(", ")}`
        )
      }
      expect([...found].sort()).toEqual(
        modules.map((module) => module.specifier).sort()
      )
      const article = page.html.match(
        /<article class="article-content">([\s\S]*?)<\/article>/u
      )![1]
      const links = [
        ...article.matchAll(/<a\b[^>]*href="([^"]+)"[^>]*>([\s\S]*?)<\/a>/gu),
      ].map((match) => [text(match[1]), text(match[2])])
      for (const route of core)
        expect(links).toContainEqual([
          `${prefix}/docs/language/${route}/`,
          title(`language/${route}`, page.locale),
        ])
      expect(links).toContainEqual([
        `${prefix}/docs/first-run/`,
        title("docs/first-run", page.locale),
      ])
      for (const [href, label] of links.filter(([href]) =>
        href.includes("#")
      )) {
        const fragment = href.split("#")[1]
        expect(headingIds).toContain(fragment)
        expect(
          page.groups.find((group) => `directory-${group.id}` === fragment)
            ?.title
        ).toBe(label)
      }
      // Task jumps point within this page, so each module destination remains a
      // single directory entry rather than a second competing list in the body.
      expect(
        links.filter(([href]) =>
          modules.some(
            (module) =>
              href === `${prefix}/docs/library/${module.specifier.slice(4)}/`
          )
        )
      ).toHaveLength(0)
      for (const term of [
        "Maybe",
        "Just",
        "Nothing",
        "Left",
        "Right",
        "Effect",
        "FileSystem",
        "HttpClient",
        "process",
        "browser",
      ])
        expect(text(article)).toContain(term)
      expect(text(article)).toContain(
        page.locale === "en"
          ? "does not grant access to arbitrary"
          : "OS上の任意のファイルを読めるわけではありません"
      )
      expect(text(article)).toContain(
        page.locale === "en"
          ? "does not supply those services"
          : "そのサービスを用意したことにはなりません"
      )
      // An unrelated landing receives the same input metadata. Its directory
      // retains the pre-change plain-link markup exactly, including group IDs.
      const genericStart = page.generic.indexOf(
        '<div class="reference-directory">'
      )
      const genericDirectory = page.generic.slice(
        genericStart,
        page.generic.indexOf("</main>", genericStart)
      )
      const expected =
        '<div class="reference-directory">' +
        page.groups
          .map(
            (group) =>
              `<section class="directory-group"><h2 id="directory-${escapeHtml(group.id)}">${escapeHtml(group.title)}</h2><nav>` +
              group.sections
                .map(
                  (section) =>
                    `<a class="directory-link" href="${escapeHtml(section.path)}">${escapeHtml(section.title)}</a>`
                )
                .join("") +
              "</nav></section>"
          )
          .join("") +
        "</div>"
      expect(genericDirectory).toBe(expected)
      expect(page.generic).not.toContain('class="directory-purpose"')
      expect(page.generic).not.toContain('class="directory-targets"')
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
