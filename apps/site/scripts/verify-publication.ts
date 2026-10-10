import assert from "node:assert/strict"
import { createHash } from "node:crypto"
import { readFileSync } from "node:fs"
import { join } from "node:path"
import { generatorInput } from "./build"

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
  console.info(
    `Publication verified: ${routes.size} routes, compiler API provenance, example hashes, and all asset hashes`
  )
  return manifest
}
