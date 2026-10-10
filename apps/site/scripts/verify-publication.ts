import assert from "node:assert/strict"
import { createHash } from "node:crypto"
import { readFileSync } from "node:fs"
import { join } from "node:path"
import { generatorInput } from "./build"
import { searchIndex } from "./search-index"

export function verifyPublication(output: string) {
  const manifest = JSON.parse(
    readFileSync(join(output, "site-manifest.json"), "utf8")
  )
  const routes = new Set<string>(manifest.pages)
  for (const route of routes) {
    const counterpart = route.startsWith("/ja/")
      ? route.slice(3)
      : route === "/"
        ? "/ja/"
        : `/ja${route}`
    assert.ok(routes.has(counterpart), `Missing locale counterpart: ${route}`)
  }
  for (const route of [
    "/",
    "/docs/",
    "/docs/search/",
    "/examples/",
    "/releases/",
    "/docs/first-run/",
    "/docs/language/",
    "/docs/types/",
    "/docs/types/records/",
    "/docs/types/variants/",
    "/docs/types/generics/",
    "/docs/composition/",
    "/docs/composition/transform/",
    "/docs/composition/apply/",
    "/docs/composition/bind/",
    "/docs/effects/",
    "/docs/effects/actions/",
    "/docs/effects/errors/",
    "/docs/effects/resources/",
    "/docs/signals/",
    "/docs/signals/derived/",
    "/docs/signals/subscriptions/",
    "/docs/signals/transactions/",
    "/docs/api/",
  ])
    assert.ok(routes.has(route), `Missing reader destination: ${route}`)
  const input = generatorInput("https://seseragi.vercel.app/")
  for (const module of input.referenceModules)
    for (const prefix of ["", "/ja"])
      assert.ok(
        routes.has(`${prefix}/docs/api/${module.specifier.slice(4)}/`),
        module.specifier
      )
  assert.deepEqual(
    manifest.examples,
    input.examples.map(({ id, sourcePath, sha256 }) => ({
      id,
      sourcePath,
      sha256,
    }))
  )
  assert.deepEqual(
    manifest.referenceModules,
    input.referenceModules.map(
      ({ specifier, availability, targets, items }) => ({
        specifier,
        availability,
        targets,
        symbols: items.map(({ identity, namespace, itemKind }) => ({
          identity,
          namespace,
          itemKind,
        })),
      })
    )
  )
  for (const file of manifest.files) {
    assert.equal(
      createHash("sha256")
        .update(readFileSync(join(output, file.path)))
        .digest("hex"),
      file.sha256,
      file.path
    )
  }
  const pages = manifest.pages.map((route: string) => ({
    route,
    html: readFileSync(join(output, route.slice(1), "index.html"), "utf8"),
  }))
  for (const locale of ["en", "ja"] as const) {
    const script = readFileSync(
      join(output, "assets", `search-index-${locale}.js`),
      "utf8"
    )
    assert.ok(script.startsWith("export default "))
    const actual = JSON.parse(
      script.slice("export default ".length).trim().replace(/;$/u, "")
    )
    assert.deepEqual(
      actual,
      searchIndex(pages, input.referenceModules, locale),
      `Search index differs from published content: ${locale}`
    )
  }
  console.info(
    `Publication verified: ${routes.size} routes, compiler API provenance, example hashes, and all asset hashes`
  )
  return manifest
}
