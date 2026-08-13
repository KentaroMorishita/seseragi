import { describe, expect, test } from "bun:test"
import { samples } from "../src/samples"

type InteractionAction = Readonly<{
  readonly kind: string
  readonly selector?: string
  readonly buttonText?: string
  readonly value?: string
}>

type InteractionStep = Readonly<{
  readonly id: string
  readonly actions: readonly InteractionAction[]
  readonly assert: Readonly<Record<string, unknown>>
}>

type InteractionFixture = Readonly<{
  readonly schema: number
  readonly sampleId: string
  readonly sourcePath: string
  readonly manifest: Readonly<{
    readonly sourceHash: string
    readonly workspaceHash: string
  }>
  readonly runner: Readonly<{
    readonly kind: string
    readonly url: string
    readonly previewSelector: string
    readonly cleanup: Readonly<{
      readonly mountedResourceCount: number
      readonly resourceCountAfterCleanup: number
    }>
  }>
  readonly requiredOperations: readonly string[]
  readonly viewports: readonly Readonly<{
    readonly id: string
    readonly previewWidth: number
    readonly previewHeight: number
    readonly browserViewport: string
    readonly horizontalOverflow: boolean
    readonly surfaces: readonly string[]
    readonly artifact?: string
    readonly artifacts?: readonly string[]
  }>[]
  readonly steps: readonly InteractionStep[]
}>

const fixture = JSON.parse(
  await Bun.file(
    new URL("./fixtures/project-flow-app.interaction.json", import.meta.url)
  ).text()
) as InteractionFixture

const repoRoot = new URL("../../../", import.meta.url)

async function assertPngArtifact(path: string): Promise<void> {
  const bytes = new Uint8Array(
    await Bun.file(new URL(path, repoRoot)).arrayBuffer()
  )
  expect([...bytes.slice(0, 8)]).toEqual([137, 80, 78, 71, 13, 10, 26, 10])
  expect(bytes.length).toBeGreaterThan(1000)
}

