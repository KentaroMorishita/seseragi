export type SemanticOrigin = Readonly<{
  source: string
  start: number
  end: number
  rule: string
}>
export type SemanticNodeInspection = Readonly<{
  id: string
  node: Readonly<Record<string, unknown>>
  origins: readonly SemanticOrigin[]
  decisions: readonly Readonly<{
    rule: string
    classification: string
    reason: string
    inputs: readonly number[]
  }>[]
}>
export type SemanticInspection = Readonly<{
  id: string
  nodes: readonly SemanticNodeInspection[]
  symbols: Readonly<Record<string, unknown>>
  identity: unknown
}>

function object(value: unknown): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value))
    throw new Error("Invalid semantic graph object")
  return value as Record<string, unknown>
}
function text(value: unknown): string {
  if (typeof value !== "string" || !value.length)
    throw new Error("Invalid semantic graph string")
  return value
}
function index(value: unknown): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0)
    throw new Error("Invalid semantic graph index")
  return value
}

/** The Rust core validates meaning. The view validates its snapshot and all
 * source navigation ranges before accepting the machine-readable payload. */
export function inspectSemanticGraph(
  id: string,
  payload: unknown,
  files: readonly Readonly<{ path: string; source: string }>[]
): SemanticInspection {
  const graph = object(payload)
  if (graph.schema !== 1 || graph.kind !== "seseragi-interop-semantic-graph")
    throw new Error("Unsupported semantic graph schema")
  const types = object(graph.types)
  const sources = object(graph.sources)
  const typeOrigins = object(graph.typeOrigins)
  const encoder = new TextEncoder()
  const boundaries = new Map<string, Set<number>>()
  for (const [path, raw] of Object.entries(sources)) {
    const source = object(raw)
    const file = files.find((file) => file.path === path)
    if (
      !file ||
      source.byteLength !== encoder.encode(file.source).length ||
      typeof source.digest !== "string" ||
      !/^[a-f0-9]{64}$/.test(source.digest)
    )
      throw new Error("Semantic source does not match its workspace snapshot")
    let offset = 0
    const points = new Set([0])
    for (const scalar of file.source) {
      offset += encoder.encode(scalar).length
      points.add(offset)
    }
    boundaries.set(path, points)
  }
  const origins = (raw: unknown): readonly SemanticOrigin[] => {
    if (!Array.isArray(raw)) throw new Error("Invalid semantic provenance")
    return raw.map((raw) => {
      const value = object(raw)
      const source = text(value.source)
      const start = index(value.start)
      const end = index(value.end)
      if (
        start > end ||
        !boundaries.get(source)?.has(start) ||
        !boundaries.get(source)?.has(end)
      )
        throw new Error("Semantic provenance is outside its source")
      return { source, start, end, rule: text(value.rule) }
    })
  }
  const symbols = object(graph.symbols)
  for (const raw of Object.values(symbols)) {
    const symbol = object(raw)
    text(symbol.publicName)
    text(symbol.declarationName)
    if (!(String(index(symbol.typeRef)) in types))
      throw new Error("Missing semantic symbol type")
    origins(symbol.origins)
  }
  if (!Array.isArray(graph.normalizations))
    throw new Error("Invalid semantic decisions")
  const decisions = graph.normalizations.map((raw) => {
    const value = object(raw)
    const output = String(index(value.output))
    if (!(output in types) || !Array.isArray(value.inputs))
      throw new Error("Missing semantic decision type")
    const inputs = value.inputs.map(index)
    if (inputs.some((input) => !(String(input) in types)))
      throw new Error("Missing semantic decision input")
    return {
      output,
      inputs,
      rule: text(value.rule),
      classification: text(value.classification),
      reason: text(value.reason),
    }
  })
  for (const key of Object.keys(typeOrigins))
    if (!(key in types)) throw new Error("Missing semantic provenance type")
  return {
    id,
    symbols,
    identity: graph.identity,
    nodes: Object.entries(types).map(([id, raw]) => {
      if (!/^\d+$/.test(id)) throw new Error("Invalid semantic type identity")
      const node = object(raw)
      text(node.kind)
      // Include transitive origins of interpretation steps without hiding the
      // direct node's own provenance or following cyclic relations forever.
      const seen = new Set<string>()
      const pending = [id]
      const traced: SemanticOrigin[] = []
      while (pending.length) {
        const current = pending.pop()!
        if (seen.has(current)) continue
        seen.add(current)
        traced.push(...origins(typeOrigins[current] ?? []))
        for (const decision of decisions)
          if (decision.output === current)
            pending.push(...decision.inputs.map(String))
      }
      return {
        id,
        node,
        origins: traced,
        decisions: decisions.filter((decision) => decision.output === id),
      }
    }),
  }
}
