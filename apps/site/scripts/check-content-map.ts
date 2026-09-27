import assert from "node:assert/strict"
import { existsSync, readFileSync } from "node:fs"
import { resolve } from "node:path"

const root = resolve(import.meta.dir, "../../..")
const mapPath = resolve(root, "docs/design/docs-site/reference-content-map.md")
const source = readFileSync(mapPath, "utf8")
const routes = [...source.matchAll(/`(\/[^`\s]+\/)`/gu)].map(
  ([, route]) => route
)

assert.equal(routes.length, 362, "Reference map route count changed")
assert.equal(
  new Set(routes).size,
  routes.length,
  "Reference routes must be unique"
)
for (const route of routes) {
  assert.match(route, /^\/docs\/(?:[a-z0-9-]+\/)+$/u)
}

const specSources = new Set(
  [...source.matchAll(/docs\/spec\/[A-Za-z0-9._/-]+/gu)].map(([path]) => path)
)
for (const path of specSources) {
  assert.ok(
    existsSync(resolve(root, path)),
    `Missing normative source: ${path}`
  )
}

console.log(
  `Reference content map: ${routes.length} unique routes, ${specSources.size} named specification sources`
)
