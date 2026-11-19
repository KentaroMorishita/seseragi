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
    expect(layout).toContain('window.matchMedia("(min-width: 761px)")')
    expect(layout).toContain("onLayoutChange()")
    // 目次はコースマップとして重ねて開くので、目次の列の幅は持たない
    expect(layout).not.toContain("navigation")
  })

  test("exposes two separators and an accessible Output toggle", async () => {
    const html = await Bun.file(new URL("tour/index.html", root)).text()

    for (const id of ["tour-lesson-resizer", "tour-output-resizer"]) {
      expect(html).toContain(`id="${id}"`)
    }
    expect(html.match(/role="separator"/g)).toHaveLength(2)
    expect(html).not.toContain('id="tour-navigation-resizer"')
    expect(html).toContain('id="tour-output-pane-toggle"')
    expect(html).toContain('aria-label="Outputを閉じる"')
  })

  test("keeps the code, Run and Output in one column with a collapsible Output", async () => {
    const styles = await Bun.file(new URL("src/tour/tour.css", root)).text()

    expect(styles).toContain('.tour-lab[data-output-collapsed="true"]')
    expect(styles).toMatch(
      /\.tour-lesson-resizer \{[\s\S]*?cursor: col-resize;/
    )
    expect(styles).toMatch(/\.tour-flow-line \{[\s\S]*?cursor: row-resize;/)
  })
})
