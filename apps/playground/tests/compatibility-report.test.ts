import { expect, test } from "bun:test"
import {
  compatibilityClasses,
  inspectCompatibilityReport,
} from "../src/workspace/compatibility-report"
import type { SemanticInspection } from "../src/workspace/semantic-inspection"

const files = [
  { path: "api.d.ts", source: "// 🙂\nexport type Text = string;" },
]
const semantic: SemanticInspection = {
  id: "api",
  nodes: [
    {
      id: "0",
      node: { kind: "primitive", domain: "string" },
      origins: [],
      decisions: [],
    },
  ],
  symbols: { Text: { publicName: "Text", typeRef: 0 } },
  identity: null,
}
function report() {
  return {
    schema: 1,
    kind: "seseragi-interop-compatibility-report",
    graphDigest: "a".repeat(64),
    identity: null,
    counts: compatibilityClasses.map((classification) => ({
      classification,
      count: classification === "direct" ? 1 : 0,
      symbols: classification === "direct" ? ["Text"] : [],
    })),
    symbols: {
      Text: {
        publicName: "Text",
        typeRef: 0,
        classification: "direct",
        semanticClassification: "direct",
        projectionClassification: null,
        runtimeAbiUnresolved: false,
        types: [0],
        reasons: [
          {
            typeRef: 0,
            classification: "direct",
            rule: "primitive",
            reason: "string domain",
            policy: null,
          },
        ],
        origins: [{ source: "api.d.ts", start: 8, end: 14, rule: "type" }],
      },
    },
  }
}
test("compatibility counts keep exact backend reasons and source provenance", () => {
  const result = inspectCompatibilityReport("api", report(), semantic, files)
  expect(result.counts[0]?.symbols).toEqual(["Text"])
  expect(result.symbols.Text?.reasons[0]?.reason).toBe("string domain")
  expect(result.symbols.Text?.origins[0]?.start).toBe(8)
})
test("forged totals, dangling types, wrong revisions and UTF-8 splits are rejected", () => {
  let input = report()
  input.counts[0]!.count = 2
  expect(() =>
    inspectCompatibilityReport("api", input, semantic, files)
  ).toThrow("count differs")
  input = report()
  input.symbols.Text.types = [100]
  expect(() =>
    inspectCompatibilityReport("api", input, semantic, files)
  ).toThrow("Dangling")
  expect(() =>
    inspectCompatibilityReport("other", report(), semantic, files)
  ).toThrow("identity")
  input = report()
  input.symbols.Text.origins[0]!.start = 4
  expect(() =>
    inspectCompatibilityReport("api", input, semantic, files)
  ).toThrow()
})
test("dynamic or unresolved meaning cannot be counted as direct", () => {
  const unsafe = {
    ...semantic,
    nodes: [
      { ...semantic.nodes[0]!, node: { kind: "primitive", domain: "dynamic" } },
    ],
  }
  expect(() =>
    inspectCompatibilityReport("api", report(), unsafe, files)
  ).toThrow("Unsafe meaning")
})

test("legal prototype-like export names remain ordinary public symbols", () => {
  const input = report()
  Object.defineProperty(input.symbols, "__proto__", {
    value: input.symbols.Text,
    enumerable: true,
  })
  delete (input.symbols as Partial<typeof input.symbols>).Text
  input.counts[0]!.symbols = ["__proto__"]
  Object.setPrototypeOf(input.symbols, null)
  const prototype = (input.symbols as Record<string, { publicName: string }>)
    .__proto__!
  prototype.publicName = "__proto__"
  const names = {
    ...semantic,
    symbols: Object.fromEntries([
      ["__proto__", { publicName: "__proto__", typeRef: 0 }],
    ]),
  }
  const result = inspectCompatibilityReport("api", input, names, files)
  expect(Object.keys(result.symbols)).toEqual(["__proto__"])
  expect(result.counts[0]?.count).toBe(1)
})

test("public roots, semantic boundaries and reason ownership cannot be hidden", () => {
  const extra = {
    ...semantic,
    symbols: {
      ...semantic.symbols,
      Hidden: { publicName: "Hidden", typeRef: 0 },
    },
  }
  expect(() =>
    inspectCompatibilityReport("api", report(), extra, files)
  ).toThrow("Incomplete")
  expect(() =>
    inspectCompatibilityReport(
      "api",
      report(),
      { ...semantic, privateSymbols: ["Text"] },
      files
    )
  ).toThrow("Unknown")
  const wrongRoot = report()
  wrongRoot.symbols.Text.publicName = "Other"
  expect(() =>
    inspectCompatibilityReport("api", wrongRoot, semantic, files)
  ).toThrow("semantic root")
  const hiddenBoundary = report()
  hiddenBoundary.symbols.Text.semanticClassification = "policy-required"
  expect(() =>
    inspectCompatibilityReport("api", hiddenBoundary, semantic, files)
  ).toThrow("hides")
  const otherType = {
    ...semantic,
    nodes: [...semantic.nodes, { ...semantic.nodes[0]!, id: "1" }],
  }
  const otherReason = report()
  otherReason.symbols.Text.reasons[0]!.typeRef = 1
  expect(() =>
    inspectCompatibilityReport("api", otherReason, otherType, files)
  ).toThrow("another symbol")
})
