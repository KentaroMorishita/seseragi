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

type ShowcaseReview = Readonly<{
  readonly schema: number
  readonly sampleId: string
  readonly reference: Readonly<{
    readonly visual: string
    readonly source: string
    readonly checked: boolean
  }>
  readonly designIntent: Readonly<{
    readonly firstView: string
    readonly layoutRhythm: string
    readonly visualIdentity: string
    readonly interaction: string
    readonly codeStructure: string
  }>
  readonly approval: Readonly<{
    readonly status: string
    readonly reviewedAt: string
    readonly evidence: readonly string[]
    readonly viewports: readonly string[]
    readonly states: readonly string[]
    readonly surfaces: readonly string[]
  }>
}>

const audit = JSON.parse(
  await Bun.file(
    new URL("./fixtures/web-ui-visual-audit.json", import.meta.url)
  ).text()
) as VisualAudit

const repoRoot = new URL("../../../", import.meta.url)
const visualReference =
  "https://github.com/user-attachments/assets/" +
  "2a77e71a-9060-43a6-ae19-f1293ff938e2"
const sourceReference =
  "https://github.com/user-attachments/files/30919541/" +
  "seseragi-landing-page.zip"

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

async function showcaseReviewFor(id: string): Promise<ShowcaseReview> {
  return JSON.parse(
    await Bun.file(
      new URL(`examples/samples/${id}/showcase-review.json`, repoRoot)
    ).text()
  ) as ShowcaseReview
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
    expect(groups.get("trail-planner-comparison")?.map(({ id }) => id)).toEqual(
      ["interactive-app", "signal-run-route"]
    )
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

  test("requires approved design intent for every Web Showcase", async () => {
    const showcaseIds = samples
      .filter(
        ({ kind, outputMode }) => kind === "showcase" && outputMode === "html"
      )
      .map(({ id }) => id)
      .sort()

    const reviews = await Promise.all(showcaseIds.map(showcaseReviewFor))
    expect(reviews.map(({ sampleId }) => sampleId).sort()).toEqual(showcaseIds)

    for (const review of reviews) {
      expect(review.schema).toBe(1)
      expect(review.reference).toEqual({
        visual: visualReference,
        source: sourceReference,
        checked: true,
      })
      for (const intent of Object.values(review.designIntent)) {
        expect(intent.trim().length).toBeGreaterThanOrEqual(48)
      }
      expect(review.approval.status).toBe("approved")
      expect(review.approval.reviewedAt).toMatch(/^\d{4}-\d{2}-\d{2}$/)
      expect(review.approval.viewports).toEqual([
        "desktop",
        "iphone-390",
        "android-360",
      ])
      expect(review.approval.states).toContain("initial")
      expect(review.approval.states.length).toBeGreaterThanOrEqual(2)
      expect(review.approval.surfaces).toEqual(["preview", "code"])
      expect(review.approval.evidence).not.toHaveLength(0)

      for (const evidence of review.approval.evidence) {
        const artifact = Bun.file(new URL(evidence, repoRoot))
        expect(await artifact.exists()).toBe(true)
        expect((await artifact.text()).trim().length).toBeGreaterThan(100)
      }
    }
  })

  test("publishes the quality contract and Showcase issue template", async () => {
    const contract = await Bun.file(
      new URL("docs/SHOWCASE_QUALITY.md", repoRoot)
    ).text()
    const template = await Bun.file(
      new URL(".github/ISSUE_TEMPLATE/showcase.md", repoRoot)
    ).text()

    expect(contract).toContain(visualReference)
    expect(contract).toContain(sourceReference)
    expect(contract).toContain("Application to #244")
    expect(contract).toContain("generic card-grid")
    expect(contract).toContain("mobile Code surface")
    expect(template).toContain("docs/SHOWCASE_QUALITY.md")
    expect(template).toContain("CI greenだけではこのIssueをcloseしない")
  })
})
