import { describe, expect, test } from "bun:test"

const root = new URL("..", import.meta.url)

describe("Playground and Tour interaction architecture", () => {
  test("keeps Playground global actions separate from workspace and editor actions", async () => {
    const html = await Bun.file(new URL("index.html", root)).text()
    const header = html.slice(
      html.indexOf('<header class="topbar">'),
      html.indexOf("</header>")
    )
    const workspaceChrome = html.slice(
      html.indexOf('class="workspace-editor-chrome"'),
      html.indexOf('id="code-workspace"')
    )

    expect(header).toContain('id="surface-switcher-button"')
    expect(header).toContain('id="mobile-tools-button"')
    expect(header).toContain('id="run-button"')
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
      html.indexOf('<header class="tour-topbar"'),
      html.indexOf("</header>")
    )
    const editorHeading = html.slice(
      html.indexOf('class="tour-editor-heading-actions"'),
      html.indexOf('id="tour-editor"')
    )

    expect(header).toContain('id="tour-surface-switcher-button"')
    expect(header).toContain('id="tour-tools-button"')
    expect(header).toContain('id="tour-run-button"')
    expect(header).not.toContain("Playgroundへ戻る")
    expect(header).not.toContain('id="tour-format-button"')
    expect(header).not.toContain('id="tour-navigation-pane-toggle"')
    expect(editorHeading).toContain('id="tour-format-button"')
    expect(html).toMatch(/class="[^"]*tour-navigation-boundary-toggle[^"]*"/)
    expect(html).toContain('id="tour-fullscreen-button"')
  })

  test("uses one shared Settings model and responsive surface", async () => {
    const playground = await Bun.file(new URL("src/main.ts", root)).text()
    const tour = await Bun.file(new URL("src/tour/main.ts", root)).text()
    const settings = await Bun.file(
      new URL("src/ui/editor-settings.ts", root)
    ).text()
    const styles = await Bun.file(new URL("src/styles.css", root)).text()

    expect(playground).toContain("createEditorPreferencesStore(localStorage")
    expect(tour).toContain("createEditorPreferencesStore(localStorage")
    expect(playground).toContain("returnFocus: mobileToolsButton")
    expect(tour).toContain("returnFocus: toolsButton")
    expect(settings).toContain("Show indentation whitespace")
    expect(settings).toContain("Line width")
    expect(settings).toContain("returnFocus.focus()")
    expect(styles).toContain(".editor-settings-dialog")
    expect(styles).toMatch(
      /@media \(max-width: 760px\)[\s\S]*?\.editor-settings-dialog \{[\s\S]*?margin: auto 0 0;/
    )
  })
})
