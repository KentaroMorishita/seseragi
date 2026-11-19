import { describe, expect, test } from "bun:test"
import type { RunTiming } from "../src/runtime/run-timing"
import { renderRunStatus, runStatusView } from "../src/ui/run-status"
import { themeButtonLabel, themeToggleTarget } from "../src/ui/theme-button"

const root = new URL("..", import.meta.url)
const html = await Bun.file(new URL("index.html", root)).text()
const styles = await Bun.file(new URL("src/ui/playground.css", root)).text()
const tokens = await Bun.file(new URL("src/ui/design/tokens.css", root)).text()
const main = await Bun.file(new URL("src/main.ts", root)).text()

describe("Playground v2 shell", () => {
  test("is built on the shared design layer instead of the old stylesheet", () => {
    expect(html).toContain('<body class="ui-app">')
    expect(main).toContain('import "./ui/design/index.css"')
    expect(main).toContain('import "./ui/playground.css"')
    expect(main).not.toContain('import "./styles.css"')
  })

  test("keeps the top bar to the view switch, Search, theme and More", () => {
    const header = html.slice(
      html.indexOf('<header class="pg-top">'),
      html.indexOf("</header>")
    )
    expect(header).toContain('aria-current="page"><svg')
    expect(header).toContain("<span>Playground</span>")
    expect(header).toContain("<span>Tour</span>")
    expect(header).toContain("Examples, lessons, API…")
    expect(header).toContain(">⌘K</kbd>")
    expect(header).toContain('id="theme-button"')
    expect(header).not.toContain("surface-switcher")
  })

  test("puts Run on the flow between code and output", () => {
    const code = html.indexOf('class="ui-card pg-code-card"')
    const flow = html.indexOf('<div class="pg-flow">')
    const output = html.indexOf('id="io-panel"')
    expect(code).toBeGreaterThan(-1)
    expect(flow).toBeGreaterThan(code)
    expect(output).toBeGreaterThan(flow)
    expect(html.slice(flow, output)).toContain('class="ui-run-button"')
  })

  test("uses English labels and Japanese guidance", () => {
    for (const label of [
      'aria-label="Run"',
      ">Run <kbd",
      'aria-label="Files"',
      'aria-label="More"',
      ">Console</button>",
      ">Preview</button>",
      ">Input</button>",
      '<h2 class="ui-card-title">Output</h2>',
      '<span class="pg-label">Guide</span>',
      '<span class="pg-label">Format</span>',
      "Saved in this browser",
      ">Docs</a>",
    ])
      expect(html).toContain(label)
    expect(html).toContain("Run を押すと、結果がここに出ます。")
  })

  test("only uses design tokens for colors", () => {
    const withoutComments = styles.replace(/\/\*[\s\S]*?\*\//gu, "")
    expect(withoutComments).not.toMatch(/#[0-9a-f]{3,8}\b/iu)
    expect(withoutComments).not.toMatch(/\b(?:rgba?|hsla?)\(/iu)
    const defined = new Set(
      [...`${tokens}\n${styles}`.matchAll(/(--[a-z0-9-]+)\s*:/gu)].map(
        (match) => match[1]!
      )
    )
    const runtime = new Set([
      "--editor-panel-ratio",
      "--stdin-panel-ratio",
      "--explorer-width",
      "--tree-level",
      "--run-timing-share",
      "--safe-area-top",
      "--safe-area-right",
      "--safe-area-bottom",
      "--safe-area-left",
    ])
    for (const match of withoutComments.matchAll(/var\((--[a-z0-9-]+)/gu)) {
      const name = match[1]!
      if (!defined.has(name) && !runtime.has(name))
        throw new Error(`unknown custom property ${name}`)
    }
  })

  test("bridges the editor theme to the syntax tokens", () => {
    for (const bridge of [
      "--seseragi-syntax-keyword: var(--syn-keyword);",
      "--seseragi-syntax-string: var(--syn-string);",
      "--cm-background: var(--surface);",
      "--cm-font-family: var(--font-code);",
    ])
      expect(styles).toContain(bridge)
  })
})

describe("run status chip", () => {
  test("shows the outcome with text, not only color", () => {
    expect(runStatusView({ kind: "idle" })).toEqual({
      hidden: true,
      text: "",
      tone: "neutral",
      details: false,
    })
    expect(runStatusView({ kind: "running" }).text).toBe("Running…")
    expect(runStatusView({ kind: "completed", timing: timing(17.6) })).toEqual({
      hidden: false,
      text: "✓ 18 ms",
      tone: "ok",
      details: true,
    })
    // 1 ms に満たない main は 0 ms と出さない
    expect(runStatusView({ kind: "completed", timing: timing(0.4) }).text).toBe(
      "✓ <1 ms"
    )
    // 時間が分からないときは、数字を出さず、内訳も開かない
    expect(runStatusView({ kind: "completed" })).toEqual({
      hidden: false,
      text: "✓ Completed",
      tone: "ok",
      details: false,
    })
    expect(runStatusView({ kind: "interactive" }).text).toBe("✓ Interactive")
    expect(runStatusView({ kind: "diagnostics", count: 1 })).toEqual({
      hidden: false,
      text: "1 error",
      tone: "error",
      details: false,
    })
    expect(runStatusView({ kind: "diagnostics", count: 3 }).text).toBe(
      "3 errors"
    )
    expect(runStatusView({ kind: "failed" }).tone).toBe("error")
  })

  test("renders into the chip element", () => {
    const classes = new Set<string>()
    const element = {
      hidden: true,
      textContent: "",
      classList: {
        toggle: (name: string, on: boolean) =>
          on ? classes.add(name) : classes.delete(name),
      },
    } as unknown as HTMLElement
    renderRunStatus(element, { kind: "diagnostics", count: 2 })
    expect(element.hidden).toBe(false)
    expect(element.textContent).toBe("2 errors")
    expect([...classes]).toEqual(["ui-status--error"])
    renderRunStatus(element, { kind: "completed" })
    expect([...classes]).toEqual(["ui-status--ok"])
    renderRunStatus(element, { kind: "idle" })
    expect(element.hidden).toBe(true)
    expect(classes.size).toBe(0)
  })

  test("is cleared when a run in progress is cancelled", () => {
    const cancel = main.slice(
      main.indexOf("function cancelActiveExecution(): void {"),
      main.indexOf("function showExecutionOutput(")
    )
    expect(cancel).toContain(
      "const wasActive = runButton.disabled || activeExecution !== undefined"
    )
    expect(cancel).toContain('if (wasActive) showRunStatus({ kind: "idle" })')
  })

  test("is updated by Run, Clear and switching examples", () => {
    for (const outcome of [
      '{ kind: "running" }',
      'kind: "diagnostics"',
      'kind: "completed"',
      '{ kind: "interactive" }',
      '{ kind: "failed" }',
      '{ kind: "idle" }',
    ])
      expect(main).toContain(outcome)
    expect(html).toContain(
      'id="run-status" class="ui-status" role="status" aria-live="polite"'
    )
  })
})

describe("theme button", () => {
  test("switches to the opposite of the theme on screen and says so", () => {
    expect(themeToggleTarget({ preference: "system", theme: "light" })).toBe(
      "dark"
    )
    expect(themeToggleTarget({ preference: "system", theme: "dark" })).toBe(
      "light"
    )
    expect(themeToggleTarget({ preference: "light", theme: "light" })).toBe(
      "dark"
    )
    expect(themeButtonLabel({ preference: "dark", theme: "dark" })).toBe(
      "Switch to light theme"
    )
  })

  test("offers a way back to the system theme in More", () => {
    expect(html).toContain(
      '<button id="theme-system-button" type="button" role="menuitem" hidden>Use system theme</button>'
    )
    expect(main).toContain("themeSystemButton")
  })
})

describe("mobile output sheet dragging", () => {
  const heights = { closed: 64, code: 196, io: 590 }

  test("moves to the next stop in the direction of a deliberate drag", async () => {
    const { sheetSnapTarget } = await import("../src/ui/sheet-drag")
    expect(sheetSnapTarget({ heights, deltaY: 60, height: 124 })).toBe("code")
    expect(sheetSnapTarget({ heights, deltaY: 50, height: 246 })).toBe("io")
    expect(sheetSnapTarget({ heights, deltaY: -50, height: 540 })).toBe("code")
    expect(sheetSnapTarget({ heights, deltaY: -50, height: 146 })).toBe(
      "closed"
    )
  })

  test("settles a short drag on the nearest stop", async () => {
    const { sheetSnapTarget } = await import("../src/ui/sheet-drag")
    expect(sheetSnapTarget({ heights, deltaY: 20, height: 420 })).toBe("io")
    expect(sheetSnapTarget({ heights, deltaY: -20, height: 210 })).toBe("code")
    expect(sheetSnapTarget({ heights, deltaY: -20, height: 90 })).toBe("closed")
  })

  test("keeps the sheet between its closed and maximum heights", async () => {
    const { clampSheetHeight } = await import("../src/ui/sheet-drag")
    expect(clampSheetHeight(20, 64, 760)).toBe(64)
    expect(clampSheetHeight(400, 64, 760)).toBe(400)
    expect(clampSheetHeight(900, 64, 760)).toBe(760)
  })

  test("can fold down to the handle and heading so the editor gets the room", () => {
    expect(styles).toContain("--pg-sheet-closed: 64px;")
    expect(styles).toMatch(
      /\.workspace\[data-mobile-panel="closed"\] \.pg-code-card \{\s*height: calc\(100% - var\(--pg-sheet-closed\) \+ var\(--radius-lg\)\);/
    )
    expect(main).toContain("closed: 64,")
  })

  test("is wired to the handle and the output heading", () => {
    expect(main).toContain("connectSheetDrag({")
    expect(main).toContain('grips: [requiredElement(".pg-output-head"')
    expect(styles).toMatch(
      /\.pg-sheet-handle,\s*\.pg-output-head \{\s*touch-action: none;\s*user-select: none;/
    )
  })
})

function timing(mainMilliseconds: number): RunTiming {
  return {
    mainMilliseconds,
    totalMilliseconds: mainMilliseconds,
    phases: [{ id: "main", label: "main", milliseconds: mainMilliseconds }],
  }
}
