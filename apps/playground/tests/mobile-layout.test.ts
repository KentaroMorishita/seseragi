import { describe, expect, test } from "bun:test"

const root = new URL("..", import.meta.url)

describe("mobile editing layout contract", () => {
  test("keeps every focused text surface at the iPhone-safe 16px size", async () => {
    const theme = await Bun.file(new URL("src/editor/theme.ts", root)).text()
    const styles = await Bun.file(new URL("src/ui/playground.css", root)).text()
    const tourStyles = await Bun.file(
      new URL("src/ui/playground.css", root)
    ).text()

    // エディターは既定で 16px。Playground は狭い画面で 16px に戻す
    expect(theme).toContain('fontSize: "var(--cm-font-size, 16px)"')
    expect(styles).toMatch(
      /@media \(max-width: 760px\)[^{]*\{\s*\.editor-host \{\s*--cm-font-size: 16px;/
    )
    expect(styles).toMatch(
      /\.ui-app input,\s*\.ui-app select,\s*\.ui-app textarea \{\s*font-size: 16px;/
    )
    // Search の入力欄も、狭い画面では 16px にする
    expect(styles).toMatch(
      /@media \(max-width: 760px\)[^{]*\{[\s\S]*?\.pg-search-field input \{\s*font-size: 16px;/
    )
    expect(tourStyles).toMatch(/textarea \{[\s\S]*?font-size: 16px;/)
  })

  test("does not disable browser zoom to work around focused controls", async () => {
    const html = await Bun.file(new URL("index.html", root)).text()

    expect(html).not.toContain("user-scalable=no")
    expect(html).not.toContain("maximum-scale=1")
  })

  test("uses a dedicated compact CodeMirror layout on small screens", async () => {
    const styles = await Bun.file(new URL("src/ui/playground.css", root)).text()

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
    const styles = await Bun.file(new URL("src/ui/playground.css", root)).text()

    expect(styles).toContain(
      "@media (orientation: landscape) and (max-width: 960px) and (max-height: 520px)"
    )
    // v2 は横向きでも、出力を下からのシートに重ねる（下のタブの列は持たない）
    expect(styles).not.toContain(".mobile-tabs")
    expect(styles).toContain("--safe-area-left: env(safe-area-inset-left)")
  })

  test("keeps surface switching, Search and overflow in one mobile topbar row with a floating Run", async () => {
    const html = await Bun.file(new URL("index.html", root)).text()
    const styles = await Bun.file(new URL("src/ui/playground.css", root)).text()

    expect(html).toContain('id="mobile-tools-button"')
    expect(html).toContain('id="mobile-tools-menu"')
    expect(html).toContain('class="ui-view-switch pg-views"')
    expect(html).toContain('id="search-button"')
    expect(html).toContain('role="menuitem"')
    expect(styles).toMatch(/\.pg-top \{[\s\S]*?z-index: 30;/)
    expect(styles).toMatch(
      /\.pg-menu \{[\s\S]*?top: calc\(100% \+ var\(--space-2\)\);/
    )
    // 狭い画面では Run が右下に浮かぶ
    expect(styles).toMatch(/\.pg-flow \{\s*position: fixed;/)
    expect(html.indexOf('id="sample-browser-button"')).toBeGreaterThan(
      html.indexOf('class="ui-card-head pg-code-head"')
    )
  })

  test("shows only the canonical brand icon in mobile headers", async () => {
    const tour = await Bun.file(new URL("tour/index.html", root)).text()
    const playground = await Bun.file(new URL("index.html", root)).text()

    // Tour も Playground と同じ上のバー（ロゴの画像）を使う
    for (const html of [tour, playground]) {
      expect(html).toContain('<img src="/brand/seseragi-icon.svg"')
      expect(html).not.toContain('content: "S"')
    }
  })
  test("shows Search as a bottom sheet without internal category names", async () => {
    const palette = await Bun.file(
      new URL("src/ui/search-palette.ts", root)
    ).text()
    const index = await Bun.file(new URL("src/ui/search-index.ts", root)).text()
    const styles = await Bun.file(new URL("src/ui/playground.css", root)).text()

    // 例は、内部の分類名ではなくグループ名（日本語）の見出しで並べる
    expect(index).toContain("section: group.title")
    for (const internal of ["Recipe", "Showcase", "dom.app", "Multi-module"]) {
      expect(palette).not.toContain(internal)
      expect(index).not.toContain(internal)
    }
    expect(styles).toMatch(
      /@media \(max-width: 760px\)[^{]*\{\s*\.pg-search-dialog \{[\s\S]*?margin: auto 0 0;/
    )
  })
  test("switches to I/O after Run in portrait and compact landscape", async () => {
    const main = await Bun.file(new URL("src/main.ts", root)).text()

    expect(main).toContain(
      '"(max-width: 760px), (max-width: 960px) and (max-height: 520px)"'
    )
    expect(main).toContain('mobilePanels.show("code")')
    expect(main).toContain('mobilePanels.show("io")')
    const html = await Bun.file(new URL("index.html", root)).text()
    const styles = await Bun.file(new URL("src/ui/playground.css", root)).text()
    // 出力は下から顔を出すシート。引き上げると画面の 7 割まで広がる
    expect(html).toContain("data-panel-toggle")
    expect(styles).toMatch(
      /\.workspace\[data-mobile-panel="io"\] \.pg-output-card \{\s*height: 70dvh;/
    )
  })

  test("removes the inactive mobile panel root from layout", async () => {
    const html = await Bun.file(new URL("index.html", root)).text()
    const styles = await Bun.file(new URL("src/ui/playground.css", root)).text()
    const panels = await Bun.file(
      new URL("src/ui/mobile-panels.ts", root)
    ).text()

    // v2 はコードを残したまま、出力のシートの高さだけを変える
    expect(styles).toMatch(
      /\.workspace\[data-mobile-panel="io"\] \.pg-output-card \{\s*height: 70dvh;/
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
      new URL("src/ui/reference-card.ts", root)
    ).text()
    const styles = await Bun.file(new URL("src/ui/playground.css", root)).text()
    const palette = await Bun.file(
      new URL("src/editor/syntax-theme.css", root)
    ).text()
    const theme = await Bun.file(new URL("src/editor/theme.ts", root)).text()

    expect(reference).toContain("highlightSeseragi(item.signature)")
    expect(reference).toContain('signature.className = "seseragi-highlight"')
    expect(styles).toContain("--seseragi-syntax-keyword: var(--syn-keyword)")
    expect(styles).toContain(".seseragi-highlight .tok-keyword")
    expect(styles).toContain(".seseragi-highlight .tok-typeName")
    expect(styles).toContain(".seseragi-highlight .tok-standardType")
    expect(styles).toContain(".seseragi-highlight .tok-operator")
    expect(styles).toContain("var(--seseragi-syntax-keyword)")
    expect(theme).toContain('keyword: "var(--seseragi-syntax-keyword)"')
    expect(palette).toContain("--seseragi-syntax-keyword: #c7a0f7")
    expect(palette).toContain("--seseragi-syntax-type-name: #f1b37a")
  })

  test("moves Input beside Output and keeps its expanded state accessible", async () => {
    const html = await Bun.file(new URL("index.html", root)).text()
    const main = await Bun.file(new URL("src/main.ts", root)).text()
    const outputHeading = html.indexOf('class="ui-card-head pg-output-head"')
    const inputToggle = html.indexOf('id="stdin-toggle-button"')

    expect(outputHeading).toBeGreaterThan(-1)
    expect(inputToggle).toBeGreaterThan(outputHeading)
    expect(html).toContain(">Input</button>")
    expect(html).not.toContain("<span>Stdin</span>")
    expect(html).toContain('aria-expanded="false"')
    expect(main).toContain('setAttribute("aria-expanded", String(visible))')
  })

  test("uses one bounded highlighted surface for mouse and touch analysis", async () => {
    const editor = await Bun.file(
      new URL("src/editor/create-editor.ts", root)
    ).text()
    const styles = await Bun.file(new URL("src/ui/playground.css", root)).text()

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
    expect(html).toContain('class="ui-card-head pg-code-head"')
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
