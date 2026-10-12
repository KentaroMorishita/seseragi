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
  privateSymbols?: readonly string[]
  identity: unknown
}>

export function describeSemanticNode(
  node: Readonly<Record<string, unknown>>
): Readonly<{
  meaning: string
  relations: readonly Readonly<{ label: string; target: string }>[]
}> {
  const relations: { label: string; target: string }[] = []
  const relation = (label: string, value: unknown) => {
    if (typeof value === "number" && Number.isSafeInteger(value) && value >= 0)
      relations.push({ label, target: String(value) })
  }
  const list = (label: string, value: unknown) => {
    if (Array.isArray(value))
      value.forEach((target, index) =>
        relation(`${label} ${index + 1}`, target)
      )
  }
  let meaning: string
  switch (node.kind) {
    case "primitive":
      meaning = `Value domain: ${String(node.domain)}.`
      break
    case "literal":
      meaning = `Literal domain: ${JSON.stringify(node.value)}.`
      break
    case "parameter":
      meaning = `Generic parameter ${Number(node.index) + 1}, owned by its lexical binder.`
      relation("Binder", node.binder)
      break
    case "generic": {
      const parameters = Array.isArray(node.parameters)
        ? node.parameters.map(object)
        : []
      meaning = `Generic binder: ${parameters.map((parameter) => String(parameter.name)).join(", ")}.`
      for (const parameter of parameters) {
        relation(`${String(parameter.name)} constraint`, parameter.constraint)
        relation(`${String(parameter.name)} default`, parameter.default)
      }
      relation("Body", node.body)
      break
    }
    case "shape-transform":
      meaning = "Structural transform over a key domain; the mapper owns its lexical key parameter and preserves unspecified modifiers."
      relation("Key domain", node.domain)
      relation("Mapper", node.mapper)
      break
    case "apply":
      meaning =
        "Apply a type constructor to the declared argument relationships."
      relation("Constructor", node.constructor)
      list("Argument", node.arguments)
      break
    case "callable": {
      const parameters = Array.isArray(node.parameters)
        ? node.parameters.map(object)
        : []
      meaning = `Callable with ${parameters.length} parameter(s); evaluation and failure guarantees belong to its runtime contract.`
      for (const parameter of parameters)
        relation(
          `${String(parameter.name)}${parameter.optional ? " (optional)" : ""}${parameter.rest ? " (rest)" : ""}`,
          parameter.typeRef
        )
      relation("Result", node.result)
      relation("Receiver", node.receiver)
      break
    }
    case "collection":
      meaning = node.mutable
        ? "Collection with mutable identity."
        : "Readonly collection domain; a foreign snapshot still requires a boundary contract."
      relation("Element", node.element)
      break
    case "async":
      meaning =
        "Asynchronous result; cancellation and failure remain runtime contract decisions."
      relation("Value", node.value)
      break
    case "nullable":
    case "optional":
      meaning =
        node.kind === "nullable"
          ? "Value domain extended with null."
          : "Optional value domain."
      relation("Value", node.value)
      break
    case "opaque":
      meaning = `Opaque identity: ${String(node.identity)}. Structural operations require their own interpretation.`
      list("Argument", node.arguments)
      break
    case "reference":
      meaning = `Declaration reference: ${String(node.symbol)}.`
      break
    case "tuple":
      meaning = "Ordered tuple element domains."
      list("Element", node.elements)
      break
    case "union":
    case "intersection":
      meaning =
        node.kind === "union"
          ? "Union of the listed value domains."
          : "Intersection of the listed type relationships."
      list("Member", node.members)
      break
    case "overload":
      meaning =
        "Overload signatures; runtime selection must be explicitly approved."
      list("Signature", node.signatures)
      break
    case "projection":
      meaning = `Type projection relation: ${String(node.relation)}.`
      relation("Subject", node.subject)
      relation("Key", node.key)
      break
    case "shape":
      meaning = `Structural shape with ${String(node.identity)} identity.`
      for (const [name, raw] of Object.entries(object(node.fields))) {
        const field = object(raw)
        relation(
          `${name}${field.optional ? " (optional)" : ""}${field.readonly ? " (readonly)" : ""}`,
          field.typeRef
        )
      }
      break
    case "namespace":
      meaning = `Member ownership: ${Object.keys(object(node.members)).join(", ")}.`
      break
    case "unresolved":
      meaning = `Meaning remains unresolved: ${String(node.reason)}.`
      break
    default:
      throw new Error("Unsupported semantic type kind")
  }
  return { meaning, relations }
}

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
  const rawPrivateSymbols = graph.privateSymbols ?? []
  if (!Array.isArray(rawPrivateSymbols)) throw new Error("Invalid private semantic declaration set")
  const privateSymbols = rawPrivateSymbols.map(text)
  if (privateSymbols.some((symbol) => !(symbol in symbols))) throw new Error("Missing private semantic declaration")
  return {
    id,
    symbols,
    privateSymbols,
    identity: graph.identity,
    nodes: Object.entries(types).map(([id, raw]) => {
      if (!/^\d+$/.test(id)) throw new Error("Invalid semantic type identity")
      const node = object(raw)
      text(node.kind)
      const description = describeSemanticNode(node)
      if (description.relations.some((relation) => !(relation.target in types)))
        throw new Error("Missing semantic type relation")
      if (node.kind === "reference" && !(text(node.symbol) in symbols))
        throw new Error("Missing semantic declaration reference")
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
