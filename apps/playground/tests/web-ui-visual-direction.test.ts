import { describe, expect, test } from "bun:test"
import { samples } from "../src/samples"

type AuditSample = Readonly<{
  readonly id: string
  readonly identity: string
  readonly photoId: string
  readonly photoGroup: string
  readonly states: readonly string[]
  readonly artifacts: readonly string[]
}>

type VisualAudit = Readonly<{
  readonly schema: number
  readonly requiredPreviewViewports: readonly string[]
  readonly samples: readonly AuditSample[]
}>

const audit = JSON.parse(
  await Bun.file(
    new URL("./fixtures/web-ui-visual-audit.json", import.meta.url)
  ).text()
) as VisualAudit

const repoRoot = new URL("../../../", import.meta.url)

function sourceFor(id: string): string {
  const sample = samples.find((candidate) => candidate.id === id)
  if (sample === undefined) throw new Error(`missing sample: ${id}`)
  return sample.workspace.files.map(({ source }) => source).join("\n")
}

async function assertPngArtifact(path: string): Promise<void> {
  const bytes = new Uint8Array(
    await Bun.file(new URL(path, repoRoot)).arrayBuffer()
  )
  expect([...bytes.slice(0, 8)]).toEqual([137, 80, 78, 71, 13, 10, 26, 10])
  expect(bytes.length).toBeGreaterThan(1000)
}

describe("canonical Web UI visual direction", () => {
  test("covers every HTML sample with a distinct visual identity and fixed image", () => {
    const htmlSamples = samples
      .filter(({ outputMode }) => outputMode === "html")
      .map(({ id }) => id)

    expect(audit.schema).toBe(1)
    expect(audit.requiredPreviewViewports).toEqual([
      "desktop",
      "iphone-390",
      "android-360",
    ])
    expect(audit.samples.map(({ id }) => id).sort()).toEqual(htmlSamples.sort())

    for (const sample of audit.samples) {
      const source = sourceFor(sample.id)
      expect(sample.identity.length).toBeGreaterThan(16)
      expect(sample.states).not.toHaveLength(0)
      expect(source).toContain(sample.photoId)
      expect(source).toContain("images.unsplash.com")
      expect(source).toContain("alt:")
      expect(source).toContain("width:")
      expect(source).toContain("height:")
      expect(source).toContain('"object-cover"')
      expect(source).toContain('"min-h-screen"')
    }

    const groups = new Map<string, AuditSample[]>()
    for (const sample of audit.samples) {
      groups.set(sample.photoGroup, [
        ...(groups.get(sample.photoGroup) ?? []),
        sample,
      ])
    }
    expect([...groups.keys()]).toEqual([
      "html-components",
      "trail-planner-comparison",
      "feature-composition",
      "form-todo",
      "project-flow-app",
    ])
    expect(groups.get("trail-planner-comparison")?.map(({ id }) => id)).toEqual([
      "interactive-app",
      "signal-run-route",
    ])
    for (const [group, groupSamples] of groups) {
      const photoIds = new Set(groupSamples.map(({ photoId }) => photoId))
      expect(photoIds.size).toBe(1)
      if (group === "trail-planner-comparison") {
        expect(groupSamples).toHaveLength(2)
      } else {
        expect(groupSamples).toHaveLength(1)
      }
    }
  })

  test("keeps responsive layouts and purpose-driven copy in the refreshed samples", () => {
    const htmlComponents = sourceFor("html-components")
    const trail = sourceFor("interactive-app")
    const explicitTrail = sourceFor("signal-run-route")
    const composition = sourceFor("feature-composition")

    expect(htmlComponents).toContain("#fff7ed")
    expect(htmlComponents).toContain('"flex-wrap"')
    expect(trail).toContain('"grid-cols-1"')
    expect(trail).toContain('"sm:grid-cols-3"')
    expect(explicitTrail).toContain('"grid-cols-1"')
    expect(explicitTrail).toContain('"sm:grid-cols-3"')
    expect(composition).toContain('"grid-cols-1"')
    expect(composition).toContain('"sm:grid-cols-2"')
    expect(composition).toContain("Compose the release rhythm.")
    expect(composition).not.toContain("Counter A")
    expect(composition).not.toContain("Counter B")
  })

  test("keeps desktop, mobile Preview, Code, and interaction review artifacts", async () => {
    for (const sample of audit.samples) {
      expect(sample.artifacts.length).toBeGreaterThanOrEqual(3)
      for (const artifact of sample.artifacts) {
        await assertPngArtifact(artifact)
      }
    }
  })
})
