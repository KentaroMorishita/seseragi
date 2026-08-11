import { previewUtilityCss } from "./preview-document"

export type PreviewClassSource = {
  readonly path: string
  readonly content: string
  readonly format: "html" | "seseragi"
}

export type PreviewSampleClassContract = {
  readonly id: string
  readonly sources: readonly PreviewClassSource[]
  readonly customClasses?: readonly string[]
  readonly dynamicUtilities?: readonly string[]
}

type DynamicClassName = {
  readonly line: number
  readonly expression: string
}

type DirectClassNameLiteral = {
  readonly line: number
  readonly tokens: readonly string[]
}

type CxClassList = {
  readonly line: number
}

type ExtractedSeseragiClasses = {
  readonly tokens: readonly string[]
  readonly dynamicClassNames: readonly DynamicClassName[]
}

export const previewUtilityTokens =
  extractPreviewUtilityTokens(previewUtilityCss)

export function extractPreviewUtilityTokens(css: string): readonly string[] {
  const tokens = new Set<string>()
  let boundary = 0
  for (let index = 0; index < css.length; index += 1) {
    const character = css[index]
    if (character === "}") {
      boundary = index + 1
      continue
    }
    if (character !== "{") continue

    const prelude = css.slice(boundary, index).trim()
    boundary = index + 1
    if (prelude.startsWith("@")) continue

    for (const match of prelude.matchAll(/\.((?:\\.|[\w-])+)/gu)) {
      const selector = match[1]
      if (selector === undefined) continue
      tokens.add(selector.replace(/\\(.)/gu, "$1"))
    }
  }
  return [...tokens].sort()
}

export function validatePreviewUtilityUsage(
  samples: readonly PreviewSampleClassContract[]
): void {
  const supported = new Set(previewUtilityTokens)
  const problems: string[] = []

  for (const sample of samples) {
    const custom = new Set(sample.customClasses ?? [])
    const dynamicUtilities = sample.dynamicUtilities ?? []
    const hasDynamicContract = dynamicUtilities.length > 0

    for (const source of sample.sources) {
      if (source.format === "html") {
        validateTokens(
          extractHtmlClassTokens(source.content),
          sample.id,
          source.path,
          supported,
          custom,
          problems
        )
        continue
      }

      const extracted = extractSeseragiClassTokens(source.content)
      validateTokens(
        extracted.tokens,
        sample.id,
        source.path,
        supported,
        custom,
        problems
      )
      if (!hasDynamicContract) {
        for (const dynamic of extracted.dynamicClassNames) {
          problems.push(
            `sample ${sample.id} file ${source.path}:${dynamic.line} ` +
              `has dynamic className ${JSON.stringify(dynamic.expression)}; ` +
              "declare preview.dynamicUtilities"
          )
        }
      }
    }

    validateTokens(
      dynamicUtilities,
      sample.id,
      "sample.json",
      supported,
      custom,
      problems
    )
  }

  if (problems.length > 0) {
    throw new Error(
      `Preview utility validation failed:\n${problems
        .sort()
        .map((problem) => `- ${problem}`)
        .join("\n")}`
    )
  }
}

