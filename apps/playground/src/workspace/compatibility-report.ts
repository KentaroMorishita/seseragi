import type { SemanticInspection, SemanticOrigin } from "./semantic-inspection"

export const compatibilityClasses = [
  "direct",
  "normalized",
  "opaque-preserved",
  "safe-boundary",
  "policy-required",
  "runtime-abi-unresolved",
  "uninterpreted",
] as const
export type CompatibilityClass = (typeof compatibilityClasses)[number]
export type CompatibilityReason = Readonly<{
  typeRef: number
  classification: CompatibilityClass
  rule: string
  reason: string
  policy: Readonly<{ name: string; version: string }> | null
}>
export type CompatibilitySymbol = Readonly<{
  publicName: string
  typeRef: number
  classification: CompatibilityClass
  semanticClassification: CompatibilityClass
  projectionClassification: CompatibilityClass | null
  runtimeAbiUnresolved: boolean
  types: readonly number[]
  reasons: readonly CompatibilityReason[]
  origins: readonly SemanticOrigin[]
}>
export type CompatibilityInspection = Readonly<{
  id: string
  graphDigest: string
  counts: readonly Readonly<{
    classification: CompatibilityClass
    count: number
    symbols: readonly string[]
  }>[]
  symbols: Readonly<Record<string, CompatibilitySymbol>>
}>
function object(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value))
    throw new Error("Invalid semantic compatibility object")
  return value as Record<string, unknown>
}
function string(value: unknown): string {
  if (typeof value !== "string" || value.trim().length === 0)
    throw new Error("Missing semantic compatibility text")
  return value
}
function classification(value: unknown): CompatibilityClass {
  if (!compatibilityClasses.includes(value as CompatibilityClass))
    throw new Error("Invalid semantic compatibility classification")
  return value as CompatibilityClass
}
function array(value: unknown): unknown[] {
  if (!Array.isArray(value))
    throw new Error("Invalid semantic compatibility list")
  return value
}
function integer(value: unknown): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0)
    throw new Error("Invalid semantic compatibility type/count")
  return value
}
export function inspectCompatibilityReport(
  id: string,
  value: unknown,
  semantic: SemanticInspection,
  files: readonly Readonly<{ path: string; source: string }>[]
): CompatibilityInspection {
  const report = object(value)
  if (
    report.schema !== 1 ||
    report.kind !== "seseragi-interop-compatibility-report"
  )
    throw new Error("Unsupported semantic compatibility report")
  const graphDigest = string(report.graphDigest)
  if (!/^[a-f0-9]{64}$/.test(graphDigest) || semantic.id !== id)
    throw new Error("Invalid semantic compatibility identity")
  const nodes = new Map(
    semantic.nodes.map((node) => [Number(node.id), node.node])
  )
  const type = (value: unknown) => {
    const result = integer(value)
    if (!nodes.has(result)) throw new Error("Dangling compatibility type")
    return result
  }
  const origin = (value: unknown): SemanticOrigin => {
    const item = object(value)
    const source = string(item.source)
    const input = files.find((file) => file.path === source)
    if (!input) throw new Error("Missing compatibility source")
    const bytes = new TextEncoder().encode(input.source)
    const start = integer(item.start)
    const end = integer(item.end)
    if (start > end || end > bytes.length)
      throw new Error("Invalid compatibility source range")
    // Reject positions inside UTF-8 scalars, including zero-length ranges.
    const decoder = new TextDecoder("utf-8", { fatal: true })
    decoder.decode(bytes.slice(0, start))
    decoder.decode(bytes.slice(0, end))
    return { source, start, end, rule: string(item.rule) }
  }
  const privateSymbols = new Set(semantic.privateSymbols ?? [])
  const publicSymbols = Object.keys(semantic.symbols).filter(
    (identity) => !privateSymbols.has(identity)
  )
  const symbols: Record<string, CompatibilitySymbol> = Object.create(null)
  for (const [identity, raw] of Object.entries(object(report.symbols))) {
    if (
      !Object.hasOwn(semantic.symbols, identity) ||
      privateSymbols.has(identity)
    )
      throw new Error("Unknown compatibility symbol")
    const item = object(raw)
    const declared = object(semantic.symbols[identity])
    if (
      item.publicName !== declared.publicName ||
      item.typeRef !== declared.typeRef
    )
      throw new Error("Compatibility symbol differs from its semantic root")
    const types = array(item.types).map(type)
    if (new Set(types).size !== types.length)
      throw new Error("Duplicate compatibility type")
    const status = classification(item.classification)
    if (
      (status === "direct" || status === "normalized") &&
      types.some((id) => {
        const node = nodes.get(id)
        return (
          node?.kind === "unresolved" ||
          (node?.kind === "primitive" && node.domain === "dynamic")
        )
      })
    )
      throw new Error("Unsafe meaning cannot count as direct compatibility")
    const semanticStatus = classification(item.semanticClassification)
    const projectionStatus =
      item.projectionClassification === null
        ? null
        : classification(item.projectionClassification)
    if (
      compatibilityClasses.indexOf(status) <
        compatibilityClasses.indexOf(semanticStatus) ||
      (projectionStatus !== null &&
        compatibilityClasses.indexOf(status) <
          compatibilityClasses.indexOf(projectionStatus))
    )
      throw new Error(
        "Compatibility status hides a semantic or projection boundary"
      )
    if (typeof item.runtimeAbiUnresolved !== "boolean")
      throw new Error("Missing compatibility ABI state")
    const reasons = array(item.reasons).map((raw): CompatibilityReason => {
      const reason = object(raw)
      const policy = reason.policy === null ? null : object(reason.policy)
      const typeRef = type(reason.typeRef)
      if (!types.includes(typeRef))
        throw new Error("Compatibility reason belongs to another symbol")
      return {
        typeRef,
        classification: classification(reason.classification),
        rule: string(reason.rule),
        reason: string(reason.reason),
        policy: policy && {
          name: string(policy.name),
          version: string(policy.version),
        },
      }
    })
    if (!reasons.length) throw new Error("Compatibility symbol has no evidence")
    if (
      status === "safe-boundary" &&
      !reasons.some((reason) => reason.policy !== null)
    )
      throw new Error("Safe compatibility boundary has no applied policy")
    const typeRef = type(item.typeRef)
    if (!types.includes(typeRef))
      throw new Error("Compatibility root is missing")
    symbols[identity] = {
      publicName: string(item.publicName),
      typeRef,
      classification: status,
      semanticClassification: semanticStatus,
      projectionClassification: projectionStatus,
      runtimeAbiUnresolved: item.runtimeAbiUnresolved,
      types,
      reasons,
      origins: array(item.origins).map(origin),
    }
  }
  const counted = new Set<string>()
  const counts = array(report.counts).map((raw, index) => {
    const item = object(raw)
    const status = classification(item.classification)
    if (status !== compatibilityClasses[index])
      throw new Error("Invalid compatibility count order")
    const identities = array(item.symbols).map(string)
    const count = integer(item.count)
    if (count !== identities.length)
      throw new Error("Compatibility count differs from evidence")
    for (const identity of identities) {
      if (counted.has(identity) || symbols[identity]?.classification !== status)
        throw new Error("Compatibility count does not match public symbols")
      counted.add(identity)
    }
    return { classification: status, count, symbols: identities }
  })
  if (
    counts.length !== compatibilityClasses.length ||
    counted.size !== Object.keys(symbols).length ||
    Object.keys(symbols).length !== publicSymbols.length
  )
    throw new Error("Incomplete semantic compatibility counts")
  return { id, graphDigest, counts, symbols }
}
