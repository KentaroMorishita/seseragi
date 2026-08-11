import { describe, expect, test } from "bun:test"

const root = new URL("..", import.meta.url)

describe("Tour desktop pane layout", () => {
  test("reuses shared ratio clamping and exclusive pointer ownership", async () => {
    const layout = await Bun.file(
      new URL("src/tour/pane-layout.ts", root)
    ).text()

    expect(layout).toContain("clampPanelRatio(")
    expect(layout).toContain("readPanelRatio(")
    expect(layout).toContain("beginExclusiveResize(")
    expect(layout).toContain("ownsExclusiveResize(")
    expect(layout).toContain("finishExclusiveResize(")
    expect(layout).toContain('window.matchMedia("(min-width: 1181px)")')
    expect(layout).toContain("onLayoutChange()")
  })

  test("exposes three separators and two compact accessible toggles", async () => {
    const html = await Bun.file(new URL("tour/index.html", root)).text()

    for (const id of [
      "tour-navigation-resizer",
      "tour-lesson-resizer",
      "tour-output-resizer",
    ]) {
      expect(html).toContain(`id="${id}"`)
      expect(html).toContain('role="separator"')
    }
    expect(html).toContain('id="tour-navigation-pane-toggle"')
    expect(html).toContain('aria-label="lesson一覧を閉じる"')
    expect(html).toContain('id="tour-output-pane-toggle"')
    expect(html).toContain('aria-label="Outputを閉じる"')
  })

  test("keeps desktop collapse state out of the narrow layout", async () => {
    const styles = await Bun.file(new URL("src/tour/styles.css", root)).text()

    expect(styles).toContain("@media (min-width: 1181px)")
    expect(styles).toContain(
      '.tour-workspace[data-navigation-collapsed="true"]'
    )
    expect(styles).toContain('.tour-lab[data-output-collapsed="true"]')
    expect(styles).toMatch(
      /@media \(max-width: 1180px\)[\s\S]*?\.tour-pane-toggle,[\s\S]*?display: none;/
    )
    expect(styles).toMatch(
      /\.tour-pane-resizer--vertical \{[\s\S]*?cursor: col-resize;/
    )
    expect(styles).toMatch(
      /\.tour-pane-resizer--horizontal \{[\s\S]*?cursor: row-resize;/
    )
  })
})
