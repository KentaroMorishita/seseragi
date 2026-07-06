import { existsSync, readdirSync, readFileSync } from "node:fs"
import { basename, dirname, join, resolve } from "node:path"

const root = resolve(import.meta.dir, "../examples/spec")
const specDir = resolve(import.meta.dir, "../docs/spec")
const lessonsDir = join(root, "lessons")
const fixturesDir = join(root, "fixtures")
const errors: string[] = []
const specSections = new Set<string>()
const diagnosticRegistry = new Map<string, string>()

const collectFiles = (directory: string, suffix: string): string[] =>
  readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name)
    return entry.isDirectory()
      ? collectFiles(path, suffix)
      : entry.name.endsWith(suffix)
        ? [path]
        : []
  })

for (const name of readdirSync(specDir).filter((name) =>
  name.endsWith(".md")
)) {
  const source = readFileSync(join(specDir, name), "utf8")
  for (const match of source.matchAll(/^## (\d+\.\d+)\b/gm)) {
    specSections.add(match[1])
  }
}

const toolingSpec = readFileSync(join(specDir, "12-tooling.md"), "utf8")
for (const match of toolingSpec.matchAll(
  /^\|\s*(SES-[PNTIEKFL]\d{4})\s*\|\s*(Error|Warning|Information|Hint)\s*\|/gm
)) {
  diagnosticRegistry.set(match[1], match[2])
}

if (diagnosticRegistry.size === 0) {
  errors.push("docs/spec/12-tooling.md: diagnostic registry is empty")
}

const syntaxSpec = readFileSync(join(specDir, "01-syntax.md"), "utf8")
const previewGrammarPath = resolve(
  import.meta.dir,
  "../extensions/seseragi-spec-preview/syntaxes/seseragi.tmLanguage.json"
)
const previewGrammar = readFileSync(previewGrammarPath, "utf8")
type TextMateRule = {
  name?: string
  match?: string
  begin?: string
  patterns?: TextMateRule[]
}
type TextMateGrammar = {
  repository?: Record<string, TextMateRule>
}
const previewGrammarObject = JSON.parse(previewGrammar) as TextMateGrammar
const flattenTextMateRules = (rule: TextMateRule): TextMateRule[] => [
  rule,
  ...(rule.patterns?.flatMap(flattenTextMateRules) ?? []),
]
const previewRules = Object.values(
  previewGrammarObject.repository ?? {}
).flatMap(flattenTextMateRules)
const reservedBlock = syntaxSpec.match(
  /## 1\.9 予約語[\s\S]*?```text\n([^`]+)```/
)
if (!reservedBlock) {
  errors.push("docs/spec/01-syntax.md: reserved word block is missing")
} else {
  const reservedWords = reservedBlock[1]?.trim().split(/\s+/) ?? []
  for (const word of reservedWords) {
    if (!previewGrammar.includes(word)) {
      errors.push(
        `extensions/seseragi-spec-preview: reserved word ${word} is missing`
      )
    }
  }
}
for (const word of [
  "constructor",
  "method",
  "property",
  "value",
  "pure",
  "task",
  "namespace",
  "since",
  "self",
]) {
  if (!previewGrammar.includes(word)) {
    errors.push(
      `extensions/seseragi-spec-preview: contextual word ${word} is missing`
    )
  }
}

const requiredTokenScopes = new Map<string, string[]>([
  ["keyword.operator.map.seseragi", ["<$>"]],
  ["keyword.operator.apply.seseragi", ["<*>"]],
  ["keyword.operator.spread.seseragi", ["..."]],
  ["keyword.operator.range.seseragi", ["..", "..="]],
  ["keyword.operator.generator.seseragi", ["<-"]],
  ["keyword.operator.arrow.seseragi", ["->"]],
  ["keyword.operator.logical.seseragi", ["&&", "||"]],
  ["keyword.operator.requirement-merge.seseragi", ["&"]],
  ["keyword.operator.comparison.seseragi", ["==", "!=", "<", "<=", ">", ">="]],
  ["keyword.operator.bind.seseragi", [">>="]],
  ["keyword.operator.pipeline.seseragi", ["|>"]],
  ["keyword.operator.application.seseragi", ["$"]],
  ["keyword.operator.signal.seseragi", [":="]],
  ["keyword.operator.arithmetic.seseragi", ["+", "-", "*", "/", "%", "**"]],
  ["keyword.operator.cons.seseragi", [":"]],
  ["keyword.operator.nullish-coalescing.seseragi", ["??"]],
  ["keyword.operator.lambda.seseragi", ["\\"]],
  ["punctuation.separator.variant.seseragi", ["|"]],
  ["punctuation.accessor.seseragi", ["."]],
  ["punctuation.terminator.seseragi", [";"]],
  ["keyword.operator.custom.seseragi", ["<+>"]],
])
for (const [scope, tokens] of requiredTokenScopes) {
  const rule = previewRules.find((candidate) => candidate.name === scope)
  if (!rule?.match) {
    errors.push(
      `extensions/seseragi-spec-preview: missing token scope ${scope}`
    )
    continue
  }
  const regex = new RegExp(rule.match)
  for (const token of tokens) {
    const match = regex.exec(token)
    if (!match || match.index !== 0 || match[0] !== token) {
      errors.push(
        `extensions/seseragi-spec-preview: scope ${scope} does not exactly match ${JSON.stringify(token)}`
      )
    }
  }
}

const appendixGrammar = readFileSync(join(specDir, "grammar.md"), "utf8")
const ebnfBlock = appendixGrammar.match(/```ebnf\n([\s\S]*?)```/)
if (!ebnfBlock) {
  errors.push("docs/spec/grammar.md: Appendix EBNF block is missing")
} else {
  const productionBodies = new Map<string, string>()
  let currentProduction: { name: string; body: string } | undefined
  for (const line of ebnfBlock[1]?.split("\n") ?? []) {
    const start = line.match(/^([a-z][a-z-]*)\s*=\s*(.*)$/)
    if (start) {
      if (currentProduction) {
        errors.push(
          `docs/spec/grammar.md: production ${currentProduction.name} is missing its terminator`
        )
      }
      currentProduction = { name: start[1] ?? "", body: start[2] ?? "" }
    } else if (currentProduction) {
      currentProduction.body += `\n${line}`
    }
    if (currentProduction && /;\s*$/.test(line)) {
      if (productionBodies.has(currentProduction.name)) {
        errors.push(
          `docs/spec/grammar.md: duplicate production ${currentProduction.name}`
        )
      }
      productionBodies.set(currentProduction.name, currentProduction.body)
      currentProduction = undefined
    }
  }
  if (currentProduction) {
    errors.push(
      `docs/spec/grammar.md: production ${currentProduction.name} is missing its terminator`
    )
  }
  if (!productionBodies.has("module")) {
    errors.push("docs/spec/grammar.md: start production module is missing")
  }

  const productionEdges = new Map<string, Set<string>>()
  for (const [name, body] of productionBodies) {
    const withoutTerminals = body.replace(/"(?:\\.|[^"\\])*"/g, "")
    const references = new Set(
      [...withoutTerminals.matchAll(/\b([a-z][a-z-]*)\b/g)].map(
        (match) => match[1] ?? ""
      )
    )
    productionEdges.set(name, references)
    for (const reference of references) {
      if (!productionBodies.has(reference)) {
        errors.push(
          `docs/spec/grammar.md: production ${name} references undefined ${reference}`
        )
      }
    }
  }

  const reachableProductions = new Set<string>()
  const pendingProductions = ["module"]
  while (pendingProductions.length > 0) {
    const name = pendingProductions.pop()
    if (!name || reachableProductions.has(name)) continue
    reachableProductions.add(name)
    for (const reference of productionEdges.get(name) ?? []) {
      if (!reachableProductions.has(reference))
        pendingProductions.push(reference)
    }
  }
  for (const name of productionBodies.keys()) {
    if (!reachableProductions.has(name)) {
      errors.push(
        `docs/spec/grammar.md: production ${name} is unreachable from module`
      )
    }
  }

  type GrammarCoverageGroup = {
    id: string
    productions: string[]
    parseTargets: string[]
    diagnosticTargets: string[]
    formatterTargets: string[]
  }
  type GrammarCoverage = {
    schema: number
    groups: GrammarCoverageGroup[]
  }
  const grammarCoveragePath = join(root, "grammar-coverage.json")
  let grammarCoverage: GrammarCoverage | undefined
  try {
    grammarCoverage = JSON.parse(
      readFileSync(grammarCoveragePath, "utf8")
    ) as GrammarCoverage
  } catch (error) {
    errors.push(`examples/spec/grammar-coverage.json: invalid JSON: ${error}`)
  }
  if (grammarCoverage) {
    if (
      grammarCoverage.schema !== 1 ||
      !Array.isArray(grammarCoverage.groups)
    ) {
      errors.push(
        "examples/spec/grammar-coverage.json: schema must be 1 with groups"
      )
    } else {
      const coveredProductions = new Map<string, string>()
      const groupIds = new Set<string>()
      for (const group of grammarCoverage.groups) {
        if (
          !group ||
          typeof group !== "object" ||
          typeof group.id !== "string" ||
          !/^[a-z][a-z0-9-]*$/.test(group.id) ||
          groupIds.has(group.id)
        ) {
          errors.push(
            "examples/spec/grammar-coverage.json: group ids must be unique kebab-case"
          )
          continue
        }
        groupIds.add(group.id)
        if (
          !Array.isArray(group.productions) ||
          group.productions.length === 0 ||
          !Array.isArray(group.parseTargets) ||
          group.parseTargets.length === 0 ||
          !Array.isArray(group.diagnosticTargets) ||
          !Array.isArray(group.formatterTargets)
        ) {
          errors.push(
            `examples/spec/grammar-coverage.json: invalid group ${group.id}`
          )
          continue
        }
        for (const production of group.productions) {
          if (!productionBodies.has(production)) {
            errors.push(
              `examples/spec/grammar-coverage.json: unknown production ${production}`
            )
          } else if (coveredProductions.has(production)) {
            errors.push(
              `examples/spec/grammar-coverage.json: production ${production} appears in both ${coveredProductions.get(production)} and ${group.id}`
            )
          } else {
            coveredProductions.set(production, group.id)
          }
        }
        for (const [kind, targets] of [
          ["parse", group.parseTargets],
          ["diagnostic", group.diagnosticTargets],
          ["formatter", group.formatterTargets],
        ] as const) {
          for (const target of targets) {
            const targetPath = resolve(root, target)
            if (
              typeof target !== "string" ||
              target.startsWith("/") ||
              target.includes("\\") ||
              target.split("/").some((segment) => segment === "..") ||
              !target.endsWith(".ssrg") ||
              !targetPath.startsWith(`${root}/`) ||
              !existsSync(targetPath)
            ) {
              errors.push(
                `examples/spec/grammar-coverage.json: missing ${kind} target ${target}`
              )
            }
          }
        }
      }
      for (const production of productionBodies.keys()) {
        if (!coveredProductions.has(production)) {
          errors.push(
            `examples/spec/grammar-coverage.json: unmapped production ${production}`
          )
        }
      }
    }
  }

  const terminals = new Set(
    [...(ebnfBlock[1]?.matchAll(/"(?:\\.|[^"\\])*"/g) ?? [])].map(
      (match) => JSON.parse(match[0]) as string
    )
  )
  const corpus = collectFiles(root, ".ssrg")
    .map((path) => readFileSync(path, "utf8"))
    .join("\n")
  for (const terminal of [...terminals].sort()) {
    const present = /^[A-Za-z]+$/.test(terminal)
      ? new RegExp(`\\b${terminal}\\b`).test(corpus)
      : corpus.includes(terminal)
    if (!present) {
      errors.push(
        `docs/spec/grammar.md: terminal ${JSON.stringify(terminal)} has no examples/spec source coverage`
      )
    }
  }
}

