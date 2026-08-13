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

  test("wraps canonical sample source instead of requiring horizontal scroll", async () => {
    const editor = await Bun.file(
      new URL("src/editor/create-editor.ts", root)
    ).text()
    const generator = await Bun.file(
      new URL("../../scripts/generate-playground-samples.ts", root)
    ).text()

    expect(editor).toContain("EditorView.lineWrapping")
    expect(generator).toContain("validatePreviewSourceReadability")
  })

  test("moves panel navigation out of the vertical stack in landscape", async () => {
    const styles = await Bun.file(new URL("src/styles.css", root)).text()

    expect(styles).toContain(
      "@media (orientation: landscape) and (max-width: 960px) and (max-height: 520px)"
    )
    expect(styles).toMatch(
      /\.mobile-tabs \{[\s\S]*?position: absolute;[\s\S]*?left: var\(--safe-area-left\);[\s\S]*?width: 42px;/
    )
    expect(styles).toMatch(/\.workspace \{[\s\S]*?margin-left: 42px;/)
    expect(styles).toContain("grid-template-rows: 42px 0 minmax(0, 1fr) 0")
    expect(styles).toContain("--safe-area-left: env(safe-area-inset-left)")
  })

  test("keeps surface switching, overflow, and Run in one mobile topbar row", async () => {
    const html = await Bun.file(new URL("index.html", root)).text()
    const styles = await Bun.file(new URL("src/styles.css", root)).text()

    expect(html).toContain('id="mobile-tools-button"')
    expect(html).toContain('id="mobile-tools-menu"')
    expect(html).toContain('id="surface-switcher-button"')
    expect(html).toContain('class="surface-switcher-menu"')
    expect(html).toContain('role="menuitem"')
    expect(styles).toContain(".global-tools-menu:not([hidden])")
    expect(styles).toContain(".surface-switcher-menu:not([hidden])")
    expect(styles).toMatch(/\.topbar \{[\s\S]*?z-index: 30;/)
    expect(styles).toMatch(
      /\.global-tools-menu:not\(\[hidden\]\) \{[\s\S]*?top: calc\(100% \+ 7px\);/
    )
    expect(html.indexOf('id="sample-browser-button"')).toBeGreaterThan(
      html.indexOf('class="workspace-editor-chrome"')
    )
  })

  test("keeps Web catalog roles readable in one-column mobile cards", async () => {
    const browser = await Bun.file(
      new URL("src/ui/sample-browser.ts", root)
    ).text()
    const styles = await Bun.file(new URL("src/styles.css", root)).text()

    expect(browser).toContain('"Minimal"')
    expect(browser).toContain('"dom.app"')
    expect(browser).toContain('"Signal + dom.run"')
    expect(browser).toContain('"Multi-module"')
    expect(browser).toContain("sample-card-prerequisite")
    expect(browser).toContain("sample-card-comparison")
    expect(styles).toMatch(/\.sample-card-meta \{[\s\S]*?line-height: 1\.45;/)
    expect(styles).toMatch(
      /@media \(max-width: 760px\)[\s\S]*?\.sample-card-grid \{\s*grid-template-columns: 1fr;/
    )
    expect(styles).toMatch(
      /@media \(max-width: 760px\)[\s\S]*?\.sample-card \{\s*min-height: 0;/
    )
  })

  test("switches to I/O after Run in portrait and compact landscape", async () => {
    const main = await Bun.file(new URL("src/main.ts", root)).text()

    expect(main).toContain(
      '"(max-width: 760px), (max-width: 960px) and (max-height: 520px)"'
    )
    expect(main).toContain('mobilePanels.show("code")')
    expect(main).toContain("tab?.click()")
  })

  test("removes the inactive mobile panel root from layout", async () => {
    const html = await Bun.file(new URL("index.html", root)).text()
    const styles = await Bun.file(new URL("src/styles.css", root)).text()
    const panels = await Bun.file(
      new URL("src/ui/mobile-panels.ts", root)
    ).text()

    expect(styles).toMatch(
      /\.workspace\[data-mobile-panel="code"\] \.io-panel,\s*\.workspace\[data-mobile-panel="io"\] \.code-workspace \{\s*display: none;/
    )
    expect(styles).not.toContain(
      '.workspace[data-mobile-panel="io"] .editor-panel'
    )
    expect(html.indexOf('id="code-workspace"')).toBeLessThan(
      html.indexOf('id="io-panel"')
    )
    expect(panels).toContain("workspace.dataset.mobilePanel = target")
    expect(panels).not.toContain("replaceChildren")
    expect(panels).not.toContain("innerHTML")
  })

  test("syntax-highlights Reference signatures with the editor tokenizer", async () => {
    const reference = await Bun.file(
      new URL("src/ui/reference-browser.ts", root)
    ).text()
    const styles = await Bun.file(new URL("src/styles.css", root)).text()

    expect(reference).toContain("highlightSeseragi(item.signature)")
    expect(reference).toContain('signature.className = "seseragi-highlight"')
    expect(styles).toContain(".seseragi-highlight .tok-keyword")
    expect(styles).toContain(".seseragi-highlight .tok-typeName")
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

  test("moves indentation whitespace into shared Settings", async () => {
    const main = await Bun.file(new URL("src/main.ts", root)).text()
    const settings = await Bun.file(
      new URL("src/ui/editor-settings.ts", root)
    ).text()
    const preferences = await Bun.file(
      new URL("src/preferences/editor-preferences.ts", root)
    ).text()

    expect(settings).toContain("Show indentation whitespace")
    expect(preferences).toContain("seseragi.editor.preferences.v1")
    expect(main).toContain("setEditorWhitespaceVisible(editor, visible)")
  })

  test("exposes adaptive formatting in local editor chrome", async () => {
    const html = await Bun.file(new URL("index.html", root)).text()
    const main = await Bun.file(new URL("src/main.ts", root)).text()

    expect(html).toContain('id="format-source-button"')
    expect(html).toContain('class="workspace-editor-chrome"')
    expect(main).toContain("resolveEditorLineWidth(")
    expect(main).toContain("formatProjectFile(request, requestedFile, {")
    expect(main).toContain("workspaceAnalysisRevision(workspaceState)")
    expect(main).toContain('setStatus("success", "Formatted")')
    expect(main).toContain('setStatus("success", "Already formatted")')
    expect(main).toContain("Cannot format:")
    expect(main).toContain("editor.focus()")
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
