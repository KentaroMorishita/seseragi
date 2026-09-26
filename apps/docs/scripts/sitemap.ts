import { readdirSync, readFileSync } from "node:fs"
import { basename, resolve } from "node:path"

const SPEC_FILE_PATTERN = /^\d{2}-.+\.md$/
const SPEC_HEADING_PATTERN = /^(#{2,3})\s+(\d+\.\d+(?:\.\d+)*)\s+(.+)$/
const ROUTE_PATTERN = /`(\/[^`\s]+\/)`/g
const VALID_ROUTE_PATTERN = /^\/(?:[a-z0-9]+(?:-[a-z0-9]+)*\/)+$/
const SOURCE_ID_PATTERN = /\b\d+\.\d+(?:\.\d+)*\b/g

export type SpecSection = {
  file: string
  line: number
  level: 2 | 3
  id: string
  title: string
}

export type CoverageMap = {
  routes: Array<{ route: string; line: number }>
  sourceIds: Set<string>
}

export type SitemapReport = {
  specFiles: number
  specSections: number
  routes: number
  sourceIds: number
}

export function collectSpecSections(
  files: Array<{ file: string; source: string }>
): SpecSection[] {
  const sections: SpecSection[] = []
  const identities = new Map<string, SpecSection>()

  for (const { file, source } of files) {
    for (const [index, line] of source.split("\n").entries()) {
      const match = line.match(SPEC_HEADING_PATTERN)
      if (!match) continue
      const section: SpecSection = {
        file,
        line: index + 1,
        level: match[1].length as 2 | 3,
        id: match[2],
        title: match[3].trim(),
      }
      const existing = identities.get(section.id)
      if (existing)
        throw new Error(
          `Duplicate specification section ${section.id}: ` +
            `${existing.file}:${existing.line} and ${section.file}:${section.line}`
        )
      identities.set(section.id, section)
      sections.push(section)
    }
  }

  return sections
}

function routeTokens(text: string): string[] {
  return [...text.matchAll(ROUTE_PATTERN)].map((match) => match[1])
}

function sourceIds(text: string): string[] {
  return [...text.matchAll(SOURCE_ID_PATTERN)].map((match) => match[0])
}

export function collectCoverageMap(source: string): CoverageMap {
  const routes: CoverageMap["routes"] = []
  const mappedSourceIds = new Set<string>()
  const lines = source.split("\n")

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index]
    const table = line.match(/^\|\s*(.+?)\s*\|\s*(.+?)\s*\|/)
    if (table?.[1].includes("`/")) {
      for (const route of routeTokens(table[1]))
        routes.push({ route, line: index + 1 })
      for (const id of sourceIds(table[2])) mappedSourceIds.add(id)
      continue
    }

    if (line.startsWith("### `/")) {
      for (const route of routeTokens(line))
        routes.push({ route, line: index + 1 })
      continue
    }

    if (line.startsWith("Source:")) {
      let sourceBlock = line
      while (lines[index + 1]?.trim() && !lines[index + 1].startsWith("The ")) {
        index += 1
        sourceBlock += ` ${lines[index]}`
      }
      for (const id of sourceIds(sourceBlock)) mappedSourceIds.add(id)
      continue
    }

    if (line.startsWith("`/") && line.includes(" is a navigable")) {
      for (const route of routeTokens(line))
        routes.push({ route, line: index + 1 })
    }
  }

  return { routes, sourceIds: mappedSourceIds }
}

export function validateSitemap(
  sections: SpecSection[],
  coverage: CoverageMap,
  grammarSource: string
): SitemapReport {
  const errors: string[] = []
  const declaredRoutes = new Map<string, number>()

  for (const { route, line } of coverage.routes) {
    if (!VALID_ROUTE_PATTERN.test(route))
      errors.push(
        `Invalid canonical route ${route} at content map line ${line}`
      )
    const existingLine = declaredRoutes.get(route)
    if (existingLine)
      errors.push(
        `Duplicate canonical route ${route} at content map lines ` +
          `${existingLine} and ${line}`
      )
    else declaredRoutes.set(route, line)
  }

  for (const section of sections) {
    if (!coverage.sourceIds.has(section.id))
      errors.push(
        `Unmapped specification section ${section.id} ` +
          `(${section.file}:${section.line} ${section.title})`
      )
  }

  for (const id of coverage.sourceIds) {
    if (!sections.some((section) => section.id === id))
      errors.push(`Unknown specification section ${id} in content map`)
  }

  if (!declaredRoutes.has("/language/grammar/"))
    errors.push("Missing canonical route /language/grammar/")
  if (!grammarSource.startsWith("# Appendix A."))
    errors.push("docs/spec/grammar.md is not the expected Appendix A source")

  if (errors.length > 0)
    throw new Error(
      `Documentation sitemap validation failed:\n- ${errors.join("\n- ")}`
    )

  return {
    specFiles: new Set(sections.map(({ file }) => file)).size,
    specSections: sections.length,
    routes: declaredRoutes.size,
    sourceIds: coverage.sourceIds.size,
  }
}

export function validateRepositorySitemap(root: string): SitemapReport {
  const specRoot = resolve(root, "docs/spec")
  const specFiles = Array.from({ length: 18 }, (_, chapter) => {
    const prefix = chapter.toString().padStart(2, "0")
    const file = readdirSync(specRoot)
      .map((path) => basename(path))
      .find(
        (name) => name.startsWith(`${prefix}-`) && SPEC_FILE_PATTERN.test(name)
      )
    if (!file)
      throw new Error(`Missing normative specification chapter ${prefix}`)
    return {
      file: `docs/spec/${file}`,
      source: readFileSync(resolve(specRoot, file), "utf8"),
    }
  })
  const sections = collectSpecSections(specFiles)
  const coverage = collectCoverageMap(
    readFileSync(
      resolve(root, "docs/design/docs-site/reference-content-map.md"),
      "utf8"
    )
  )
  return validateSitemap(
    sections,
    coverage,
    readFileSync(resolve(specRoot, "grammar.md"), "utf8")
  )
}
