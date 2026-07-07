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
          !Array.isArray(group.formatterTargets) ||
          group.formatterTargets.length === 0
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

const validateTokenArtifact = (
  tokens: TokenArtifact,
  directory: string,
  prefix: string
): Buffer | undefined => {
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
    return undefined
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

  return sourceBytes
}

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

type RuntimeFeature = {
  id: string
  kind: string
  typescript: string
  boundary: string
  import: { module: string; export: string } | null
}
type RuntimeAbiArtifact = {
  schema: number
  identity: string
  abiMajor: number
  targetFamily: string
  features: RuntimeFeature[]
}

const runtimeAbiPath = join(
  root,
  "artifacts",
  "runtime-schema-1",
  "core",
  "abi.json"
)
let runtimeAbi: RuntimeAbiArtifact | undefined
try {
  runtimeAbi = JSON.parse(
    readFileSync(runtimeAbiPath, "utf8")
  ) as RuntimeAbiArtifact
} catch (error) {
  errors.push(
    `artifacts/runtime-schema-1/core/abi.json: invalid JSON: ${error}`
  )
}
const runtimeFeatures = new Set<string>()
if (
  runtimeAbi &&
  (runtimeAbi.schema !== 1 ||
    runtimeAbi.identity !== "@seseragi/runtime" ||
    runtimeAbi.abiMajor !== 1 ||
    runtimeAbi.targetFamily !== "typescript" ||
    !Array.isArray(runtimeAbi.features))
) {
  errors.push("artifacts/runtime-schema-1/core/abi.json: invalid ABI envelope")
} else if (runtimeAbi) {
  for (const feature of runtimeAbi.features) {
    if (
      !feature ||
      !/^[a-z][a-z0-9-]*(\.[a-z][a-z0-9-]*)+$/.test(feature.id) ||
      runtimeFeatures.has(feature.id) ||
      !["value-representation", "runtime-helper"].includes(feature.kind) ||
      typeof feature.typescript !== "string" ||
      typeof feature.boundary !== "string" ||
      (feature.kind === "value-representation" && feature.import !== null) ||
      (feature.kind === "runtime-helper" &&
        (!feature.import ||
          !/^@seseragi\/runtime(?:\/[a-z][a-z0-9-]*)+$/.test(
            feature.import.module
          ) ||
          !/^[A-Za-z_$][A-Za-z0-9_$]*$/.test(feature.import.export)))
    ) {
      errors.push("artifacts/runtime-schema-1/core/abi.json: invalid feature")
    }
    runtimeFeatures.add(feature.id)
  }
}

