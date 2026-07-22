import { describe, expect, test } from "bun:test"

const root = new URL("..", import.meta.url)

describe("mobile editing layout contract", () => {
  test("keeps every focused text surface at the iPhone-safe 16px size", async () => {
    const theme = await Bun.file(new URL("src/editor/theme.ts", root)).text()
    const styles = await Bun.file(new URL("src/styles.css", root)).text()

    expect(theme).toContain('fontSize: "16px"')
    expect(styles).toMatch(/\.sample-browser-title \{[\s\S]*?font-size: 16px;/)
    expect(styles).toMatch(/textarea \{[\s\S]*?font-size: 16px;/)
    expect(styles).toMatch(
      /\.reference-browser-filters input,[\s\S]*?font-size: 16px;/
    )
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

  test("keeps Sample, overflow tools, and Run in one mobile topbar row", async () => {
    const html = await Bun.file(new URL("index.html", root)).text()
    const styles = await Bun.file(new URL("src/styles.css", root)).text()

    expect(html).toContain('id="mobile-tools-button"')
    expect(html).toContain('id="mobile-tools-menu"')
    expect(html).toContain('role="menuitem"')
    expect(styles).toMatch(
      /\.topbar \{[\s\S]*?grid-template-columns: auto minmax\(96px, 1fr\) auto auto;/
    )
    expect(styles).toMatch(/\.toolbar \{[\s\S]*?display: none;/)
    expect(styles).toContain(".mobile-tools-menu:not([hidden])")
  })

  test("moves Input beside Output and keeps its expanded state accessible", async () => {
    const html = await Bun.file(new URL("index.html", root)).text()
    const main = await Bun.file(new URL("src/main.ts", root)).text()
    const outputHeading = html.indexOf('class="output-heading-actions"')
    const inputToggle = html.indexOf('id="stdin-toggle-button"')

    expect(inputToggle).toBeGreaterThan(outputHeading)
    expect(html).toContain("<span>Input</span>")
    expect(html).not.toContain("<span>Stdin</span>")
    expect(html).toContain('aria-expanded="false"')
    expect(main).toContain('setAttribute("aria-expanded", String(visible))')
  })

  test("uses one bounded highlighted surface for mouse and touch analysis", async () => {
    const editor = await Bun.file(
      new URL("src/editor/create-editor.ts", root)
    ).text()
    const styles = await Bun.file(new URL("src/styles.css", root)).text()

    expect(editor).toContain('event.pointerType !== "touch"')
    expect(editor).toContain("activateHover(")
    expect(editor).toContain("visualViewportSpace(document)")
    expect(editor).toContain("highlightSeseragi(hover.title)")
    expect(styles).toContain(".cm-tooltip:has(> .analysis-hover)")
    expect(styles).toMatch(/\.analysis-hover \{[\s\S]*?max-height:/)
    expect(styles).toMatch(/\.analysis-hover-signature \.tok-keyword/)
  })

  test("exposes indentation whitespace in desktop tools and mobile overflow", async () => {
    const html = await Bun.file(new URL("index.html", root)).text()
    const main = await Bun.file(new URL("src/main.ts", root)).text()

    expect(html).toContain('id="whitespace-toggle-button"')
    expect(html).toContain('id="mobile-whitespace-button"')
    expect(html).toContain("Show indentation")
    expect(main).toContain("seseragi.playground.showWhitespace")
    expect(main).toContain("setEditorWhitespaceVisible(editor, visible)")
  })

  test("supports focus, arrow keys, outside taps, and Escape in overflow", async () => {
    const menu = await Bun.file(new URL("src/ui/overflow-menu.ts", root)).text()

    expect(menu).toContain('event.key === "ArrowDown"')
    expect(menu).toContain('event.key === "ArrowUp"')
    expect(menu).toContain('event.key !== "Escape"')
    expect(menu).toContain('ownerDocument.addEventListener("pointerdown"')
    expect(menu).toContain("?.focus()")
  })
})