type ArtifactToken = {
  kind: string
  start: number
  end: number
  raw: string
}
type TokenArtifact = {
  schema: number
  source: string
  positionEncoding: string
  tokens: ArtifactToken[]
}
type CstNodeArtifact = {
  kind: string
  startToken: number
  endToken: number
  children: CstNodeArtifact[]
}
type CstArtifact = {
  schema: number
  source: string
  tokens: string
  root: CstNodeArtifact
  missing: Array<{ expected: string; atToken: number; atByte: number }>
  errors: Array<{ code: string; startToken: number; endToken: number }>
}
type WireDiagnostic = {
  id: string
  code: string
  severity: string
  messageKey: string
  primary: { start: number; end: number }
  related: unknown[]
  fixes: unknown[]
}
type DiagnosticArtifact = {
  schema: number
  source: string
  positionEncoding: string
  diagnostics: WireDiagnostic[]
}
type InterfaceExportArtifact = {
  symbol: string
  namespace: string
  name: string
  visibility: string
  declaration: { start: number; end: number }
  scheme: unknown
}
type InterfaceArtifact = {
  schema: number
  module: string
  source: string
  dependencies: Array<{
    specifier: string
    module: string
    origin: { start: number; end: number }
    imports: Array<{ namespace: string; name: string; symbol: string }>
  }>
  exports: InterfaceExportArtifact[]
  operators: Array<{
    symbol: string
    spelling: string
    fixity: string
    precedence: number
    origin: { start: number; end: number }
  }>
  instances: Array<{
    trait: string
    head: unknown
    constraints: unknown[]
    origin: { start: number; end: number }
  }>
}

