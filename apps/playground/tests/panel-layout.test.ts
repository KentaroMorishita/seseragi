import { describe, expect, test } from "bun:test"
import { clampPanelRatio, readPanelRatio } from "../src/ui/panel-layout"

describe("desktop panel layout", () => {
  test("keeps both sides above their usable minimum", () => {
    expect(clampPanelRatio(0.1, 1000, 360, 300)).toBe(0.36)
    expect(clampPanelRatio(0.95, 1000, 360, 300)).toBe(0.7)
    expect(clampPanelRatio(0.6, 1000, 360, 300)).toBe(0.6)
  })

  test("falls back when the persisted ratio is invalid", () => {
    expect(readPanelRatio({ getItem: () => "0.72" }, "layout", 0.68)).toBe(0.72)
    expect(readPanelRatio({ getItem: () => "broken" }, "layout", 0.68)).toBe(
      0.68
    )
    expect(readPanelRatio({ getItem: () => null }, "layout", 0.68)).toBe(0.68)
  })

  test("exposes accessible separators and consolidated tools", async () => {
    const html = await Bun.file(
      new URL("../index.html", import.meta.url)
    ).text()
    const styles = await Bun.file(
      new URL("../src/styles.css", import.meta.url)
    ).text()

    expect(html).toContain('id="workspace-resizer"')
    expect(html).toContain('aria-orientation="vertical"')
    expect(html).toContain('id="io-resizer"')
    expect(html).toContain('aria-orientation="horizontal"')
    expect(html).toContain('role="toolbar"')
    expect(html).toContain('id="reset-sample-button"')
    expect(html).toContain('id="stdin-toggle-button"')
    expect(styles).toMatch(/\.workspace \{[\s\S]*?grid-row: 3;/)
    expect(styles).toMatch(/footer \{[\s\S]*?grid-row: 4;/)
  })

  test("locks the shell and scrolls only inside editor and output surfaces", async () => {
    const styles = await Bun.file(
      new URL("../src/styles.css", import.meta.url)
    ).text()

    expect(styles).toMatch(
      /html,\s*body \{[\s\S]*?height: 100%;[\s\S]*?overflow: hidden;/
    )
    expect(styles).toMatch(
      /\.app-shell \{[\s\S]*?height: 100dvh;[\s\S]*?overflow: hidden;/
    )
    expect(styles).toMatch(/\.workspace \{[\s\S]*?overflow: hidden;/)
    expect(styles).toMatch(
      /\.editor-host \.cm-scroller \{[\s\S]*?overflow: auto;/
    )
    expect(styles).toMatch(/pre \{[\s\S]*?overflow: auto;/)
  })
})
