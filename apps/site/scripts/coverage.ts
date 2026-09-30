import assert from "node:assert/strict"
import { readFileSync } from "node:fs"
import { resolve } from "node:path"

const root = resolve(import.meta.dir, "../../..")
const map = readFileSync(
  resolve(root, "docs/design/docs-site/reference-content-map.md"),
  "utf8"
)

export const plannedReferenceRoutes = [
  ...map.matchAll(/`(\/[^`\s]+\/)`/gu),
].map(([, route]) => route)

export function referenceCoverage(publishedRoutes: string[]) {
  const published = new Set(publishedRoutes)
  const planned = new Set(plannedReferenceRoutes)
  const articles = publishedRoutes.filter((route) =>
    /^\/docs\/language\/.+\/$/u.test(route)
  )
  for (const route of articles) {
    assert.ok(planned.has(route), `Unmapped language article: ${route}`)
    assert.ok(published.has(`/ja${route}`), `Missing Japanese route: ${route}`)
  }
  const language = plannedReferenceRoutes.filter((route) =>
    route.startsWith("/docs/language/")
  )
  for (const route of language) {
    assert.ok(published.has(route), `Missing core reference article: ${route}`)
    assert.ok(
      published.has(`/ja${route}`),
      `Missing Japanese article: ${route}`
    )
  }
  return [
    "language",
    "interop",
    "projects",
    "tooling",
    "applications",
    "internals",
    "library",
    "packages",
  ].map((area) => {
    const routes = plannedReferenceRoutes.filter((route) =>
      route.startsWith(`/docs/${area}/`)
    )
    const missing = routes.filter((route) => !published.has(route))
    return {
      area,
      planned: routes.length,
      published: routes.length - missing.length,
      missing,
    }
  })
}
