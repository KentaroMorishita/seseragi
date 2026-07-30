import { describe, expect, test } from "bun:test"
import {
  beginExclusiveResize,
  finishExclusiveResize,
  ownsExclusiveResize,
} from "../src/ui/resize-coordinator"
import {
  nextWorkspaceFocusRegion,
  type WorkspaceFocusRegion,
} from "../src/workspace/focus-navigation"

describe("Playground workspace integrated UX", () => {
  test("cycles visible workspace regions in both keyboard directions", () => {
    const regions: readonly WorkspaceFocusRegion[] = [
      "explorer",
      "tabs",
      "editor",
      "io",
    ]

    expect(nextWorkspaceFocusRegion(regions, undefined, false)).toBe("explorer")
    expect(nextWorkspaceFocusRegion(regions, undefined, true)).toBe("io")
    expect(nextWorkspaceFocusRegion(regions, "explorer", true)).toBe("io")
    expect(nextWorkspaceFocusRegion(regions, "io", false)).toBe("explorer")
    expect(nextWorkspaceFocusRegion(["editor", "io"], "editor", false)).toBe(
      "io"
    )
    expect(nextWorkspaceFocusRegion([], undefined, false)).toBeUndefined()
  })

  test("allows only one nested panel resizer to own a pointer", () => {
    const explorer = resizeHandle()
    const workspace = resizeHandle()

    expect(beginExclusiveResize(explorer, 7)).toBe(true)
    expect(explorer.dataset.dragging).toBe("true")
    expect(ownsExclusiveResize(explorer, 7)).toBe(true)
    expect(beginExclusiveResize(workspace, 9)).toBe(false)
    expect(ownsExclusiveResize(workspace, 9)).toBe(false)
    expect(finishExclusiveResize(workspace, 9)).toBe(false)
    expect(finishExclusiveResize(explorer, 7)).toBe(true)
    expect(explorer.dataset.dragging).toBeUndefined()
    expect(beginExclusiveResize(workspace, 9)).toBe(true)
    expect(finishExclusiveResize(workspace, 9)).toBe(true)
  })

  test("exposes stable browser selectors, empty states and focus shortcuts", async () => {
    const [html, main, explorer, tabs, focus, cards] = await Promise.all([
      Bun.file(new URL("../index.html", import.meta.url)).text(),
      Bun.file(new URL("../src/main.ts", import.meta.url)).text(),
      Bun.file(new URL("../src/workspace/explorer.ts", import.meta.url)).text(),
      Bun.file(
        new URL("../src/workspace/editor-tabs.ts", import.meta.url)
      ).text(),
      Bun.file(
        new URL("../src/workspace/focus-navigation.ts", import.meta.url)
      ).text(),
      Bun.file(
        new URL("../src/diagnostics/diagnostic-cards.ts", import.meta.url)
      ).text(),
    ])

    for (const selector of [
      "workspace-shell",
      "workspace-explorer",
      "workspace-tree",
      "workspace-tabs",
      "workspace-editor",
      "workspace-io",
      "workspace-output",
      "workspace-empty-state",
    ]) {
      expect(html).toContain(`data-testid="${selector}"`)
    }
    expect(html).toContain('id="workspace-notice"')
    expect(html).toContain('role="tabpanel"')
    expect(html).toContain('aria-keyshortcuts="Control+Shift+E Meta+Shift+E"')
    expect(main).toContain("setEditorEditable(editor, hasActiveFile)")
    expect(main).toContain("workspaceState.entryFile === undefined")
    expect(main).toContain("connectWorkspaceFocusNavigation(")
    expect(explorer).toContain('element.dataset.testid = "workspace-tree-item"')
    expect(explorer).toContain(
      'element.setAttribute("aria-selected", String(isSelected))'
    )
    expect(explorer).toContain('element.setAttribute("aria-current", "page")')
    expect(tabs).toContain('wrapper.dataset.testid = "workspace-tab"')
    expect(tabs).toContain('"aria-labelledby"')
    expect(focus).toContain('event.key !== "F6"')
    expect(cards).toContain('card.dataset.testid = "workspace-diagnostic"')
    expect(cards).toContain("card.dataset.diagnosticPath = path")
  })
})

function resizeHandle() {
  const captures = new Set<number>()
  return {
    dataset: {} as DOMStringMap,
    setPointerCapture(pointerId: number) {
      captures.add(pointerId)
    },
    hasPointerCapture(pointerId: number) {
      return captures.has(pointerId)
    },
    releasePointerCapture(pointerId: number) {
      captures.delete(pointerId)
    },
  }
}