export function validatePreviewSourceReadability(
  samples: readonly PreviewSampleClassContract[]
): void {
  const problems: string[] = []

  for (const sample of samples) {
    const sources = sample.sources.filter(
      (source) => source.format === "seseragi"
    )
    let usesCx = false

    for (const source of sources) {
      const literals = extractDirectClassNameLiterals(source.content)
      for (const literal of literals) {
        if (literal.tokens.length < 5) continue
        problems.push(
          `sample ${sample.id} file ${source.path}:${literal.line} has ` +
            `${literal.tokens.length}-token className literal; define a cx [...] value`
        )
      }

      const cxLists = extractCxClassLists(source.content)
      usesCx ||= cxLists.length > 0

      if (
        cxLists.length > 0 &&
        !hasLocalCxDefinition(source.content) &&
        !hasCxImport(source.content)
      ) {
        problems.push(
          `sample ${sample.id} file ${source.path} uses cx without a local ` +
            "definition or an explicit import"
        )
      }
    }

    if (!usesCx) continue
    if (
      sources.length === 1 &&
      !hasLocalCxDefinition(sources[0]?.content ?? "")
    ) {
      problems.push(
        `single-file sample ${sample.id} must define its own cx helper`
      )
    }
    if (
      sources.length > 1 &&
      !sources.some(
        (source) =>
          /(?:^|\/)styles\.ssrg$/u.test(source.path) &&
          /\bpub\s+fn\s+cx\b/u.test(source.content)
      )
    ) {
      problems.push(
        `project sample ${sample.id} must expose cx from a workspace styles.ssrg file`
      )
    }
  }

  if (problems.length > 0) {
    throw new Error(
      `Preview source readability validation failed:\n${problems
        .sort()
        .map((problem) => `- ${problem}`)
        .join("\n")}`
    )
  }
}

