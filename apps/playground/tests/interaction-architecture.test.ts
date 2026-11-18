import { describe, expect, test } from "bun:test"

const root = new URL("..", import.meta.url)

describe("Playground and Tour interaction architecture", () => {
  test("keeps Playground global actions separate from workspace and editor actions", async () => {
    const html = await Bun.file(new URL("index.html", root)).text()
    const header = html.slice(
      html.indexOf('<header class="pg-top">'),
      html.indexOf("</header>")
    )
    const workspaceChrome = html.slice(
      html.indexOf('class="ui-card-head pg-code-head"'),
      html.indexOf('id="code-workspace"')
    )
    const flow = html.slice(
      html.indexOf('<div class="pg-flow">'),
      html.indexOf('id="io-panel"')
    )

    // 上のバーは、画面の切り替え・Search・テーマ・More だけ。Run は流れの上に置く
    expect(header).toContain('class="ui-view-switch pg-views"')
    expect(header).toContain('href="./tour/"')
    expect(header).toContain('id="search-button"')
    expect(header).toContain('id="theme-button"')
    expect(header).toContain('id="mobile-tools-button"')
    expect(header).not.toContain('id="run-button"')
    expect(flow).toContain('id="run-button"')
    expect(flow).toContain('id="workspace-resizer"')
    expect(header).not.toContain('id="sample-browser-button"')
    expect(header).not.toContain('id="explorer-toggle-button"')
    expect(header).not.toContain('id="format-source-button"')
    expect(header).not.toContain("Show indentation")
    expect(workspaceChrome).toContain('id="sample-browser-button"')
    expect(workspaceChrome).toContain('id="explorer-toggle-button"')
    expect(workspaceChrome).toContain('id="format-source-button"')
  })

  test("keeps Tour pane and editor controls local while preserving global Run", async () => {
    const html = await Bun.file(new URL("tour/index.html", root)).text()
    const header = html.slice(
      html.indexOf('<header class="pg-top tour-topbar"'),
      html.indexOf("</header>")
    )
    const codeHead = html.slice(
      html.indexOf('class="ui-card-head tour-code-head"'),
      html.indexOf('id="tour-editor"')
    )
    const flow = html.slice(
      html.indexOf('class="tour-flow"'),
      html.indexOf('id="tour-output-section"')
    )

    // 上のバーは Playground と同じ。Run はコードと出力の間の流れの上に置く
    expect(header).toContain('class="ui-view-switch pg-views"')
    expect(header).toContain('href="../"')
    expect(header).toContain('id="tour-search-button"')
    expect(header).toContain('id="tour-theme-button"')
    expect(header).toContain('id="tour-tools-button"')
    expect(header).not.toContain('id="tour-run-button"')
    expect(header).not.toContain('id="tour-format-button"')
    expect(flow).toContain('id="tour-run-button"')
    expect(flow).toContain('id="tour-output-resizer"')
    expect(codeHead).toContain('id="tour-format-button"')
    expect(codeHead).toContain('id="tour-output-pane-toggle"')
    expect(html).toContain('id="tour-fullscreen-button"')
  })
  test("uses one shared Settings model and responsive surface", async () => {
    const playground = await Bun.file(new URL("src/main.ts", root)).text()
    const tour = await Bun.file(new URL("src/tour/main.ts", root)).text()
    const settings = await Bun.file(
      new URL("src/ui/editor-settings.ts", root)
    ).text()
    const styles = await Bun.file(new URL("src/ui/playground.css", root)).text()

    expect(playground).toContain("createEditorPreferencesStore(localStorage")
    expect(tour).toContain("createEditorPreferencesStore(localStorage")
    expect(playground).toContain("returnFocus: searchButton")
    expect(tour).toContain("returnFocus: searchButton")
    expect(settings).toContain("Show indentation whitespace")
    expect(settings).toContain("Line width")
    expect(settings).toContain("returnFocus.focus()")
    expect(styles).toContain(".editor-settings-dialog")
    expect(styles).toMatch(
      /@media \(max-width: 760px\)[\s\S]*?\.editor-settings-dialog \{[\s\S]*?margin: auto 0 0;/
    )
  })
})