const tokenArtifactsDir = join(root, "artifacts", "token-schema-1")
for (const entry of readdirSync(tokenArtifactsDir, { withFileTypes: true })) {
  if (!entry.isDirectory()) continue
  const directory = join(tokenArtifactsDir, entry.name)
  const prefix = `artifacts/token-schema-1/${entry.name}`
  try {
    const tokens = JSON.parse(
      readFileSync(join(directory, "tokens.json"), "utf8")
    ) as TokenArtifact
    validateTokenArtifact(tokens, directory, prefix)
  } catch (error) {
    errors.push(`${prefix}/tokens.json: invalid JSON: ${error}`)
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
  const sourceBytes = validateTokenArtifact(tokens, directory, prefix)
  if (!sourceBytes) continue

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

  const stageNames = [
    "surface-ast",
    "resolved-ast",
    "typed-hir",
    "core-ir",
    "typescript-ir",
  ] as const
  const hasFullLoweringStageArtifacts = stageNames
    .filter((stage) => stage === "core-ir" || stage === "typescript-ir")
    .some((stage) => existsSync(join(directory, `${stage}.json`)))
    || existsSync(join(directory, "generated-module.json"))
  const surfaceOnly = existsSync(join(directory, "surface-ast.json"))
    ? readArtifact<{
        schema?: unknown
        source?: unknown
        declarations?: unknown
      }>("surface-ast.json")
    : undefined
  if (surfaceOnly) {
    if (
      surfaceOnly.schema !== 1 ||
      surfaceOnly.source !== tokens.source ||
      !Array.isArray(surfaceOnly.declarations)
    ) {
      errors.push(`${prefix}/surface-ast.json: invalid surface AST artifact`)
    }
  }
  if (existsSync(join(directory, "resolved-ast.json"))) {
    const resolvedOnly = readArtifact<{
      schema?: unknown
      stage?: unknown
      source?: unknown
      module?: unknown
      declarations?: unknown
    }>("resolved-ast.json")
    if (
      resolvedOnly &&
      (resolvedOnly.schema !== 1 ||
        (Object.hasOwn(resolvedOnly, "stage") &&
          resolvedOnly.stage !== "resolved-ast") ||
        resolvedOnly.source !== tokens.source ||
        resolvedOnly.module !== moduleInterface?.module ||
        !Array.isArray(resolvedOnly.declarations))
    ) {
      errors.push(`${prefix}/resolved-ast.json: invalid resolved AST artifact`)
    }
  }
  if (existsSync(join(directory, "typed-hir.json"))) {
    const typedOnly = readArtifact<{
      schema?: unknown
      stage?: unknown
      source?: unknown
      module?: unknown
      declarations?: unknown
    }>("typed-hir.json")
    if (
      typedOnly &&
      (typedOnly.schema !== 1 ||
        typedOnly.stage !== "typed-hir" ||
        typedOnly.source !== tokens.source ||
        typedOnly.module !== moduleInterface?.module ||
        !Array.isArray(typedOnly.declarations))
    ) {
      errors.push(`${prefix}/typed-hir.json: invalid typed HIR artifact`)
    }
  }
  if (hasFullLoweringStageArtifacts) {
    const stages = new Map<string, Record<string, unknown>>()
    for (const stage of stageNames) {
      const artifact = readArtifact<Record<string, unknown>>(`${stage}.json`)
      if (!artifact) continue
      const isSurfaceOnlyShape =
        stage === "surface-ast" && !Object.hasOwn(artifact, "stage")
      const isResolvedOnlyShape =
        stage === "resolved-ast" && !Object.hasOwn(artifact, "stage")
      if (
        artifact.schema !== 1 ||
        (!isSurfaceOnlyShape && !isResolvedOnlyShape && artifact.stage !== stage)
      ) {
        errors.push(`${prefix}/${stage}.json: invalid stage envelope`)
      }
      stages.set(stage, artifact)
    }
    const surface = stages.get("surface-ast") as
      | {
          source?: unknown
          declarations?: Array<Record<string, unknown>>
          root?: { declarations?: Array<Record<string, unknown>> }
        }
      | undefined
    const resolved = stages.get("resolved-ast") as
      | {
          source?: unknown
          module?: unknown
          declarations?: Array<Record<string, unknown>>
        }
      | undefined
    const typed = stages.get("typed-hir") as
      | {
          source?: unknown
          module?: unknown
          declarations?: Array<Record<string, unknown>>
        }
      | undefined
    const core = stages.get("core-ir") as
      | { module?: unknown; bindings?: Array<Record<string, unknown>> }
      | undefined
    const typescript = stages.get("typescript-ir") as
      | {
          module?: unknown
          runtimeRequirements?: unknown
          bindings?: Array<Record<string, unknown>>
        }
      | undefined
    const surfaceDecl = (surface?.declarations ??
      surface?.root?.declarations)?.[0]
    const resolvedDecl = resolved?.declarations?.[0]
    const typedDecl = typed?.declarations?.[0]
    const coreBinding = core?.bindings?.[0]
    const typescriptBinding = typescript?.bindings?.[0]
    const moduleName = moduleInterface?.module
    const symbol = moduleInterface?.exports[0]?.symbol
    const coreValue = coreBinding?.value as
      | { kind?: unknown; value?: unknown }
      | undefined
    const typescriptInitializer = typescriptBinding?.initializer as
      | { kind?: unknown; value?: unknown }
      | undefined
    if (
      surface?.source !== tokens.source ||
      surfaceDecl?.kind !== "let" ||
      resolved?.source !== tokens.source ||
      resolved?.module !== moduleName ||
      !(
        resolvedDecl?.symbol === symbol ||
        (typeof resolvedDecl?.symbol === "number" &&
          resolvedDecl.name === moduleInterface?.exports[0]?.name)
      ) ||
      typed?.source !== tokens.source ||
      typed?.module !== moduleName ||
      typedDecl?.symbol !== symbol ||
      core?.module !== moduleName ||
      coreBinding?.symbol !== symbol ||
      coreValue?.kind !== "int64" ||
      typescript?.module !== moduleName ||
      typescriptBinding?.kind !== "const" ||
      typescriptBinding?.name !== moduleInterface?.exports[0]?.name ||
      typescriptInitializer?.kind !== "bigint" ||
      typescriptInitializer?.value !== coreValue.value
    ) {
      errors.push(`${prefix}: inconsistent stage artifact chain`)
    }
    const requirements = typescript?.runtimeRequirements
    if (
      !Array.isArray(requirements) ||
      new Set(requirements).size !== requirements.length ||
      requirements.some(
        (requirement) =>
          typeof requirement !== "string" || !runtimeFeatures.has(requirement)
      )
    ) {
      errors.push(`${prefix}/typescript-ir.json: invalid runtime requirements`)
    }

    const generated = readArtifact<{
      schema: number
      module: string
      target: string
      runtime: {
        identity: string
        abiMajor: number
        requirements: string[]
      }
      outputs: { typescript: string; sourceMap: string | null }
    }>("generated-module.json")
    if (
      generated &&
      (generated.schema !== 1 ||
        generated.module !== moduleName ||
        generated.target !== "typescript-es2022" ||
        generated.runtime?.identity !== runtimeAbi?.identity ||
        generated.runtime?.abiMajor !== runtimeAbi?.abiMajor ||
        JSON.stringify(generated.runtime?.requirements) !==
          JSON.stringify(requirements) ||
        generated.outputs?.sourceMap !== "main.ts.map" ||
        generated.outputs?.typescript !== "main.ts")
    ) {
      errors.push(`${prefix}/generated-module.json: invalid generated metadata`)
    }
    if (generated && typescriptBinding && typescriptInitializer) {
      const emitted = `${typescriptBinding.exported ? "export " : ""}const ${typescriptBinding.name}: bigint = ${typescriptInitializer.value}n;\n`
      const outputPath = join(directory, generated.outputs.typescript)
      if (
        !existsSync(outputPath) ||
        readFileSync(outputPath, "utf8") !== emitted
      ) {
        errors.push(
          `${prefix}/${generated.outputs.typescript}: emitter snapshot mismatch`
        )
      }
      const sourceMapPath = join(directory, generated.outputs.sourceMap ?? "")
      try {
        const sourceMap = JSON.parse(readFileSync(sourceMapPath, "utf8")) as {
          version?: unknown
          file?: unknown
          sourceRoot?: unknown
          sources?: unknown
          sourcesContent?: unknown
          names?: unknown
          mappings?: unknown
        }
        if (
          sourceMap.version !== 3 ||
          sourceMap.file !== generated.outputs.typescript ||
          sourceMap.sourceRoot !== "" ||
          JSON.stringify(sourceMap.sources) !==
            JSON.stringify([`seseragi://${moduleName}`]) ||
          JSON.stringify(sourceMap.sourcesContent) !==
            JSON.stringify([sourceBytes.toString("utf8")]) ||
          JSON.stringify(sourceMap.names) !== JSON.stringify(["answer"]) ||
          sourceMap.mappings !== "AAAA,aAAQA,iBAAc"
        ) {
          errors.push(
            `${prefix}/${generated.outputs.sourceMap}: invalid source map`
          )
        }
      } catch (error) {
        errors.push(
          `${prefix}/${generated.outputs.sourceMap}: invalid source map JSON: ${error}`
        )
      }
    }
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

const effectArtifactsDir = join(root, "artifacts", "stage-schema-1")
for (const entry of readdirSync(effectArtifactsDir, { withFileTypes: true })) {
  if (!entry.isDirectory()) continue
  const directory = join(effectArtifactsDir, entry.name)
  const prefix = `artifacts/stage-schema-1/${entry.name}`
  const readArtifact = <T>(name: string): T | undefined => {
    try {
      return JSON.parse(readFileSync(join(directory, name), "utf8")) as T
    } catch (error) {
      errors.push(`${prefix}/${name}: invalid JSON: ${error}`)
      return undefined
    }
  }

  const moduleInterface = readArtifact<InterfaceArtifact>("interface.json")
  if (!moduleInterface) continue
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
  const sourceBytes = readFileSync(sourcePath)
  validateInterfaceArtifact(
    moduleInterface,
    moduleInterface.source,
    sourceBytes,
    prefix
  )

  type StageArtifact = {
    schema: number
    stage: string
    source?: string
    module?: string
    root?: { declarations?: Array<Record<string, unknown>> }
    declarations?: Array<Record<string, unknown>>
    functions?: Array<Record<string, unknown>>
    runtimeRequirements?: string[]
    imports?: Array<{ feature: string; local: string }>
  }
  const stages = new Map<string, StageArtifact>()
  for (const stage of [
    "surface-ast",
    "resolved-ast",
    "typed-hir",
    "core-ir",
    "typescript-ir",
  ]) {
    const artifact = readArtifact<StageArtifact>(`${stage}.json`)
    if (!artifact) continue
    if (artifact.schema !== 1 || artifact.stage !== stage) {
      errors.push(`${prefix}/${stage}.json: invalid stage envelope`)
    }
    stages.set(stage, artifact)
  }

  const surface = stages.get("surface-ast")
  const resolved = stages.get("resolved-ast")
  const typed = stages.get("typed-hir")
  const core = stages.get("core-ir")
  const typescript = stages.get("typescript-ir")
  const surfaceFn = surface?.root?.declarations?.[0]
  const resolvedFn = resolved?.declarations?.[0]
  const typedFn = typed?.declarations?.[0]
  const coreFn = core?.functions?.[0]
  const typescriptFn = typescript?.functions?.[0]
  const exported = moduleInterface.exports[0]
  const typedParameters = typedFn?.parameters as
    | Array<Record<string, unknown>>
    | undefined
  const typedEffect = typedFn?.effect as
    | {
        environment?: {
          kind?: unknown
          closed?: unknown
          fields?: Array<{ name?: unknown }>
        }
        failure?: { name?: unknown }
        success?: { name?: unknown }
      }
    | undefined
  const coreParameters = coreFn?.parameters as
    | Array<Record<string, unknown>>
    | undefined
  const coreBody = coreFn?.body as
    | {
        kind?: unknown
        operation?: unknown
        requirements?: unknown
        failure?: unknown
        success?: unknown
      }
    | undefined
  const tsParameters = typescriptFn?.parameters as
    | Array<Record<string, unknown>>
    | undefined
  const tsBody = typescriptFn?.body as
    | { kind?: unknown; callee?: unknown; arguments?: unknown[] }
    | undefined
  if (
    surface?.source !== moduleInterface.source ||
    surfaceFn?.kind !== "effect-fn" ||
    surfaceFn?.name !== exported?.name ||
    resolved?.source !== moduleInterface.source ||
    resolved?.module !== moduleInterface.module ||
    resolvedFn?.kind !== "effect-fn" ||
    resolvedFn?.symbol !== exported?.symbol ||
    resolvedFn?.implicitUnitParameter !== true ||
    typed?.source !== moduleInterface.source ||
    typed?.module !== moduleInterface.module ||
    typedFn?.symbol !== exported?.symbol ||
    typedParameters?.length !== 1 ||
    typedParameters[0]?.kind !== "implicit-unit" ||
    typedEffect?.environment?.kind !== "record" ||
    typedEffect.environment.closed !== true ||
    typedEffect.environment.fields?.length !== 1 ||
    typedEffect.environment.fields[0]?.name !== "console" ||
    typedEffect.failure?.name !== "ConsoleError" ||
    typedEffect.success?.name !== "Unit" ||
    core?.module !== moduleInterface.module ||
    coreFn?.symbol !== exported?.symbol ||
    coreParameters?.length !== 1 ||
    coreParameters[0]?.type !== "Unit" ||
    coreBody?.kind !== "effect-operation" ||
    coreBody.operation !== "console.println" ||
    JSON.stringify(coreBody.requirements) !== JSON.stringify(["console"]) ||
    coreBody.failure !== "ConsoleError" ||
    coreBody.success !== "Unit" ||
    typescript?.module !== moduleInterface.module ||
    typescriptFn?.kind !== "const-function" ||
    typescriptFn?.name !== exported?.name ||
    tsParameters?.length !== 1 ||
    tsParameters[0]?.implicit !== true ||
    tsParameters[0]?.type !== "undefined" ||
    tsBody?.kind !== "call" ||
    tsBody.arguments?.length !== 1
  ) {
    errors.push(`${prefix}: inconsistent Effect stage artifact chain`)
  }

  const requirements = typescript?.runtimeRequirements
  if (
    JSON.stringify(requirements) !==
    JSON.stringify(["core.unit", "effect.console.println"])
  ) {
    errors.push(`${prefix}/typescript-ir.json: invalid runtime requirements`)
  }
  const imports = typescript?.imports
  const imported = imports?.[0]
  const importedFeature = runtimeAbi?.features.find(
    (feature) => feature.id === imported?.feature
  )
  if (
    imports?.length !== 1 ||
    imported?.feature !== "effect.console.println" ||
    !/^[A-Za-z_$][A-Za-z0-9_$]*$/.test(imported?.local ?? "") ||
    !importedFeature ||
    importedFeature.kind !== "runtime-helper" ||
    !importedFeature.import ||
    tsBody?.callee !== imported.local
  ) {
    errors.push(`${prefix}/typescript-ir.json: invalid runtime import`)
  }

  const generated = readArtifact<{
    schema: number
    module: string
    target: string
    runtime: { identity: string; abiMajor: number; requirements: string[] }
    outputs: { typescript: string; sourceMap: string | null }
  }>("generated-module.json")
  if (
    generated &&
    (generated.schema !== 1 ||
      generated.module !== moduleInterface.module ||
      generated.target !== "typescript-es2022" ||
      generated.runtime?.identity !== runtimeAbi?.identity ||
      generated.runtime?.abiMajor !== runtimeAbi?.abiMajor ||
      JSON.stringify(generated.runtime?.requirements) !==
        JSON.stringify(requirements) ||
      generated.outputs?.typescript !== "main.ts" ||
      generated.outputs?.sourceMap !== "main.ts.map")
  ) {
    errors.push(`${prefix}/generated-module.json: invalid generated metadata`)
  }

  if (generated && imported && importedFeature?.import && tsBody) {
    const argument = tsBody.arguments?.[0] as
      | { kind?: unknown; value?: unknown }
      | undefined
    const emitted =
      `import { ${importedFeature.import.export} as ${imported.local} } from ` +
      `"${importedFeature.import.module}"\n\n` +
      `export const main = (_unit: undefined) => ${imported.local}(` +
      `${JSON.stringify(argument?.value)})\n`
    const outputPath = join(directory, generated.outputs.typescript)
    if (
      !existsSync(outputPath) ||
      readFileSync(outputPath, "utf8") !== emitted
    ) {
      errors.push(
        `${prefix}/${generated.outputs.typescript}: emitter snapshot mismatch`
      )
    }
    const sourceMapName = generated.outputs.sourceMap
    if (!sourceMapName) {
      errors.push(`${prefix}/generated-module.json: source map is required`)
    } else {
      const sourceMap = readArtifact<{
        version?: unknown
        file?: unknown
        sourceRoot?: unknown
        sources?: unknown
        sourcesContent?: unknown
        names?: unknown
        mappings?: unknown
      }>(sourceMapName)
      if (
        sourceMap &&
        (sourceMap.version !== 3 ||
          sourceMap.file !== generated.outputs.typescript ||
          sourceMap.sourceRoot !== "" ||
          JSON.stringify(sourceMap.sources) !==
            JSON.stringify([`seseragi://${moduleInterface.module}`]) ||
          JSON.stringify(sourceMap.sourcesContent) !==
            JSON.stringify([sourceBytes.toString("utf8")]) ||
          JSON.stringify(sourceMap.names) !==
            JSON.stringify(["main", "println"]) ||
          sourceMap.mappings !== ";;aAAcA,6BAGZC,sBAAQ")
      ) {
        errors.push(`${prefix}/${sourceMapName}: invalid source map`)
      }
    }
  }
}

const executionArtifactsDir = join(root, "artifacts", "execution-schema-1")
for (const entry of readdirSync(executionArtifactsDir, {
  withFileTypes: true,
})) {
  if (!entry.isDirectory()) continue
  const directory = join(executionArtifactsDir, entry.name)
  const prefix = `artifacts/execution-schema-1/${entry.name}`
  let run: {
    schema?: unknown
    case?: unknown
    target?: unknown
    entry?: {
      module?: unknown
      export?: unknown
      compiledModule?: unknown
    }
    invocation?: {
      argument?: unknown
      effect?: { cold?: unknown; rootScope?: unknown }
    }
    requiredEnvironment?: {
      kind?: unknown
      closed?: unknown
      fields?: Array<{ name?: unknown; type?: unknown }>
    }
    hostEnvironment?: {
      closed?: unknown
      services?: Array<{
        field?: unknown
        type?: unknown
        adapter?: unknown
      }>
    }
    expected?: {
      exit?: { kind?: unknown; value?: unknown }
      process?: { exitCode?: unknown }
      stdout?: unknown
      stderr?: unknown
      trace?: Array<{
        service?: unknown
        operation?: unknown
        arguments?: unknown[]
        stdout?: unknown
      }>
    }
  }
  try {
    run = JSON.parse(readFileSync(join(directory, "run.json"), "utf8"))
  } catch (error) {
    errors.push(`${prefix}/run.json: invalid JSON: ${error}`)
    continue
  }

  const compiledModuleReference = run.entry?.compiledModule
  const compiledModulePath =
    typeof compiledModuleReference === "string"
      ? resolve(directory, compiledModuleReference)
      : ""
  if (
    run.schema !== 1 ||
    run.case !== entry.name ||
    run.target !== "node-process" ||
    run.entry?.module !== "artifact/effect-main" ||
    run.entry?.export !== "main" ||
    typeof compiledModuleReference !== "string" ||
    compiledModuleReference.startsWith("/") ||
    compiledModuleReference.includes("\\") ||
    !compiledModulePath.startsWith(`${join(root, "artifacts")}/`) ||
    !existsSync(compiledModulePath) ||
    run.invocation?.argument !== "Unit" ||
    run.invocation?.effect?.cold !== true ||
    run.invocation?.effect?.rootScope !== true ||
    run.requiredEnvironment?.kind !== "record" ||
    run.requiredEnvironment.closed !== true ||
    JSON.stringify(run.requiredEnvironment.fields) !==
      JSON.stringify([{ name: "console", type: "Console" }]) ||
    run.hostEnvironment?.closed !== false ||
    JSON.stringify(run.hostEnvironment.services) !==
      JSON.stringify([
        {
          field: "console",
          type: "Console",
          adapter: "capture-console",
        },
      ]) ||
    run.expected?.exit?.kind !== "success" ||
    run.expected.exit.value !== "Unit" ||
    run.expected.process?.exitCode !== 0 ||
    run.expected.stdout !== "stdout.txt" ||
    run.expected.stderr !== "stderr.txt" ||
    JSON.stringify(run.expected.trace) !==
      JSON.stringify([
        {
          service: "console",
          operation: "println",
          arguments: ["hello"],
          stdout: "hello\n",
        },
      ])
  ) {
    errors.push(`${prefix}/run.json: invalid execution contract`)
    continue
  }

  const stdoutPath = join(directory, run.expected.stdout)
  const stderrPath = join(directory, run.expected.stderr)
  if (
    !existsSync(stdoutPath) ||
    readFileSync(stdoutPath, "utf8") !== "hello\n"
  ) {
    errors.push(`${prefix}/${run.expected.stdout}: stdout mismatch`)
  }
  if (!existsSync(stderrPath) || readFileSync(stderrPath, "utf8") !== "") {
    errors.push(`${prefix}/${run.expected.stderr}: stderr mismatch`)
  }

  try {
    const generated = JSON.parse(readFileSync(compiledModulePath, "utf8")) as {
      schema?: unknown
      module?: unknown
      target?: unknown
      runtime?: { requirements?: unknown }
      outputs?: { typescript?: unknown }
    }
    if (
      generated.schema !== 1 ||
      generated.module !== run.entry.module ||
      generated.target !== "typescript-es2022" ||
      JSON.stringify(generated.runtime?.requirements) !==
        JSON.stringify(["core.unit", "effect.console.println"]) ||
      generated.outputs?.typescript !== "main.ts"
    ) {
      errors.push(`${prefix}/run.json: compiled module reference mismatch`)
    }
  } catch (error) {
    errors.push(`${prefix}/run.json: invalid compiled module JSON: ${error}`)
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
