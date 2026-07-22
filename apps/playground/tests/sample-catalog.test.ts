import { describe, expect, test } from "bun:test"
import {
  parseLearningPaths,
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

  test("rejects missing prerequisites and cycles", () => {
    expect(() =>
      validateSampleCatalog([{ id: "one", prerequisites: ["missing"] }], [])
    ).toThrow("missing prerequisite")
    expect(() =>
      validateSampleCatalog(
        [
          { id: "one", prerequisites: ["two"] },
          { id: "two", prerequisites: ["one"] },
        ],
        []
      )
    ).toThrow("prerequisite cycle")
  })

  test("rejects paths that reference unknown samples", () => {
    const paths = parseLearningPaths({
      schema: 1,
      paths: [
        {
          id: "start",
          title: "Start",
          summary: "Start here",
          samples: ["missing"],
        },
      ],
    })
    expect(() => validateSampleCatalog([], paths)).toThrow("missing sample")
  })
})
