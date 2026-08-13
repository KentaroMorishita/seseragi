import { describe, expect, test } from "bun:test"
import {
  maximumFeaturedSamples,
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

  test("requires Web catalog classification only for HTML samples", () => {
    expect(
      parseSampleMetadata(
        {
          ...validMetadata,
          outputMode: "html",
          experience: "minimal",
          architecture: "static",
          focus: "component",
        },
        "hello-world"
      )
    ).toMatchObject({
      experience: "minimal",
      architecture: "static",
      focus: "component",
    })

    expect(() =>
      parseSampleMetadata(
        { ...validMetadata, outputMode: "html" },
        "hello-world"
      )
    ).toThrow("requires experience, architecture and focus")
    expect(() =>
      parseSampleMetadata(
        { ...validMetadata, experience: "minimal" },
        "hello-world"
      )
    ).toThrow("must not declare Web catalog classification")
  })

  test("validates interactive and workspace architecture boundaries", () => {
    const interactiveHtml = {
      ...validMetadata,
      capabilities: ["dom"],
      outputMode: "html",
      experience: "minimal",
      architecture: "dom-app",
      focus: "state",
      interactive: true,
      files: { source: "main.ssrg", guide: "guide.md" },
    }
    expect(
      parseSampleMetadata(interactiveHtml, "hello-world").architecture
    ).toBe("dom-app")

    expect(() =>
      parseSampleMetadata(
        { ...interactiveHtml, architecture: "static" },
        "hello-world"
      )
    ).toThrow("must not be interactive")
    expect(() =>
      parseSampleMetadata(
        { ...interactiveHtml, architecture: "multi-module" },
        "hello-world"
      )
    ).toThrow("requires a workspace")
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

  test("keeps package topology out of sample view metadata", () => {
    const packageMetadata = {
      ...validMetadata,
      outputMode: "html",
      capabilities: ["dom"],
      experience: "showcase",
      architecture: "multi-module",
      focus: "project",
      interactive: true,
      files: { manifest: "seseragi.toml", guide: "guide.md" },
      workspace: {
        active: "app.ssrg",
        open: ["main.ssrg", "app.ssrg"],
        expanded: ["feature"],
      },
    }
    expect(parseSampleMetadata(packageMetadata, "hello-world")).toMatchObject({
      files: { manifest: "seseragi.toml" },
      workspace: {
        active: "app.ssrg",
        open: ["main.ssrg", "app.ssrg"],
        expanded: ["feature"],
      },
    })
    expect(() =>
      parseSampleMetadata(
        {
          ...packageMetadata,
          files: {
            source: "main.ssrg",
            manifest: "seseragi.toml",
            guide: "guide.md",
          },
        },
        "hello-world"
      )
    ).toThrow("exactly one of source or manifest")
    expect(() =>
      parseSampleMetadata(
        {
          ...packageMetadata,
          workspace: {
            ...packageMetadata.workspace,
            entry: "main.ssrg",
            files: ["main.ssrg", "app.ssrg"],
          },
        },
        "hello-world"
      )
    ).toThrow("entry and files come from seseragi.toml")
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

  test("keeps Featured exclusive to a small Discover set", () => {
    expect(() =>
      validateSampleCatalog(
        [
          {
            id: "lesson",
            kind: "lesson" as const,
            prerequisites: [],
            featured: true,
          },
        ],
        []
      )
    ).toThrow("cannot be featured in Discover")

    const samples = Array.from(
      { length: maximumFeaturedSamples + 1 },
      (_, index) => ({
        id: `recipe-${index}`,
        kind: "recipe" as const,
        prerequisites: [],
        featured: true,
      })
    )
    const group = {
      id: "recipes",
      title: "Recipes",
      summary: "Purposeful examples",
      kind: "recipe" as const,
      samples: samples.map(({ id }) => id),
    }
    expect(() => validateSampleCatalog(samples, [group])).toThrow(
      `at most ${maximumFeaturedSamples}`
    )
  })

  test("requires all five Web UI catalog roles", () => {
    const webSamples = [
      {
        id: "static",
        kind: "lesson" as const,
        prerequisites: [],
        experience: "minimal" as const,
        architecture: "static" as const,
        focus: "component" as const,
      },
      {
        id: "app",
        kind: "lesson" as const,
        prerequisites: [],
        experience: "minimal" as const,
        architecture: "dom-app" as const,
        focus: "state" as const,
        comparisonSample: "run",
      },
      {
        id: "run",
        kind: "lesson" as const,
        prerequisites: [],
        experience: "minimal" as const,
        architecture: "signal-run" as const,
        focus: "state" as const,
        comparisonSample: "app",
      },
      {
        id: "advanced",
        kind: "lesson" as const,
        prerequisites: [],
        experience: "showcase" as const,
        architecture: "signal-run" as const,
        focus: "form" as const,
      },
      {
        id: "project",
        kind: "lesson" as const,
        prerequisites: [],
        experience: "showcase" as const,
        architecture: "multi-module" as const,
        focus: "project" as const,
      },
    ]

    expect(() => validateSampleCatalog(webSamples, [])).not.toThrow()
    expect(() => validateSampleCatalog(webSamples.slice(1), [])).toThrow(
      "minimal static HTML/component"
    )
    expect(() =>
      validateSampleCatalog(
        webSamples.filter(({ architecture }) => architecture !== "dom-app"),
        []
      )
    ).toThrow("minimal dom.app")
    expect(() =>
      validateSampleCatalog(
        webSamples.map((sample) =>
          sample.id === "app"
            ? { ...sample, comparisonSample: "missing" }
            : sample
        ),
        []
      )
    ).toThrow("comparison references unknown sample missing")
  })
})
