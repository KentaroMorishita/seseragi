import assert from "node:assert/strict"
import { existsSync, readFileSync } from "node:fs"
import { join } from "node:path"

export const docsBudgets = Object.freeze({
  clientJavascriptBytes: 1_600_000,
  largestHtmlBytes: 800_000,
  publishedBytes: 4_500_000,
  searchEntries: 2_000,
})

type Page = { route: string; title: string }
type File = { path: string; bytes: number; sha256: string }

const xml = (value: string) =>
  value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;")

export function sitemap(origin: string, pages: Page[]): string {
  return `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${pages
    .map(({ route }) => `  <url><loc>${xml(origin + route)}</loc></url>`)
    .join("\n")}\n</urlset>\n`
}

export function robots(origin: string, base: string): string {
  return `User-agent: *\nAllow: ${base}\nSitemap: ${origin}${base}sitemap.xml\n`
}

function occurrences(source: string, pattern: RegExp): number {
  return [...source.matchAll(pattern)].length
}

function publishedPath(pathname: string, base: string): string {
  assert.ok(
    pathname.startsWith(base),
    `Same-site URL escapes base: ${pathname}`
  )
  const relative = pathname.slice(base.length)
  return relative === "" || relative.endsWith("/")
    ? `${relative}index.html`
    : relative
}

export function validatePublishedSite(input: {
  root: string
  origin: string
  base: string
  pages: Page[]
  files: File[]
  clientJavascriptBytes: number
  searchEntries: number
}) {
  const paths = new Set(input.files.map(({ path }) => path))
  assert.equal(paths.size, input.files.length, "Duplicate published file")
  const pagePaths = new Map<string, string>()
  for (const page of input.pages) {
    const path = publishedPath(page.route, input.base)
    assert.ok(paths.has(path), `Missing page file: ${page.route}`)
    pagePaths.set(page.route, path)
  }
  const htmlByPath = new Map<string, string>()
  const idsByPath = new Map<string, Set<string>>()
  for (const [route, path] of pagePaths) {
    const html = readFileSync(join(input.root, path), "utf8")
    htmlByPath.set(path, html)
    assert.equal(occurrences(html, /<html\b/g), 1, `${route}: html landmark`)
    assert.ok(/<html\s[^>]*lang="ja"/.test(html), `${route}: language`)
    assert.equal(occurrences(html, /<main\b/g), 1, `${route}: main landmark`)
    assert.equal(occurrences(html, /<h1\b/g), 1, `${route}: h1`)
    assert.equal(occurrences(html, /<title>/g), 1, `${route}: title`)
    assert.equal(
      occurrences(html, /<meta\s[^>]*name="description"/g),
      1,
      `${route}: description`
    )
    assert.equal(
      occurrences(html, /<link\s[^>]*rel="canonical"/g),
      1,
      `${route}: canonical`
    )
    assert.ok(
      html.includes(`rel="canonical" href="${input.origin}${route}"`),
      `${route}: canonical URL`
    )
    const ids = new Set<string>()
    for (const match of html.matchAll(/\sid="([^"]+)"/g)) {
      assert.ok(!ids.has(match[1]), `${route}: duplicate id ${match[1]}`)
      ids.add(match[1])
    }
    idsByPath.set(path, ids)
  }

  for (const [route, path] of pagePaths) {
    const html = htmlByPath.get(path)!
    for (const match of html.matchAll(/\s(?:href|src)="([^"]+)"/g)) {
      const value = match[1]
      const resolved = new URL(value, input.origin + route)
      if (resolved.origin !== input.origin) continue
      assert.equal(resolved.search, "", `${route}: same-site query ${value}`)
      const targetPath = publishedPath(resolved.pathname, input.base)
      assert.ok(
        paths.has(targetPath) && existsSync(join(input.root, targetPath)),
        `${route}: missing target ${value}`
      )
      if (resolved.hash !== "") {
        const id = decodeURIComponent(resolved.hash.slice(1))
        assert.ok(
          idsByPath.get(targetPath)?.has(id),
          `${route}: missing fragment ${value}`
        )
      }
    }
  }

  const htmlFiles = input.files.filter(({ path }) => path.endsWith(".html"))
  const largestHtmlBytes = Math.max(...htmlFiles.map(({ bytes }) => bytes))
  const publishedBytes = input.files.reduce(
    (total, { bytes }) => total + bytes,
    0
  )
  assert.ok(
    input.clientJavascriptBytes <= docsBudgets.clientJavascriptBytes,
    `Client JavaScript budget exceeded: ${input.clientJavascriptBytes}`
  )
  assert.ok(
    largestHtmlBytes <= docsBudgets.largestHtmlBytes,
    `HTML page budget exceeded: ${largestHtmlBytes}`
  )
  assert.ok(
    publishedBytes <= docsBudgets.publishedBytes,
    `Published site budget exceeded: ${publishedBytes}`
  )
  assert.ok(
    input.searchEntries <= docsBudgets.searchEntries,
    `Search corpus budget exceeded: ${input.searchEntries}`
  )
  return { largestHtmlBytes, publishedBytes, budgets: docsBudgets }
}