const validByteRange = (
  range: { start: number; end: number } | undefined,
  sourceBytes: Buffer
): boolean =>
  !!range &&
  Number.isInteger(range.start) &&
  Number.isInteger(range.end) &&
  range.start >= 0 &&
  range.end >= range.start &&
  range.end <= sourceBytes.length &&
  Buffer.from(
    sourceBytes.subarray(0, range.start).toString("utf8"),
    "utf8"
  ).equals(sourceBytes.subarray(0, range.start)) &&
  Buffer.from(
    sourceBytes.subarray(0, range.end).toString("utf8"),
    "utf8"
  ).equals(sourceBytes.subarray(0, range.end))

const validateInterfaceArtifact = (
  moduleInterface: InterfaceArtifact,
  sourceName: string,
  sourceBytes: Buffer,
  prefix: string
): void => {
  if (
    moduleInterface.schema !== 1 ||
    !/^[a-z][a-z0-9-]*(\/[a-z][a-z0-9-]*)*$/.test(moduleInterface.module) ||
    moduleInterface.source !== sourceName ||
    !Array.isArray(moduleInterface.dependencies) ||
    !Array.isArray(moduleInterface.exports) ||
    !Array.isArray(moduleInterface.operators) ||
    !Array.isArray(moduleInterface.instances)
  ) {
    errors.push(`${prefix}/interface.json: invalid interface envelope`)
    return
  }

  const symbols = new Set<string>()
  for (const exported of moduleInterface.exports) {
    if (
      !exported ||
      typeof exported.symbol !== "string" ||
      !exported.symbol.startsWith(`${moduleInterface.module}::`) ||
      symbols.has(exported.symbol) ||
      !["value", "type", "operator"].includes(exported.namespace) ||
      typeof exported.name !== "string" ||
      exported.name.length === 0 ||
      exported.visibility !== "public" ||
      !validByteRange(exported.declaration, sourceBytes) ||
      !exported.scheme ||
      typeof exported.scheme !== "object"
    ) {
      errors.push(`${prefix}/interface.json: invalid export`)
    }
    symbols.add(exported.symbol)
  }

  for (const dependency of moduleInterface.dependencies) {
    if (
      !dependency ||
      typeof dependency.specifier !== "string" ||
      dependency.specifier.length === 0 ||
      !/^[a-z][a-z0-9-]*(\/[a-z][a-z0-9-]*)*$/.test(dependency.module) ||
      !validByteRange(dependency.origin, sourceBytes) ||
      !Array.isArray(dependency.imports) ||
      dependency.imports.some(
        (imported) =>
          !imported ||
          !["value", "type", "operator"].includes(imported.namespace) ||
          typeof imported.name !== "string" ||
          imported.name.length === 0 ||
          typeof imported.symbol !== "string" ||
          !imported.symbol.startsWith(`${dependency.module}::`)
      )
    ) {
      errors.push(`${prefix}/interface.json: invalid dependency`)
    }
  }

  for (const operator of moduleInterface.operators) {
    if (
      !operator ||
      !symbols.has(operator.symbol) ||
      !/^[!$%&*+\-./:<=>?@^|~]{2,}$/.test(operator.spelling) ||
      !["infixl", "infixr", "infix"].includes(operator.fixity) ||
      !Number.isInteger(operator.precedence) ||
      operator.precedence < 0 ||
      operator.precedence > 8 ||
      !validByteRange(operator.origin, sourceBytes)
    ) {
      errors.push(`${prefix}/interface.json: invalid operator`)
    }
  }

  for (const instance of moduleInterface.instances) {
    if (
      !instance ||
      !/^[A-Z][A-Za-z0-9']*$/.test(instance.trait) ||
      !instance.head ||
      typeof instance.head !== "object" ||
      !Array.isArray(instance.constraints) ||
      !validByteRange(instance.origin, sourceBytes)
    ) {
      errors.push(`${prefix}/interface.json: invalid instance`)
    }
  }
}

const artifactsDir = join(root, "artifacts", "schema-1")
for (const entry of readdirSync(artifactsDir, { withFileTypes: true })) {
  if (!entry.isDirectory()) continue
  const directory = join(artifactsDir, entry.name)
  const prefix = `artifacts/schema-1/${entry.name}`
  const readArtifact = <T>(name: string): T | undefined => {
    const path = join(directory, name)
    try {
      return JSON.parse(readFileSync(path, "utf8")) as T
    } catch (error) {
      errors.push(`${prefix}/${name}: invalid JSON: ${error}`)
      return undefined
    }
  }

  const tokens = readArtifact<TokenArtifact>("tokens.json")
  if (!tokens) continue
  const sourcePath = resolve(directory, tokens.source)
  if (
    tokens.schema !== 1 ||
    tokens.positionEncoding !== "utf-8" ||
    tokens.source.startsWith("/") ||
    tokens.source.includes("\\") ||
    tokens.source.split("/").some((segment) => segment === "..") ||
    !sourcePath.startsWith(`${directory}/`) ||
    !existsSync(sourcePath) ||
    !Array.isArray(tokens.tokens) ||
    tokens.tokens.length === 0
  ) {
    errors.push(`${prefix}/tokens.json: invalid token artifact envelope`)
    continue
  }

  const sourceBytes = readFileSync(sourcePath)
  let previousEnd = 0
  let reconstructed = ""
  for (const [index, token] of tokens.tokens.entries()) {
    if (
      !token ||
      typeof token.kind !== "string" ||
      !/^[a-z][a-z0-9.-]*$/.test(token.kind) ||
      typeof token.raw !== "string" ||
      !Number.isInteger(token.start) ||
      !Number.isInteger(token.end) ||
      token.start !== previousEnd ||
      token.end < token.start ||
      token.end > sourceBytes.length
    ) {
      errors.push(`${prefix}/tokens.json: invalid token ${index}`)
      continue
    }
    const actual = sourceBytes.subarray(token.start, token.end).toString("utf8")
    if (actual !== token.raw) {
      errors.push(`${prefix}/tokens.json: raw mismatch at token ${index}`)
    }
    const startsOnBoundary = Buffer.from(
      sourceBytes.subarray(0, token.start).toString("utf8"),
      "utf8"
    ).equals(sourceBytes.subarray(0, token.start))
    const endsOnBoundary = Buffer.from(
      sourceBytes.subarray(0, token.end).toString("utf8"),
      "utf8"
    ).equals(sourceBytes.subarray(0, token.end))
    if (!startsOnBoundary || !endsOnBoundary) {
      errors.push(`${prefix}/tokens.json: token ${index} splits UTF-8`)
    }
    if (token.kind !== "eof") reconstructed += token.raw
    previousEnd = token.end
  }
  const lastToken = tokens.tokens.at(-1)
  if (
    !lastToken ||
    lastToken.kind !== "eof" ||
    lastToken.start !== sourceBytes.length ||
    lastToken.end !== sourceBytes.length ||
    lastToken.raw !== "" ||
    reconstructed !== sourceBytes.toString("utf8")
  ) {
    errors.push(`${prefix}/tokens.json: token stream is not lossless with EOF`)
  }

  const cst = readArtifact<CstArtifact>("cst.json")
  if (cst) {
    const nonEofTokenCount = tokens.tokens.length - 1
    const validateNode = (
      node: CstNodeArtifact,
      parentStart: number,
      parentEnd: number
    ): void => {
      if (
        !node ||
        typeof node.kind !== "string" ||
        !/^[a-z][a-z0-9-]*$/.test(node.kind) ||
        !Number.isInteger(node.startToken) ||
        !Number.isInteger(node.endToken) ||
        node.startToken < parentStart ||
        node.endToken > parentEnd ||
        node.endToken < node.startToken ||
        !Array.isArray(node.children)
      ) {
        errors.push(`${prefix}/cst.json: invalid CST node`)
        return
      }
      for (const child of node.children) {
        validateNode(child, node.startToken, node.endToken)
      }
    }
    if (
      cst.schema !== 1 ||
      cst.source !== tokens.source ||
      cst.tokens !== "tokens.json" ||
      !Array.isArray(cst.missing) ||
      !Array.isArray(cst.errors) ||
      cst.root?.kind !== "module" ||
      cst.root?.startToken !== 0 ||
      cst.root?.endToken !== nonEofTokenCount
    ) {
      errors.push(`${prefix}/cst.json: invalid CST artifact envelope`)
    } else {
      validateNode(cst.root, 0, nonEofTokenCount)
      for (const missing of cst.missing) {
        if (
          !missing ||
          typeof missing.expected !== "string" ||
          missing.expected.length === 0 ||
          !Number.isInteger(missing.atToken) ||
          missing.atToken < 0 ||
          missing.atToken > nonEofTokenCount ||
          !Number.isInteger(missing.atByte) ||
          missing.atByte !==
            (tokens.tokens[missing.atToken]?.start ?? sourceBytes.length)
        ) {
          errors.push(`${prefix}/cst.json: invalid missing token`)
        }
      }
      for (const error of cst.errors) {
        if (
          !error ||
          !diagnosticRegistry.has(error.code) ||
          !Number.isInteger(error.startToken) ||
          !Number.isInteger(error.endToken) ||
          error.startToken < 0 ||
          error.endToken < error.startToken ||
          error.endToken > nonEofTokenCount
        ) {
          errors.push(`${prefix}/cst.json: invalid error range`)
        }
      }
    }
  }

  const diagnostics = readArtifact<DiagnosticArtifact>("diagnostics.json")
  if (
    diagnostics &&
    (diagnostics.schema !== 1 ||
      diagnostics.source !== tokens.source ||
      diagnostics.positionEncoding !== "utf-8" ||
      !Array.isArray(diagnostics.diagnostics))
  ) {
    errors.push(`${prefix}/diagnostics.json: invalid diagnostic envelope`)
  } else if (diagnostics) {
    const ids = new Set<string>()
    for (const diagnostic of diagnostics.diagnostics) {
      const range = diagnostic.primary
      if (
        !diagnostic ||
        !/^d[1-9][0-9]*$/.test(diagnostic.id) ||
        ids.has(diagnostic.id) ||
        !diagnosticRegistry.has(diagnostic.code) ||
        diagnosticRegistry.get(diagnostic.code) !== diagnostic.severity ||
        !/^[a-z][a-z0-9-]*(\.[a-z][a-z0-9-]*)+$/.test(diagnostic.messageKey) ||
        !range ||
        !Number.isInteger(range.start) ||
        !Number.isInteger(range.end) ||
        range.start < 0 ||
        range.end < range.start ||
        range.end > sourceBytes.length ||
        !Array.isArray(diagnostic.related) ||
        !Array.isArray(diagnostic.fixes)
      ) {
        errors.push(`${prefix}/diagnostics.json: invalid diagnostic`)
      }
      ids.add(diagnostic.id)
    }
  }

  const moduleInterface = readArtifact<InterfaceArtifact>("interface.json")
  if (moduleInterface) {
    validateInterfaceArtifact(
      moduleInterface,
      tokens.source,
      sourceBytes,
      prefix
    )
  }
}

const interfaceArtifactsDir = join(root, "artifacts", "interface-schema-1")
for (const entry of readdirSync(interfaceArtifactsDir, {
  withFileTypes: true,
})) {
  if (!entry.isDirectory()) continue
  const directory = join(interfaceArtifactsDir, entry.name)
  const prefix = `artifacts/interface-schema-1/${entry.name}`
  let moduleInterface: InterfaceArtifact
  try {
    moduleInterface = JSON.parse(
      readFileSync(join(directory, "interface.json"), "utf8")
    ) as InterfaceArtifact
  } catch (error) {
    errors.push(`${prefix}/interface.json: invalid JSON: ${error}`)
    continue
  }
  const sourcePath = resolve(directory, moduleInterface.source)
  if (
    moduleInterface.source.startsWith("/") ||
    moduleInterface.source.includes("\\") ||
    moduleInterface.source.split("/").some((segment) => segment === "..") ||
    !sourcePath.startsWith(`${directory}/`) ||
    !existsSync(sourcePath)
  ) {
    errors.push(`${prefix}/interface.json: missing source`)
    continue
  }
  validateInterfaceArtifact(
    moduleInterface,
    moduleInterface.source,
    readFileSync(sourcePath),
    prefix
  )
  for (const dependency of moduleInterface.dependencies ?? []) {
    if (dependency.specifier.startsWith("./")) {
      const dependencyPath = resolve(
        directory,
        `${dependency.specifier.slice(2)}.ssrg`
      )
      if (
        !dependencyPath.startsWith(`${directory}/`) ||
        !existsSync(dependencyPath)
      ) {
        errors.push(
          `${prefix}/interface.json: missing dependency source ${dependency.specifier}`
        )
      }
    }
  }
}

const lessons = readdirSync(lessonsDir)
  .filter((name) => /^\d{2}-.*\.ssrg$/.test(name))
  .sort()

lessons.forEach((name, index) => {
  const expectedNumber = String(index + 1).padStart(2, "0")
  const actualNumber = name.slice(0, 2)
  const path = join(lessonsDir, name)
  const source = readFileSync(path, "utf8")
  const lines = source.split("\n")

  if (actualNumber !== expectedNumber) {
    errors.push(`${name}: expected lesson number ${expectedNumber}`)
  }
  if (!lines[0]?.startsWith(`// Lesson ${actualNumber}:`)) {
    errors.push(`${name}: first line must declare its lesson number and goal`)
  }
  if (!lines[1]?.startsWith("// 前提:")) {
    errors.push(`${name}: second line must declare prerequisites in Japanese`)
  }

  const prerequisites = [...(lines[1]?.matchAll(/\b(\d{2})\b/g) ?? [])].map(
    (match) => Number(match[1])
  )
  if (!lines[1]?.includes("なし") && prerequisites.length === 0) {
    errors.push(`${name}: prerequisites must name an earlier lesson or なし`)
  }
  if (prerequisites.some((lesson) => lesson >= Number(actualNumber))) {
    errors.push(`${name}: prerequisites must refer only to earlier lessons`)
  }

  const hasJapaneseExplanation = lines
    .slice(2)
    .some((line) => /^\s*\/\/ .*[ぁ-んァ-ヶ一-龠]/.test(line))
  if (!hasJapaneseExplanation) {
    errors.push(`${name}: missing a Japanese learning comment`)
  }

  const targetMarker = lines.find((line) => line.startsWith("// Test target:"))
  const servicesMarker = lines.find((line) =>
    line.startsWith("// Test services:")
  )
  if (servicesMarker && !targetMarker) {
    errors.push(`${name}: Test services requires Test target`)
  }
  if (
    targetMarker &&
    targetMarker.slice("// Test target:".length).trim() === ""
  ) {
    errors.push(`${name}: Test target must not be empty`)
  }
  if (servicesMarker) {
    const relativePath = servicesMarker.slice("// Test services:".length).trim()
    const servicesPath = resolve(dirname(path), relativePath)
    if (
      relativePath === "" ||
      relativePath.startsWith("/") ||
      relativePath.includes("\\") ||
      relativePath.split("/").some((segment) => segment === "..") ||
      !servicesPath.startsWith(`${lessonsDir}/`) ||
      !existsSync(servicesPath)
    ) {
      errors.push(`${name}: missing Test services ${relativePath}`)
    } else {
      try {
        const parsed: unknown = JSON.parse(readFileSync(servicesPath, "utf8"))
        if (
          !parsed ||
          typeof parsed !== "object" ||
          Array.isArray(parsed) ||
          (parsed as { schema?: unknown }).schema !== 1
        ) {
          errors.push(`${name}: Test services schema must be 1`)
        }
      } catch (error) {
        errors.push(`${name}: invalid Test services JSON: ${error}`)
      }
    }
  }

  const marker = lines.findIndex((line) =>
    line.startsWith("// Expected stdout:")
  )
  if (marker < 0) {
    errors.push(`${name}: missing Expected stdout marker`)
    return
  }

  const snapshot = lines[marker]?.slice("// Expected stdout:".length).trim()
  if (snapshot) {
    const snapshotPath = resolve(dirname(path), snapshot)
    if (!existsSync(snapshotPath)) {
      errors.push(`${name}: missing stdout snapshot ${snapshot}`)
    }
  } else if (!lines.slice(marker + 1).some((line) => line.startsWith("// "))) {
    errors.push(`${name}: inline Expected stdout is empty`)
  }
})

const lessonReadme = readFileSync(join(lessonsDir, "README.md"), "utf8")
for (const name of lessons) {
  const number = basename(name).slice(0, 2)
  if (!new RegExp(`\\|\\s*${number}\\s*\\|`).test(lessonReadme)) {
    errors.push(`${name}: missing from lessons/README.md learning path`)
  }
}

type ExpectedRange = {
  start: number
  end: number
  text: string
}

type ExpectedDiagnostic = {
  code: string
  severity: string
  primary: ExpectedRange
}

type FixtureExpectation = {
  schema: number
  kind: string
  spec: string[]
  diagnostics?: ExpectedDiagnostic[]
}

type ProjectExpectation = {
  schema: number
  kind: string
  phase: string
  spec: string[]
  lock: string
  stdin?: string
  stdout?: string
  diagnostics?: ProjectDiagnostic[]
  command?: string
  artifacts?: ProjectArtifact[]
  services?: string
  stderr?: string
  exitCode?: number
  args?: string[]
  shapes?: ProjectShape[]
  differentialProfiles?: string[]
}

type ProjectDiagnostic = ExpectedDiagnostic & {
  file: string
}

type ProjectArtifact = {
  output: string
  snapshot: string
}

type ProjectShape = {
  symbol: string
  require: string[]
}

const shapePredicates = new Set([
  "newtype-erased",
  "surface-sugar-erased",
  "self-tail-loop",
])

let fixtureCount = 0
for (const fixtureKind of ["compile", "diagnostics"] as const) {
  const directory = join(fixturesDir, fixtureKind)
  const sources = readdirSync(directory)
    .filter((name) => name.endsWith(".ssrg"))
    .sort()
  const sidecars = new Set(
    readdirSync(directory).filter((name) => name.endsWith(".expect.json"))
  )

  for (const name of sources) {
    fixtureCount += 1
    const sourcePath = join(directory, name)
    const source = readFileSync(sourcePath, "utf8")
    const expectedKind = fixtureKind === "compile" ? "compile" : "diagnostic"
    const sidecarName = name.replace(/\.ssrg$/, ".expect.json")
    const sidecarPath = join(directory, sidecarName)

    if (!existsSync(sidecarPath)) {
      errors.push(`${fixtureKind}/${name}: missing ${sidecarName}`)
      continue
    }

    if (source.includes("\r")) {
      errors.push(`${fixtureKind}/${name}: fixture source must use LF`)
    }
    if (!source.endsWith("\n")) {
      errors.push(`${fixtureKind}/${name}: fixture source needs final newline`)
    }
    sidecars.delete(sidecarName)

    let expectation: FixtureExpectation
    try {
      const parsed: unknown = JSON.parse(readFileSync(sidecarPath, "utf8"))
      if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
        errors.push(`${fixtureKind}/${sidecarName}: root must be an object`)
        continue
      }
      expectation = parsed as FixtureExpectation
    } catch (error) {
      errors.push(`${fixtureKind}/${sidecarName}: invalid JSON: ${error}`)
      continue
    }

    if (expectation.schema !== 1) {
      errors.push(`${fixtureKind}/${sidecarName}: schema must be 1`)
    }
    if (expectation.kind !== expectedKind) {
      errors.push(`${fixtureKind}/${sidecarName}: kind must be ${expectedKind}`)
    }
    if (
      !Array.isArray(expectation.spec) ||
      expectation.spec.length === 0 ||
      expectation.spec.some((section) => !/^\d+\.\d+$/.test(section))
    ) {
      errors.push(
        `${fixtureKind}/${sidecarName}: spec must contain section numbers`
      )
    } else {
      for (const section of expectation.spec) {
        if (!specSections.has(section)) {
          errors.push(
            `${fixtureKind}/${sidecarName}: unknown spec section ${section}`
          )
        }
      }
    }

    if (expectedKind === "compile") {
      if (expectation.diagnostics !== undefined) {
        errors.push(
          `${fixtureKind}/${sidecarName}: compile fixture cannot declare diagnostics`
        )
      }
      continue
    }

    if (
      !Array.isArray(expectation.diagnostics) ||
      expectation.diagnostics.length === 0
    ) {
      errors.push(
        `${fixtureKind}/${sidecarName}: diagnostic fixture needs diagnostics`
      )
      continue
    }

    const bytes = Buffer.from(source, "utf8")
    for (const diagnostic of expectation.diagnostics) {
      if (
        !diagnostic ||
        typeof diagnostic !== "object" ||
        Array.isArray(diagnostic)
      ) {
        errors.push(
          `${fixtureKind}/${sidecarName}: diagnostic must be an object`
        )
        continue
      }
      if (!/^SES-[PNTIEKFL]\d{4}$/.test(diagnostic.code)) {
        errors.push(
          `${fixtureKind}/${sidecarName}: invalid code ${diagnostic.code}`
        )
      } else if (!diagnosticRegistry.has(diagnostic.code)) {
        errors.push(
          `${fixtureKind}/${sidecarName}: unregistered code ${diagnostic.code}`
        )
      }
      if (
        !["Error", "Warning", "Information", "Hint"].includes(
          diagnostic.severity
        )
      ) {
        errors.push(
          `${fixtureKind}/${sidecarName}: invalid severity ${diagnostic.severity}`
        )
      } else if (
        diagnosticRegistry.get(diagnostic.code) !== diagnostic.severity
      ) {
        errors.push(
          `${fixtureKind}/${sidecarName}: severity for ${diagnostic.code} must match registry ${diagnosticRegistry.get(diagnostic.code)}`
        )
      }

      const range = diagnostic.primary
      if (
        !range ||
        typeof range !== "object" ||
        typeof range.text !== "string" ||
        !Number.isInteger(range.start) ||
        !Number.isInteger(range.end) ||
        range.start < 0 ||
        range.end < range.start ||
        range.end > bytes.length
      ) {
        errors.push(`${fixtureKind}/${sidecarName}: invalid primary byte range`)
        continue
      }

      const actual = bytes.subarray(range.start, range.end).toString("utf8")
      if (actual !== range.text) {
        errors.push(
          `${fixtureKind}/${sidecarName}: primary text expected ${JSON.stringify(range.text)}, got ${JSON.stringify(actual)}`
        )
      }
      const startsOnBoundary = Buffer.from(
        bytes.subarray(0, range.start).toString("utf8"),
        "utf8"
      ).equals(bytes.subarray(0, range.start))
      const endsOnBoundary = Buffer.from(
        bytes.subarray(0, range.end).toString("utf8"),
        "utf8"
      ).equals(bytes.subarray(0, range.end))
      if (!startsOnBoundary || !endsOnBoundary) {
        errors.push(
          `${fixtureKind}/${sidecarName}: primary range is not on UTF-8 boundaries`
        )
      }
    }
  }

  for (const orphan of [...sidecars].sort()) {
    errors.push(`${fixtureKind}/${orphan}: no matching .ssrg source`)
  }
}

