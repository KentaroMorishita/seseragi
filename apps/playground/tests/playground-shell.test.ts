import { describe, expect, test } from "bun:test"
import { renderRunStatus, runStatusView } from "../src/ui/run-status"
import { nextThemePreference, themeButtonLabel } from "../src/ui/theme-button"

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
    })
    expect(runStatusView({ kind: "running" }).text).toBe("Running…")
    expect(runStatusView({ kind: "completed", milliseconds: 17.6 })).toEqual({
      hidden: false,
      text: "✓ 18 ms",
      tone: "ok",
    })
    expect(runStatusView({ kind: "interactive" }).text).toBe("✓ Interactive")
    expect(runStatusView({ kind: "diagnostics", count: 1 })).toEqual({
      hidden: false,
      text: "1 error",
      tone: "error",
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
    renderRunStatus(element, { kind: "completed", milliseconds: 3 })
    expect([...classes]).toEqual(["ui-status--ok"])
    renderRunStatus(element, { kind: "idle" })
    expect(element.hidden).toBe(true)
    expect(classes.size).toBe(0)
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
  test("cycles System, Light and Dark", () => {
    expect(nextThemePreference("system")).toBe("light")
    expect(nextThemePreference("light")).toBe("dark")
    expect(nextThemePreference("dark")).toBe("system")
    expect(themeButtonLabel("system")).toBe("Theme: System")
    expect(themeButtonLabel("dark")).toBe("Theme: Dark")
  })
})
