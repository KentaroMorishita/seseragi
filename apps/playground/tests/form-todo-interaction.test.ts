import { describe, expect, test } from "bun:test"
import { samples } from "../src/samples"

type InteractionAction = Readonly<{
  readonly kind: string
  readonly selector?: string
  readonly buttonText?: string
  readonly cardId?: string
  readonly filter?: string
  readonly key?: string
  readonly pointerType?: string
  readonly value?: string
  readonly checked?: boolean
  readonly repeat?: number
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
    new URL("./fixtures/form-todo.interaction.json", import.meta.url)
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

describe("form-todo browser interaction fixture", () => {
  test("keeps the recorded fixture tied to the generated advanced sample", () => {
    const sample = samples.find(({ id }) => id === fixture.sampleId)
    expect(sample).toBeDefined()
    if (sample === undefined) return

    expect(fixture.schema).toBe(1)
    expect(fixture.sourcePath).toBe(sample.sourcePath)
    expect(sample.interactive).toBe(true)
    expect(sample.sourceHash).toBe(fixture.manifest.sourceHash)
    expect(sample.workspaceHash).toBe(fixture.manifest.workspaceHash)
    expect(fixture.runner.kind).toBe("browser-use")
    expect(fixture.runner.previewSelector).toBe("#html-preview")

    for (const marker of [
      "signals.make initialModel",
      "dom.run (dom.defaultOptions ())",
      "onSubmit: dispatch state Submitted",
      "onPointerDown: pointerTask state",
      "onKeyDown: filterKeyTask state",
      'role: "alert"',
      'role: "status"',
      "ToggleComplete",
      "TogglePinned",
      "ClearCompleted",
      "fn emptyState",
      "fn planCard",
    ]) {
      expect(sample.source).toContain(marker)
    }
    expect(sample.source).not.toContain("html.table")
  })

  test("covers every #188 interaction state and action family", () => {
    const stepIds = new Set(fixture.steps.map(({ id }) => id))
    for (const operation of fixture.requiredOperations) {
      expect(
        stepIds.has(operation) ||
          [...stepIds].some((id) => id.startsWith(`${operation}-`))
      ).toBe(true)
    }

    const actionKinds = new Set(
      fixture.steps.flatMap(({ actions }) => actions.map(({ kind }) => kind))
    )
    expect(actionKinds).toEqual(
      new Set([
        "input",
        "submit",
        "click",
        "change",
        "remove",
        "filter",
        "keyboard",
        "pointer",
        "clear",
      ])
    )

    expect(fixture.steps.map(({ id }) => id)).toEqual([
      "initial",
      "invalid-submit",
      "valid-submit",
      "item-add",
      "inline-edit",
      "complete",
      "pin",
      "remove",
      "filter-done",
      "keyboard-arrow-right",
      "keyboard-end",
      "pointer",
      "clear-completed",
      "empty-state",
    ])
    expect(fixture.steps.at(-1)?.assert).toMatchObject({
      planCount: 0,
      emptyHeading: "Your launch loop is clear.",
    })
  })

  test("keeps code and Preview artifacts for all requested viewport contracts", async () => {
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
      expect(viewport.surfaces).toEqual(["code-editor", "preview"])
      for (const artifact of [
        ...(viewport.artifact === undefined ? [] : [viewport.artifact]),
        ...(viewport.artifacts ?? []),
      ]) {
        await assertPngArtifact(artifact)
      }
    }
  })
})
