import { describe, expect, test } from "bun:test"
import {
  parseDiscoverGroups,
  parseSampleMetadata,
  validateSampleCatalog,
} from "../src/sample-catalog"

const validMetadata = {
  id: "hello-world",
  title: "Hello world",
  summary: "Small start",
  kind: "lesson",
  difficulty: "beginner",
  topics: ["main"],
  capabilities: ["console"],
  outputMode: "text",
  prerequisites: [],
  featured: true,
  files: {
    source: "main.ssrg",
    guide: "guide.md",
    expectedOutput: "stdout.txt",
  },
}

describe("sample catalog validation", () => {
  test("rejects unknown metadata fields", () => {
    expect(() =>
      parseSampleMetadata({ ...validMetadata, sequence: 1 }, "hello-world")
    ).toThrow("unknown field")
  })

  test("requires output snapshots for non-interactive samples", () => {
    expect(() =>
      parseSampleMetadata(
        { ...validMetadata, files: { source: "main.ssrg", guide: "guide.md" } },
        "hello-world"
      )
    ).toThrow("requires expectedOutput")
  })

  test("normalizes a multi-file project workspace", () => {
    expect(
      parseSampleMetadata(
        {
          ...validMetadata,
          workspace: {
            entry: "main.ssrg",
            files: ["main.ssrg", "feature/greeting.ssrg"],
            open: ["main.ssrg", "feature/greeting.ssrg"],
            active: "feature/greeting.ssrg",
            expanded: ["feature"],
          },
        },
        "hello-world"
      ).workspace
    ).toEqual({
      entry: "main.ssrg",
      files: ["main.ssrg", "feature/greeting.ssrg"],
      open: ["main.ssrg", "feature/greeting.ssrg"],
      active: "feature/greeting.ssrg",
      expanded: ["feature"],
    })
  })

  test("parses explicit Preview dynamic and custom class contracts", () => {
    expect(
      parseSampleMetadata(
        {
          ...validMetadata,
          preview: {
            dynamicUtilities: ["sm:p-10"],
            customClasses: ["sample-hook"],
          },
        },
        "hello-world"
      ).preview
    ).toEqual({
      dynamicUtilities: ["sm:p-10"],
      customClasses: ["sample-hook"],
    })

    expect(() =>
      parseSampleMetadata(
        {
          ...validMetadata,
          preview: { dynamicUtilities: ["two tokens"] },
        },
        "hello-world"
      )
    ).toThrow("individual class tokens")
  })

  test("rejects project paths outside the declared workspace", () => {
    expect(() =>
      parseSampleMetadata(
        {
          ...validMetadata,
          workspace: {
            entry: "main.ssrg",
            files: ["main.ssrg", "feature/greeting.ssrg"],
            active: "missing.ssrg",
          },
        },
        "hello-world"
      )
    ).toThrow("active must appear in files")
    expect(() =>
      parseSampleMetadata(
        {
          ...validMetadata,
          workspace: {
            entry: "main.ssrg",
            files: ["main.ssrg", "../outside.ssrg"],
          },
        },
        "hello-world"
      )
    ).toThrow("relative sample path")
  })

  test("rejects missing prerequisites and cycles", () => {
    expect(() =>
      validateSampleCatalog(
        [{ id: "one", kind: "lesson", prerequisites: ["missing"] }],
        []
      )
    ).toThrow("missing prerequisite")
    expect(() =>
      validateSampleCatalog(
        [
          { id: "one", kind: "lesson", prerequisites: ["two"] },
          { id: "two", kind: "lesson", prerequisites: ["one"] },
        ],
        []
      )
    ).toThrow("prerequisite cycle")
  })

  test("rejects discover groups that reference unknown samples", () => {
    const groups = parseDiscoverGroups({
      schema: 1,
      groups: [
        {
          id: "start",
          title: "Start",
          summary: "Start here",
          kind: "recipe",
          samples: ["missing"],
        },
      ],
    })
    expect(() => validateSampleCatalog([], groups)).toThrow("missing sample")
  })

  test("requires each Recipe and Showcase in one matching discover group", () => {
    const recipe = { id: "one", kind: "recipe" as const, prerequisites: [] }
    const showcase = {
      id: "two",
      kind: "showcase" as const,
      prerequisites: [],
    }
    const recipeGroup = {
      id: "recipes",
      title: "Recipes",
      summary: "Purposeful examples",
      kind: "recipe" as const,
      samples: ["one"],
    }

    expect(() =>
      validateSampleCatalog([recipe, showcase], [recipeGroup])
    ).toThrow("showcase sample two is missing")
    expect(() =>
      validateSampleCatalog(
        [recipe],
        [recipeGroup, { ...recipeGroup, id: "more-recipes" }]
      )
    ).toThrow("multiple discover groups")
    expect(() =>
      validateSampleCatalog(
        [recipe],
        [{ ...recipeGroup, kind: "showcase" as const }]
      )
    ).toThrow("requires showcase samples")
  })
})
