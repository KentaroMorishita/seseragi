import { describe, expect, test } from "bun:test"

const root = new URL("..", import.meta.url)

describe("mobile editing layout contract", () => {
  test("keeps every focused text surface at the iPhone-safe 16px size", async () => {
    const theme = await Bun.file(new URL("src/editor/theme.ts", root)).text()
    const styles = await Bun.file(new URL("src/styles.css", root)).text()

    expect(theme).toContain('fontSize: "16px"')
    expect(styles).toMatch(
      /\.sample-picker select \{[\s\S]*?font-size: 16px;[\s\S]*?\}/
    )
    expect(styles).toMatch(/textarea \{[\s\S]*?font-size: 16px;/)
  })

  test("does not disable browser zoom to work around focused controls", async () => {
    const html = await Bun.file(new URL("index.html", root)).text()

    expect(html).not.toContain("user-scalable=no")
    expect(html).not.toContain("maximum-scale=1")
  })

  test("uses a dedicated compact CodeMirror layout on small screens", async () => {
    const styles = await Bun.file(new URL("src/styles.css", root)).text()

    expect(styles).toContain(
      "@media (max-width: 760px), (max-width: 960px) and (max-height: 520px)"
    )
    expect(styles).toContain("--cm-line-height: 1.35")
    expect(styles).toContain("--cm-line-inline-padding: 7px")
    expect(styles).toContain("--cm-line-number-min-width: 26px")
    expect(styles).toMatch(
      /\.editor-host \.cm-gutters \.cm-gutter-lint \{\s*[^}]*display: none !important;/
    )
  })

  test("moves panel navigation out of the vertical stack in landscape", async () => {
    const styles = await Bun.file(new URL("src/styles.css", root)).text()

    expect(styles).toContain(
      "@media (orientation: landscape) and (max-width: 960px) and (max-height: 520px)"
    )
    expect(styles).toMatch(
      /\.mobile-tabs \{[\s\S]*?position: absolute;[\s\S]*?width: 42px;/
    )
    expect(styles).toMatch(/\.workspace \{[\s\S]*?margin-left: 42px;/)
    expect(styles).toContain("grid-template-rows: 42px 0 minmax(0, 1fr) 0")
  })
})
