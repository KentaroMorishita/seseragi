import assert from "node:assert/strict"
import { readdirSync, readFileSync, statSync } from "node:fs"
import { basename, join, relative } from "node:path"
import { type Locale, locales } from "./model"

export type ContentConfig = {
  schema: 1
  defaultLocale: Locale
  locales: Locale[]
  missingTranslation: "error"
}

function files(root: string, current = root): string[] {
  return readdirSync(current)
    .flatMap((name) => {
      const path = join(current, name)
      return statSync(path).isDirectory() ? files(root, path) : [path]
    })
    .sort()
}

export function discoverAuthoringFiles(contentRoot: string): string[] {
  const pagesRoot = join(contentRoot, "pages")
  const discovered = files(pagesRoot).filter((path) => path.endsWith(".md"))
  assert.ok(discovered.length > 0, "At least one authored page is required")
  for (const path of discovered)
    assert.ok(
      locales.some((locale) => basename(path) === `${locale}.md`),
      `Unexpected authoring file: ${relative(contentRoot, path)}`
    )
  return discovered
}

export function readContentConfig(contentRoot: string): ContentConfig {
  const config = JSON.parse(
    readFileSync(join(contentRoot, "content.config.json"), "utf8")
  ) as ContentConfig
  assert.deepEqual(Object.keys(config).sort(), [
    "defaultLocale",
    "locales",
    "missingTranslation",
    "schema",
  ])
  assert.equal(config.schema, 1)
  assert.equal(config.defaultLocale, "en")
  assert.deepEqual(config.locales, [...locales])
  assert.equal(config.missingTranslation, "error")
  return config
}

export function readGlossaries(contentRoot: string): Record<Locale, unknown> {
  return Object.fromEntries(
    locales.map((locale) => [
      locale,
      JSON.parse(
        readFileSync(join(contentRoot, "glossary", `${locale}.json`), "utf8")
      ),
    ])
  ) as Record<Locale, unknown>
}