const projectsDir = join(fixturesDir, "projects")
const projects = readdirSync(projectsDir, { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .sort()

for (const name of projects) {
  const directory = join(projectsDir, name)
  const manifestPath = join(directory, "seseragi.toml")
  const expectationPath = join(directory, "project.expect.json")

  if (!existsSync(manifestPath)) {
    errors.push(`projects/${name}: missing seseragi.toml`)
  } else {
    const manifest = readFileSync(manifestPath, "utf8")
    let inForeignTypescript = false
    for (const line of manifest.split("\n")) {
      if (line === "[foreign.typescript]") {
        inForeignTypescript = true
        continue
      }
      if (line.startsWith("[")) {
        inForeignTypescript = false
      }
      if (!inForeignTypescript) {
        continue
      }
      const reference = line.match(
        /^(manifest|lockfile|bindings)\s*=\s*"([^"]+)"\s*$/
      )
      if (!reference) {
        continue
      }
      const relativePath = reference[2] ?? ""
      const targetPath = resolve(directory, relativePath)
      if (
        relativePath.startsWith("/") ||
        relativePath.includes("\\") ||
        relativePath.split("/").some((segment) => segment === "..") ||
        !targetPath.startsWith(`${directory}/`) ||
        !existsSync(targetPath)
      ) {
        errors.push(
          `projects/${name}: missing foreign.typescript ${reference[1]} ${relativePath}`
        )
      }
    }
  }
  if (!existsSync(expectationPath)) {
    errors.push(`projects/${name}: missing project.expect.json`)
    continue
  }

  let expectation: ProjectExpectation
  try {
    const parsed: unknown = JSON.parse(readFileSync(expectationPath, "utf8"))
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      errors.push(`projects/${name}: expectation root must be an object`)
      continue
    }
    expectation = parsed as ProjectExpectation
  } catch (error) {
    errors.push(`projects/${name}: invalid project.expect.json: ${error}`)
    continue
  }

  if (expectation.schema !== 1) {
    errors.push(`projects/${name}: schema must be 1`)
  }
  if (expectation.kind !== "project") {
    errors.push(`projects/${name}: kind must be project`)
  }
  if (
    !["compile", "diagnostic", "run", "test", "convert", "tooling"].includes(
      expectation.phase
    )
  ) {
    errors.push(`projects/${name}: invalid phase ${expectation.phase}`)
  }
  if (
    !Array.isArray(expectation.spec) ||
    expectation.spec.length === 0 ||
    expectation.spec.some((section) => !specSections.has(section))
  ) {
    errors.push(`projects/${name}: spec must reference known sections`)
  }
  if (!["generate", "fixture"].includes(expectation.lock)) {
    errors.push(`projects/${name}: lock must be generate or fixture`)
  } else if (
    expectation.lock === "fixture" &&
    !existsSync(join(directory, "seseragi.lock"))
  ) {
    errors.push(`projects/${name}: fixture lock requires seseragi.lock`)
  }

  if (expectation.phase === "diagnostic") {
    if (
      expectation.command !== undefined &&
      !["compile", "convert", "tooling"].includes(expectation.command)
    ) {
      errors.push(`projects/${name}: invalid diagnostic command`)
    }
    if (
      !Array.isArray(expectation.diagnostics) ||
      expectation.diagnostics.length === 0
    ) {
      errors.push(`projects/${name}: diagnostic phase needs diagnostics`)
    } else {
      for (const diagnostic of expectation.diagnostics) {
        if (
          !diagnostic ||
          typeof diagnostic !== "object" ||
          Array.isArray(diagnostic)
        ) {
          errors.push(`projects/${name}: diagnostic must be an object`)
          continue
        }
        if (
          typeof diagnostic.file !== "string" ||
          diagnostic.file.length === 0 ||
          diagnostic.file.startsWith("/") ||
          diagnostic.file.includes("\\") ||
          diagnostic.file.split("/").some((segment) => segment === "..")
        ) {
          errors.push(`projects/${name}: invalid diagnostic file`)
          continue
        }

        const sourcePath = resolve(directory, diagnostic.file)
        if (
          !sourcePath.startsWith(`${directory}/`) ||
          !existsSync(sourcePath)
        ) {
          errors.push(
            `projects/${name}: missing diagnostic file ${diagnostic.file}`
          )
          continue
        }
        if (!/^SES-[PNTIEKFL]\d{4}$/.test(diagnostic.code)) {
          errors.push(
            `projects/${name}: invalid diagnostic code ${diagnostic.code}`
          )
        } else if (!diagnosticRegistry.has(diagnostic.code)) {
          errors.push(
            `projects/${name}: unregistered diagnostic code ${diagnostic.code}`
          )
        }
        if (diagnosticRegistry.get(diagnostic.code) !== diagnostic.severity) {
          errors.push(
            `projects/${name}: severity for ${diagnostic.code} must match registry ${diagnosticRegistry.get(diagnostic.code)}`
          )
        }

        const source = readFileSync(sourcePath)
        const range = diagnostic.primary
        if (
          !range ||
          typeof range !== "object" ||
          typeof range.text !== "string" ||
          !Number.isInteger(range.start) ||
          !Number.isInteger(range.end) ||
          range.start < 0 ||
          range.end < range.start ||
          range.end > source.length
        ) {
          errors.push(`projects/${name}: invalid diagnostic primary range`)
          continue
        }
        const actual = source.subarray(range.start, range.end).toString("utf8")
        if (actual !== range.text) {
          errors.push(
            `projects/${name}: primary text expected ${JSON.stringify(range.text)}, got ${JSON.stringify(actual)}`
          )
        }
        const startsOnBoundary = Buffer.from(
          source.subarray(0, range.start).toString("utf8"),
          "utf8"
        ).equals(source.subarray(0, range.start))
        const endsOnBoundary = Buffer.from(
          source.subarray(0, range.end).toString("utf8"),
          "utf8"
        ).equals(source.subarray(0, range.end))
        if (!startsOnBoundary || !endsOnBoundary) {
          errors.push(
            `projects/${name}: diagnostic range is not on UTF-8 boundaries`
          )
        }
      }
    }
  } else if (expectation.diagnostics !== undefined) {
    errors.push(`projects/${name}: only diagnostic phase accepts diagnostics`)
  } else if (
    expectation.command !== undefined &&
    !(expectation.phase === "tooling" && ["doc"].includes(expectation.command))
  ) {
    errors.push(`projects/${name}: invalid command for ${expectation.phase}`)
  }

  if (expectation.artifacts !== undefined) {
    if (
      !["convert", "tooling"].includes(expectation.phase) ||
      !Array.isArray(expectation.artifacts) ||
      expectation.artifacts.length === 0
    ) {
      errors.push(
        `projects/${name}: artifacts require convert/tooling phase and a non-empty array`
      )
    } else {
      const outputs = new Set<string>()
      for (const artifact of expectation.artifacts) {
        if (
          !artifact ||
          typeof artifact !== "object" ||
          Array.isArray(artifact)
        ) {
          errors.push(`projects/${name}: artifact must be an object`)
          continue
        }
        let validPaths = true
        for (const field of [artifact.output, artifact.snapshot]) {
          if (
            typeof field !== "string" ||
            field.length === 0 ||
            field.startsWith("/") ||
            field.includes("\\") ||
            field.split("/").some((segment) => segment === "..")
          ) {
            errors.push(`projects/${name}: invalid artifact path`)
            validPaths = false
          }
        }
        if (!validPaths) {
          continue
        }
        if (outputs.has(artifact.output)) {
          errors.push(`projects/${name}: duplicate artifact output`)
        }
        outputs.add(artifact.output)

        const snapshotPath = resolve(directory, artifact.snapshot)
        if (
          !snapshotPath.startsWith(`${directory}/`) ||
          !existsSync(snapshotPath)
        ) {
          errors.push(
            `projects/${name}: missing artifact snapshot ${artifact.snapshot}`
          )
          continue
        }
        const snapshot = readFileSync(snapshotPath, "utf8")
        if (snapshot.includes("\r") || !snapshot.endsWith("\n")) {
          errors.push(
            `projects/${name}: artifact snapshot must use LF and final newline`
          )
        }
      }
    }
  }

  if (expectation.stdout !== undefined) {
    const stdoutPath = resolve(directory, expectation.stdout)
    if (!existsSync(stdoutPath)) {
      errors.push(`projects/${name}: missing stdout ${expectation.stdout}`)
    } else {
      const stdout = readFileSync(stdoutPath, "utf8")
      if (stdout.includes("\r")) {
        errors.push(`projects/${name}: stdout snapshot must use LF`)
      }
      if (!stdout.endsWith("\n")) {
        errors.push(`projects/${name}: stdout snapshot needs final newline`)
      }
    }
  }

  if (expectation.stderr !== undefined) {
    const stderrPath = resolve(directory, expectation.stderr)
    if (!existsSync(stderrPath)) {
      errors.push(`projects/${name}: missing stderr ${expectation.stderr}`)
    } else {
      const stderr = readFileSync(stderrPath, "utf8")
      if (stderr.includes("\r")) {
        errors.push(`projects/${name}: stderr snapshot must use LF`)
      }
      if (!stderr.endsWith("\n")) {
        errors.push(`projects/${name}: stderr snapshot needs final newline`)
      }
    }
  }

  if (
    expectation.exitCode !== undefined &&
    (!Number.isInteger(expectation.exitCode) ||
      expectation.exitCode < 0 ||
      expectation.exitCode > 255)
  ) {
    errors.push(`projects/${name}: exitCode must be an integer from 0 to 255`)
  }

  if (
    expectation.args !== undefined &&
    (!Array.isArray(expectation.args) ||
      expectation.args.some((argument) => typeof argument !== "string"))
  ) {
    errors.push(`projects/${name}: args must be an array of strings`)
  }

  if (expectation.shapes !== undefined) {
    if (
      expectation.phase !== "compile" ||
      !Array.isArray(expectation.shapes) ||
      expectation.shapes.length === 0
    ) {
      errors.push(
        `projects/${name}: shapes require compile phase and a non-empty array`
      )
    } else {
      const symbols = new Set<string>()
      const profileIndex = expectation.args?.indexOf("--profile") ?? -1
      if (expectation.args?.[profileIndex + 1] !== "release") {
        errors.push(`projects/${name}: shapes require --profile release`)
      }
      for (const shape of expectation.shapes) {
        if (
          !shape ||
          typeof shape !== "object" ||
          Array.isArray(shape) ||
          typeof shape.symbol !== "string" ||
          !/^[a-z][A-Za-z0-9_-]*(\/[a-z][A-Za-z0-9_-]*)*::[a-z][A-Za-z0-9']*$/.test(
            shape.symbol
          )
        ) {
          errors.push(`projects/${name}: invalid shape symbol`)
          continue
        }
        if (symbols.has(shape.symbol)) {
          errors.push(
            `projects/${name}: duplicate shape symbol ${shape.symbol}`
          )
        }
        symbols.add(shape.symbol)
        if (
          !Array.isArray(shape.require) ||
          shape.require.length === 0 ||
          new Set(shape.require).size !== shape.require.length ||
          shape.require.some(
            (predicate) =>
              typeof predicate !== "string" || !shapePredicates.has(predicate)
          )
        ) {
          errors.push(
            `projects/${name}: invalid shape predicates for ${shape.symbol}`
          )
        }
      }
    }
  }

  if (expectation.differentialProfiles !== undefined) {
    if (
      !["run", "test"].includes(expectation.phase) ||
      !Array.isArray(expectation.differentialProfiles) ||
      expectation.differentialProfiles.length !== 2 ||
      expectation.differentialProfiles[0] !== "development" ||
      expectation.differentialProfiles[1] !== "release"
    ) {
      errors.push(
        `projects/${name}: differentialProfiles must be development, release for run/test phase`
      )
    }
    if (expectation.args?.includes("--profile")) {
      errors.push(
        `projects/${name}: differentialProfiles cannot combine with --profile`
      )
    }
  }

  if (expectation.stdin !== undefined) {
    const stdinPath = resolve(directory, expectation.stdin)
    if (!existsSync(stdinPath)) {
      errors.push(`projects/${name}: missing stdin ${expectation.stdin}`)
    }
  }

  if (expectation.services !== undefined) {
    const servicesPath = resolve(directory, expectation.services)
    if (
      expectation.services.startsWith("/") ||
      expectation.services.includes("\\") ||
      expectation.services.split("/").some((segment) => segment === "..") ||
      !servicesPath.startsWith(`${directory}/`) ||
      !existsSync(servicesPath)
    ) {
      errors.push(`projects/${name}: missing services ${expectation.services}`)
    } else {
      try {
        const parsed: unknown = JSON.parse(readFileSync(servicesPath, "utf8"))
        if (
          !parsed ||
          typeof parsed !== "object" ||
          Array.isArray(parsed) ||
          (parsed as { schema?: unknown }).schema !== 1
        ) {
          errors.push(`projects/${name}: services schema must be 1`)
        }
      } catch (error) {
        errors.push(`projects/${name}: invalid services JSON: ${error}`)
      }
    }
  }
}

const statusSource = readFileSync(
  resolve(import.meta.dir, "../docs/STATUS.md"),
  "utf8"
)
const statusCounts = [
  `の${lessons.length} lesson`,
  `positive ${readdirSync(join(fixturesDir, "compile")).filter((name) => name.endsWith(".ssrg")).length}件、diagnostic ${readdirSync(join(fixturesDir, "diagnostics")).filter((name) => name.endsWith(".ssrg")).length}件`,
  `project ${projects.length}件`,
]
for (const expected of statusCounts) {
  if (!statusSource.includes(expected)) {
    errors.push(`docs/STATUS.md: stale artifact count, expected ${expected}`)
  }
}

if (errors.length > 0) {
  console.error(errors.join("\n"))
  process.exit(1)
}

console.log(
  `Spec lessons: ${lessons.length} checked; fixtures: ${fixtureCount} checked; projects: ${projects.length} checked`
)
