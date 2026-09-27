import { expect, test } from "bun:test"
import { readFileSync } from "node:fs"
import { resolve } from "node:path"

const root = resolve(import.meta.dir, "../../..")

function readConfiguration(path: string) {
  return JSON.parse(readFileSync(resolve(root, path), "utf8"))
}

test("Vercel keeps the site preview separate from the root Playground", () => {
  const playground = readConfiguration("vercel.json")
  const site = readConfiguration("apps/site/vercel.json")

  expect(playground.buildCommand).toBe("cd apps/playground && bun run build")
  expect(playground.outputDirectory).toBe("apps/playground/dist")
  expect(playground.rewrites).toEqual([
    { source: "/(.*)", destination: "/index.html" },
  ])

  expect(site.buildCommand).toBe("bun run build:site:production")
  expect(site.outputDirectory).toBe("target/site")
  expect(site.trailingSlash).toBe(true)
  expect(site.rewrites).toBeUndefined()
})
