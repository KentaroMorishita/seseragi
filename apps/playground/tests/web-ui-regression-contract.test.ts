import { describe, expect, test } from "bun:test"
import { samples } from "../src/samples"
import matrix from "./fixtures/web-ui-regression.json"

const expectedViewports = [
  ["desktop", 1440, 1000],
  ["landscape-568", 568, 320],
  ["iphone-390", 390, 844],
  ["android-360", 360, 800],
  ["minimum-320", 320, 700],
]

describe("Web UI visual regression matrix", () => {
  test("covers every canonical HTML sample, viewport, surface, and state", () => {
    const htmlSamples = samples.filter(
      ({ outputMode }) => outputMode === "html"
    )

    expect(matrix.schema).toBe(1)
    expect(matrix.runner).toEqual({
      kind: "playwright",
      browser: "chromium",
      baseUrl: "http://127.0.0.1:4173",
      artifactRoot: "test-results/web-ui-review",
    })
    expect(
      matrix.viewports.map(({ id, width, height }) => [id, width, height])
    ).toEqual(expectedViewports)
    expect(matrix.samples.map(({ id }) => id).sort()).toEqual(
      htmlSamples.map(({ id }) => id).sort()
    )

    for (const entry of matrix.samples) {
      const sample = htmlSamples.find(({ id }) => id === entry.id)
      expect(sample).toBeDefined()
      if (sample === undefined) continue

      expect(sample.title).toBe(entry.pickerLabel)
      expect(sample.architecture).toBeDefined()
      if (sample.architecture === undefined) continue
      expect(entry.architecture).toBe(sample.architecture)
      expect(entry.requiredSurfaces).toContain("sample-picker")
      expect(entry.requiredSurfaces).toContain("code")
      expect(entry.requiredSurfaces).toContain("preview")
      expect(entry.requiredStates[0]).toBe("initial")
      expect(entry.requiredStates).toContain("image-fallback")
      expect(entry.heading.length).toBeGreaterThan(3)
    }
  })

  test("keeps mobile source readable before browser review starts", () => {
    for (const sample of samples.filter(
      ({ outputMode }) => outputMode === "html"
    )) {
      const source = sample.workspace.files
        .map(({ source }) => source)
        .join("\n")
      const longClassLiterals = source
        .split("\n")
        .filter((line) => line.includes('class: "') && line.length > 96)

      expect(longClassLiterals, sample.id).toEqual([])
      expect(
        source.includes('"min-h-screen"') ||
          source.includes('minHeight: "100vh"'),
        sample.id
      ).toBe(true)
      expect(
        source.includes('"object-cover"') ||
          source.includes('objectFit: "cover"'),
        sample.id
      ).toBe(true)
    }
  })
})