describe("project-flow-app browser interaction fixture", () => {
  test("ties the Release Room project, Explorer defaults, and source contract to the manifest", () => {
    const sample = samples.find(({ id }) => id === fixture.sampleId)
    expect(sample).toBeDefined()
    if (sample === undefined) return

    expect(fixture.schema).toBe(1)
    expect(sample.sourcePath).toBe(fixture.sourcePath)
    expect(sample.manifestPath).toBe(
      "examples/samples/project-flow-app/seseragi.toml"
    )
    expect(sample.manifest).toContain('target = "web"')
    expect(sample.interactive).toBe(true)
    expect(sample.sourceHash).toBe(fixture.manifest.sourceHash)
    expect(sample.workspaceHash).toBe(fixture.manifest.workspaceHash)
    expect(fixture.runner.kind).toBe("browser-use")
    expect(fixture.runner.previewSelector).toBe("#html-preview")
    expect(sample.project).toMatchObject({
      entryFile: "main.ssrg",
      activeFile: "app.ssrg",
      openFiles: [
        "main.ssrg",
        "app.ssrg",
        "ui/components.ssrg",
        "focus/model.ssrg",
        "notes/model.ssrg",
      ],
      expandedFolders: ["ui", "focus", "notes"],
    })

    const sourceByPath = new Map(
      sample.workspace.files.map(({ path, source }) => [path, source])
    )
    expect([...sourceByPath.keys()]).toEqual([
      "main.ssrg",
      "app.ssrg",
      "focus/model.ssrg",
      "focus/view.ssrg",
      "notes/form.ssrg",
      "notes/model.ssrg",
      "notes/view.ssrg",
      "ui/components.ssrg",
      "ui/styles.ssrg",
    ])

    const source = (path: string): string => {
      const value = sourceByPath.get(path)
      if (value === undefined)
        throw new Error(`missing project source: ${path}`)
      return value
    }
    expect(source("main.ssrg")).toContain('dom.query "#app"')
    expect(source("main.ssrg")).toContain("dom.defaultOptions ()")
    expect(source("main.ssrg")).toContain("dom.run")
    expect(source("app.ssrg")).toContain("signals.make $ ShellState")
    expect(source("app.ssrg")).toContain("signals.map")
    expect(source("app.ssrg")).toContain("<$>")
    expect(source("app.ssrg")).toContain("<*>")
    expect(source("app.ssrg")).toContain("createFocus")
    expect(source("app.ssrg")).toContain("createNotes")
    expect(source("app.ssrg")).toContain("parseSampleUrl")
    expect(source("app.ssrg")).not.toContain("dom.app")
    expect(source("focus/model.ssrg")).toContain("struct FocusState")
    expect(source("focus/model.ssrg")).toContain("type FocusAction")
    expect(source("focus/model.ssrg")).toContain("signals.make")
    expect(source("notes/model.ssrg")).toContain("struct NotesState")
    expect(source("notes/model.ssrg")).toContain("type NotesAction")
    expect(source("notes/model.ssrg")).toContain("signals.make")
    expect(source("notes/form.ssrg")).toContain('id: "story-draft"')
    expect(source("notes/view.ssrg")).toContain("fn emptyState")
    expect(source("ui/styles.ssrg")).toContain("pub fn cx")
    expect(source("ui/components.ssrg")).toContain("pub fn metric")
  })

  test("covers every #189 state transition, including app composition and cleanup", () => {
    const stepIds = new Set(fixture.steps.map(({ id }) => id))
    expect([...stepIds]).toEqual([...fixture.requiredOperations])
    expect(
      new Set(
        fixture.steps.flatMap(({ actions }) => actions.map(({ kind }) => kind))
      )
    ).toEqual(
      new Set(["click", "submit", "input", "remove", "clear", "cleanup"])
    )

    expect(fixture.steps.map(({ id }) => id)).toEqual([
      "initial",
      "focus-interaction",
      "invalid-submit",
      "valid-submit",
      "combined-summary",
      "item-add",
      "inline-edit",
      "item-remove",
      "empty-state",
      "studio-toggle",
      "cleanup-resources",
    ])
    expect(
      fixture.steps.find(({ id }) => id === "combined-summary")?.assert
    ).toMatchObject({
      focusMetric: "3",
      storiesMetric: "3",
    })
    expect(
      fixture.steps.find(({ id }) => id === "empty-state")?.assert
    ).toMatchObject({
      storyCards: 0,
      emptyHeading: "The deck is clear.",
    })
    expect(
      fixture.steps.find(({ id }) => id === "studio-toggle")?.assert
    ).toMatchObject({
      studio: "Day studio",
      shellOwnedVisualState: true,
    })
  })

  test("records one mounted preview resource and zero after cleanup", () => {
    expect(fixture.runner.cleanup).toMatchObject({
      mountedResourceCount: 1,
      resourceCountAfterCleanup: 0,
    })
    expect(
      fixture.steps.find(({ id }) => id === "cleanup-resources")?.assert
    ).toMatchObject({ resourceCount: 0, iframeMounted: false })
  })

  test("keeps Explorer, Code, and Preview artifacts for all requested viewports", async () => {
    expect(fixture.viewports.map(({ id }) => id)).toEqual([
      "desktop",
      "iphone-13",
      "small-android",
    ])
    for (const viewport of fixture.viewports) {
      expect(viewport.previewWidth).toBeGreaterThan(0)
      expect(viewport.previewHeight).toBeGreaterThan(0)
      expect(viewport.browserViewport).toBe("1710x1112")
      expect(viewport.horizontalOverflow).toBe(false)
      expect(viewport.surfaces).toContain("preview")
      for (const artifact of [
        ...(viewport.artifact === undefined ? [] : [viewport.artifact]),
        ...(viewport.artifacts ?? []),
      ]) {
        await assertPngArtifact(artifact)
      }
    }
    expect(fixture.viewports[0]?.surfaces).toEqual([
      "explorer",
      "code-editor",
      "preview",
    ])
  })
})
