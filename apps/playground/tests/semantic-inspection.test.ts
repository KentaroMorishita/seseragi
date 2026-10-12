import { expect, test } from "bun:test"
import { inspectSemanticGraph } from "../src/workspace/semantic-inspection"

const source = "// 日本語\nexport type Name = string;"
const files = [{ path: "api.d.ts", source }]
const start = new TextEncoder().encode("// 日本語\nexport type Name = ").length
const origin = { source: "api.d.ts", start, end: start + 6, rule: "domain" }
function graph() {
  return {
    schema: 1,
    kind: "seseragi-interop-semantic-graph",
    types: {
      "0": { kind: "primitive", domain: "string" },
      "1": { kind: "nullable", value: 0 },
    },
    symbols: {
      Name: {
        publicName: "Name",
        declarationName: "Name",
        typeRef: 1,
        origins: [origin],
      },
    },
    sources: {
      "api.d.ts": {
        digest: "a".repeat(64),
        byteLength: new TextEncoder().encode(source).length,
      },
    },
    typeOrigins: { "0": [origin] },
    normalizations: [
      {
        output: 1,
        inputs: [0],
        rule: "nullable-domain",
        classification: "normalized",
        reason: "null domain is explicit",
      },
      {
        output: 0,
        inputs: [1],
        rule: "recursive-evidence",
        classification: "direct",
        reason: "cycle is visited once",
      },
    ],
    identity: { frontend: { name: "test", version: "1" } },
  }
}

test("semantic inspection keeps relations, reasons and transitive Unicode byte provenance", () => {
  const result = inspectSemanticGraph("api", graph(), files)
  expect(result.id).toBe("api")
  expect(result.nodes[1]?.node).toEqual({ kind: "nullable", value: 0 })
  expect(result.nodes[1]?.origins).toEqual([origin])
  expect(result.nodes[1]?.decisions[0]?.reason).toBe("null domain is explicit")
  expect(result.symbols).toEqual(graph().symbols)
})

test("incompatible graphs, missing source snapshots and invalid UTF-8 boundaries are rejected", () => {
  expect(() =>
    inspectSemanticGraph("api", { ...graph(), schema: 2 }, files)
  ).toThrow("schema")
  expect(() => inspectSemanticGraph("api", graph(), [])).toThrow("snapshot")
  expect(() =>
    inspectSemanticGraph("api", graph(), [
      { path: "api.d.ts", source: "changed" },
    ])
  ).toThrow("snapshot")
  const malformed = graph()
  malformed.typeOrigins["0"] = [{ ...origin, start: 4, end: 5 }]
  expect(() => inspectSemanticGraph("api", malformed, files)).toThrow(
    "provenance"
  )
  malformed.typeOrigins["0"] = [{ ...origin, end: 999 }]
  expect(() => inspectSemanticGraph("api", malformed, files)).toThrow(
    "provenance"
  )
})

test("missing symbol, normalization and provenance nodes fail before navigation", () => {
  const malformed = graph()
  malformed.symbols.Name.typeRef = 999
  expect(() => inspectSemanticGraph("api", malformed, files)).toThrow(
    "symbol type"
  )
  malformed.symbols.Name.typeRef = 1
  malformed.normalizations[0]!.inputs = [999]
  expect(() => inspectSemanticGraph("api", malformed, files)).toThrow(
    "decision input"
  )
  malformed.normalizations[0]!.inputs = [0]
  expect(() =>
    inspectSemanticGraph(
      "api",
      { ...malformed, typeOrigins: { "999": [origin] } },
      files
    )
  ).toThrow("provenance type")
})