export function extractSeseragiClassTokens(
  source: string
): ExtractedSeseragiClasses {
  const searchable = maskCommentsAndTemplates(source)
  const tokens = new Set<string>()
  const cxBindings = new Set<string>()
  const dynamicClassNames: DynamicClassName[] = []

  for (const match of searchable.matchAll(
    /\b(?:pub\s+)?let\s+([a-z][\w']*)\s*=\s*cx\s*\[/gu
  )) {
    if (match[1] !== undefined) cxBindings.add(match[1])
  }

  for (const match of searchable.matchAll(/\bcx\s*\[/gu)) {
    const open = searchable.indexOf("[", match.index)
    const close = findClosingBracket(searchable, open)
    if (close === -1) continue
    addQuotedClassLists(source.slice(open + 1, close), tokens)
  }

  const propertyPattern = /\bclassName\s*:\s*/gu
  for (const match of searchable.matchAll(propertyPattern)) {
    const valueStart = (match.index ?? 0) + match[0].length
    if (searchable[valueStart] === '"') {
      const literal = readQuotedString(source, valueStart)
      if (literal !== undefined) addClassList(literal.value, tokens)
      continue
    }

    const expression = searchable.slice(valueStart)
    if (/^cx\s*\[/u.test(expression)) continue
    const identifier = expression.match(/^([a-z][\w']*)/u)?.[1]
    if (identifier !== undefined && cxBindings.has(identifier)) continue
    dynamicClassNames.push({
      line: lineNumber(source, valueStart),
      expression: expression.match(/^[^,}\n]*/u)?.[0]?.trim() || "<expression>",
    })
  }

  for (const match of searchable.matchAll(/\bclassName\s*,/gu)) {
    if (cxBindings.has("className")) continue
    dynamicClassNames.push({
      line: lineNumber(source, match.index ?? 0),
      expression: "className",
    })
  }

  return {
    tokens: [...tokens].sort(),
    dynamicClassNames,
  }
}

export function extractHtmlClassTokens(html: string): readonly string[] {
  const tokens = new Set<string>()
  for (const match of html.matchAll(/\bclass\s*=\s*(["'])(.*?)\1/gsu)) {
    addClassList(match[2] ?? "", tokens)
  }
  return [...tokens].sort()
}

function extractDirectClassNameLiterals(
  source: string
): readonly DirectClassNameLiteral[] {
  const searchable = maskCommentsAndTemplates(source)
  const literals: DirectClassNameLiteral[] = []

  for (const match of searchable.matchAll(/\bclassName\s*:\s*/gu)) {
    const valueStart = (match.index ?? 0) + match[0].length
    if (searchable[valueStart] !== '"') continue
    const literal = readQuotedString(source, valueStart)
    if (literal === undefined) continue
    literals.push({
      line: lineNumber(source, valueStart),
      tokens: literal.value.split(/\s+/u).filter((token) => token !== ""),
    })
  }

  return literals
}

function extractCxClassLists(source: string): readonly CxClassList[] {
  const searchable = maskCommentsAndTemplates(source)
  const lists: CxClassList[] = []

  for (const match of searchable.matchAll(/\bcx\s*\[/gu)) {
    const open = searchable.indexOf("[", match.index)
    const close = findClosingBracket(searchable, open)
    if (close === -1) continue
    lists.push({ line: lineNumber(source, open) })
  }

  return lists
}

function hasLocalCxDefinition(source: string): boolean {
  return /\b(?:pub\s+)?fn\s+cx\b/u.test(maskCommentsAndTemplates(source))
}

function hasCxImport(source: string): boolean {
  return /\bimport\s+\{[^}]*\bcx\b[^}]*\}\s+from\s+"[^"\n]+"/su.test(source)
}

function validateTokens(
  tokens: readonly string[],
  sampleId: string,
  path: string,
  supported: ReadonlySet<string>,
  custom: ReadonlySet<string>,
  problems: string[]
): void {
  for (const token of tokens) {
    if (supported.has(token) || custom.has(token)) continue
    problems.push(
      `sample ${sampleId} file ${path} uses undefined utility ` +
        JSON.stringify(token)
    )
  }
}

function addQuotedClassLists(source: string, tokens: Set<string>): void {
  for (const match of source.matchAll(/"((?:\\.|[^"\\])*)"/gsu)) {
    addClassList(match[1] ?? "", tokens)
  }
}

function addClassList(value: string, tokens: Set<string>): void {
  for (const token of value.split(/\s+/u)) {
    if (token !== "") tokens.add(token)
  }
}

function readQuotedString(
  source: string,
  start: number
): { readonly value: string; readonly end: number } | undefined {
  let escaped = false
  for (let index = start + 1; index < source.length; index += 1) {
    const character = source[index]
    if (escaped) {
      escaped = false
      continue
    }
    if (character === "\\") {
      escaped = true
      continue
    }
    if (character === '"') {
      return { value: source.slice(start + 1, index), end: index + 1 }
    }
  }
  return undefined
}

function findClosingBracket(source: string, open: number): number {
  if (open < 0) return -1
  let depth = 0
  let inString = false
  let escaped = false
  for (let index = open; index < source.length; index += 1) {
    const character = source[index]
    if (inString) {
      if (escaped) escaped = false
      else if (character === "\\") escaped = true
      else if (character === '"') inString = false
      continue
    }
    if (character === '"') {
      inString = true
      continue
    }
    if (character === "[") depth += 1
    if (character === "]") {
      depth -= 1
      if (depth === 0) return index
    }
  }
  return -1
}

function maskCommentsAndTemplates(source: string): string {
  const result = source.split("")
  let index = 0
  while (index < source.length) {
    if (source.startsWith("//", index)) {
      const end = source.indexOf("\n", index)
      const limit = end === -1 ? source.length : end
      result.fill(" ", index, limit)
      index = limit
      continue
    }
    if (source.startsWith("/*", index)) {
      const end = source.indexOf("*/", index + 2)
      const limit = end === -1 ? source.length : end + 2
      for (let cursor = index; cursor < limit; cursor += 1) {
        if (source[cursor] !== "\n") result[cursor] = " "
      }
      index = limit
      continue
    }
    if (source[index] === '"') {
      const literal = readQuotedString(source, index)
      const limit = literal?.end ?? source.length
      for (let masked = index + 1; masked < limit - 1; masked += 1) {
        if (source[masked] !== "\n") result[masked] = " "
      }
      index = limit
      continue
    }
    if (source[index] === "`") {
      let cursor = index + 1
      let escaped = false
      while (cursor < source.length) {
        const character = source[cursor]
        if (escaped) escaped = false
        else if (character === "\\") escaped = true
        else if (character === "`") {
          cursor += 1
          break
        }
        cursor += 1
      }
      for (let masked = index; masked < cursor; masked += 1) {
        if (source[masked] !== "\n") result[masked] = " "
      }
      index = cursor
      continue
    }
    index += 1
  }
  return result.join("")
}

function lineNumber(source: string, offset: number): number {
  return source.slice(0, offset).split("\n").length
}
