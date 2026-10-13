import { readFileSync, writeFileSync } from "node:fs"
import {
  expect,
  type CDPSession,
  type Page,
  type TestInfo,
  test,
} from "@playwright/test"

// Diagnostic experiment on old source 5ab only; no expected PNG or baseline writes.
test.use({ viewport: { width: 1440, height: 1000 }, trace: "off" })

// Byte-identical mocks and routes from the old canonical web-ui-regression spec.
const mockImage = `<svg xmlns="http://www.w3.org/2000/svg" width="960" height="480" viewBox="0 0 960 480"><rect width="960" height="480" fill="#1d4ed8"/><path d="M0 362 224 180l126 96 190-164 420 250v118H0Z" fill="#34d399"/><circle cx="748" cy="118" r="54" fill="#fef3c7"/></svg>`
const mockLogo = readFileSync(
  new URL("../../../assets/brand/source/seseragi-logo-dark.svg", import.meta.url),
  "utf8"
)

async function recordPhase(page: Page, cdp: CDPSession, label: string) {
  const geometry = await page.evaluate(() => {
    const rect = (element: Element) => element.getBoundingClientRect().toJSON()
    const describe = (element: Element) => {
      const style = getComputedStyle(element)
      return {
        tag: element.tagName,
        id: element.id,
        classes: element.className,
        inlineStyle: element instanceof HTMLElement ? element.style.cssText : null,
        rect: rect(element),
        clientHeight: element.clientHeight,
        scrollHeight: element.scrollHeight,
        style: {
          fontFamily: style.fontFamily,
          fontSize: style.fontSize,
          fontWeight: style.fontWeight,
          lineHeight: style.lineHeight,
          padding: style.padding,
          minWidth: style.minWidth,
          height: style.height,
          position: style.position,
        },
      }
    }
    const required = (selector: string) => {
      const element = document.querySelector(selector)
      if (!element) throw new Error(`Missing diagnostic control node: ${selector}`)
      return element
    }
    const editor = required("#editor .cm-editor")
    // CodeMirror creates a body child with exactly its theme classes, not .cm-editor.
    const wrappers = [...document.body.children].filter(
      (element) =>
        element instanceof HTMLElement &&
        !element.contains(editor) &&
        element.classList.length > 0 &&
        [...element.classList].every((name) => editor.classList.contains(name)) &&
        getComputedStyle(element).position === "relative"
    )
    if (wrappers.length !== 1) {
      throw new Error(`Expected one body-mounted CM tooltip wrapper, got ${wrappers.length}`)
    }
    const firstLine = required("#editor .cm-content .cm-line")
    const fullRange = document.createRange()
    fullRange.selectNodeContents(firstLine)
    const walker = document.createTreeWalker(firstLine, NodeFilter.SHOW_TEXT)
    const textRanges = []
    for (let node = walker.nextNode(); node; node = walker.nextNode()) {
      const range = document.createRange()
      range.selectNodeContents(node)
      textRanges.push({
        text: node.textContent,
        rect: range.getBoundingClientRect().toJSON(),
        clientRects: [...range.getClientRects()].map((value) => value.toJSON()),
        parent: describe(node.parentElement ?? firstLine),
      })
    }
    const gutterCells = [...document.querySelectorAll("#editor .cm-lineNumbers .cm-gutterElement")]
    const firstCell = gutterCells.find((element) => element.getBoundingClientRect().height > 0)
    if (!firstCell) throw new Error("Missing positive-height first gutter cell")
    return {
      viewport: { width: innerWidth, height: innerHeight, dpr: devicePixelRatio, scrollX, scrollY },
      fontsStatus: document.fonts.status,
      statusText: required("#status-text").textContent,
      body: describe(document.body),
      html: describe(document.documentElement),
      host: describe(required("#editor")),
      editor: describe(editor),
      scroller: describe(required("#editor .cm-scroller")),
      content: describe(required("#editor .cm-content")),
      gutters: describe(required("#editor .cm-gutters")),
      firstGutterCell: { ...describe(firstCell), text: firstCell.textContent },
      tooltipWrapper: describe(wrappers[0]),
      firstLine: { text: firstLine.textContent, rect: rect(firstLine), range: fullRange.getBoundingClientRect().toJSON(), textRanges },
    }
  })
  const { root } = await cdp.send("DOM.getDocument")
  const platformFonts: Record<string, unknown> = {}
  for (const [name, selector] of [
    ["content", "#editor .cm-content"],
    ["gutter", "#editor .cm-lineNumbers"],
  ]) {
    const { nodeId } = await cdp.send("DOM.querySelector", { nodeId: root.nodeId, selector })
    if (!nodeId) throw new Error(`CDP cannot resolve ${selector}`)
    platformFonts[name] = await cdp.send("CSS.getPlatformFontsForNode", { nodeId })
  }
  // Query nodes with direct text children; container-only font queries can be empty.
  for (const [name, selector] of [
    ["firstLineTextNodes", "#editor .cm-content .cm-line:first-child, #editor .cm-content .cm-line:first-child *"],
    ["gutterTextNodes", "#editor .cm-lineNumbers .cm-gutterElement"],
  ]) {
    const { nodeIds } = await cdp.send("DOM.querySelectorAll", { nodeId: root.nodeId, selector })
    if (nodeIds.length === 0) throw new Error(`CDP cannot resolve rendered ${name}`)
    const records = []
    for (const nodeId of nodeIds) {
      const { node } = await cdp.send("DOM.describeNode", { nodeId, depth: 0 })
      const fonts = await cdp.send("CSS.getPlatformFontsForNode", { nodeId })
      records.push({ nodeName: node.nodeName, attributes: node.attributes, ...fonts })
    }
    if (!records.some((record) => record.fonts.some((font) => font.glyphCount > 0))) {
      throw new Error(`CDP returned no rendered glyph font for ${name}`)
    }
    platformFonts[name] = records
  }
  return { label, geometry, platformFonts, layoutMetrics: await cdp.send("Page.getLayoutMetrics") }
}

async function capture(page: Page, info: TestInfo, name: string, fullPage: boolean) {
  const path = info.outputPath(`${name}.png`)
  const bytes = await page.screenshot({ path, fullPage, animations: "disabled", caret: "hide" })
  await info.attach(name, { path, contentType: "image/png" })
  // Read dimensions of the actual browser-produced PNG; no image edits or synthesis.
  return { name, fullPage, path, width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) }
}

test("old tooltip height controls viewport versus full-page capture", async ({ page, context }, info) => {
  const errors: string[] = []
  page.on("pageerror", (error) => errors.push(String(error)))
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(`console: ${message.text()}`)
  })
  const phases: Awaited<ReturnType<typeof recordPhase>>[] = []
  const screenshots: Awaited<ReturnType<typeof capture>>[] = []
  const cdp = await context.newCDPSession(page)
  await cdp.send("DOM.enable")
  await cdp.send("CSS.enable")
  await context.tracing.start({ screenshots: true, snapshots: true, sources: true })
  try {
    await page.route("https://raw.githubusercontent.com/KentaroMorishita/seseragi/main/assets/brand/source/seseragi-logo-dark.svg", (route) =>
      route.fulfill({ contentType: "image/svg+xml", body: mockLogo })
    )
    await page.route("https://images.unsplash.com/**", (route) =>
      route.fulfill({ contentType: "image/svg+xml", body: mockImage })
    )
    await page.goto("/")
    await expect(page.locator("#sample-browser-button")).toBeVisible()
    await page.locator("#sample-browser-button").click()
    const dialog = page.locator("#sample-browser-dialog")
    await expect(dialog).toBeVisible()
    await dialog.locator('[data-sample-id="html-components"]').click()
    await expect(dialog).toBeHidden()
    await expect(page.locator("#status-text")).toHaveText("Analysis ready")
    await page.locator("#run-button").click()
    const preview = page.frameLocator("#html-preview")
    await expect(preview.locator("h1")).toContainText("Seseragi components")
    await expect(preview.locator("img").first()).toBeVisible()
    await expect.poll(async () => preview.locator("img").first().evaluate((image) => {
      const element = image as HTMLImageElement
      return { complete: element.complete, naturalWidth: element.naturalWidth }
    })).toEqual({ complete: true, naturalWidth: 960 })

    phases.push(await recordPhase(page, cdp, "before-viewport-capture"))
    expect(phases[0].geometry.body.scrollHeight).toBe(2000)
    screenshots.push(await capture(page, info, "01-old-viewport-before", false))
    phases.push(await recordPhase(page, cdp, "after-viewport-before-full-page"))
    screenshots.push(await capture(page, info, "02-old-full-page", true))
    phases.push(await recordPhase(page, cdp, "after-old-full-page"))
    screenshots.push(await capture(page, info, "03-old-viewport-after", false))
    phases.push(await recordPhase(page, cdp, "after-old-viewport-after"))
    expect(screenshots[1].height).toBe(2000)

    await page.evaluate(() => {
      const editor = document.querySelector("#editor .cm-editor")
      if (!editor) throw new Error("Missing real editor at bounded mutation")
      const wrappers = [...document.body.children].filter(
        (element) => element instanceof HTMLElement && !element.contains(editor) && element.classList.length > 0 &&
          [...element.classList].every((name) => editor.classList.contains(name)) && getComputedStyle(element).position === "relative"
      )
      if (wrappers.length !== 1) throw new Error("Tooltip wrapper became ambiguous")
      ;(wrappers[0] as HTMLElement).style.height = "0px"
    })
    await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))))
    phases.push(await recordPhase(page, cdp, "after-height-only-fix-before-full-page"))
    const fixed = phases[phases.length - 1].geometry
    expect(fixed.body.scrollHeight).toBe(1000)
    expect(fixed.tooltipWrapper.rect.height).toBe(0)
    expect(fixed.host.rect.height).toBeGreaterThan(0)
    expect(fixed.editor.rect.height).toBeGreaterThan(0)
    expect(fixed.editor.rect.height).toBeCloseTo(fixed.host.rect.height, 1)
    screenshots.push(await capture(page, info, "04-fixed-full-page", true))
    phases.push(await recordPhase(page, cdp, "after-fixed-full-page"))
    screenshots.push(await capture(page, info, "05-fixed-viewport", false))
    phases.push(await recordPhase(page, cdp, "after-fixed-viewport"))
    expect(screenshots[3].height).toBe(1000)
    for (const shot of screenshots.filter((value) => !value.fullPage)) {
      expect({ width: shot.width, height: shot.height }).toEqual({ width: 1440, height: 1000 })
    }
    for (const phase of phases) {
      expect(phase.geometry.host.rect.height).toBeGreaterThan(0)
      expect(phase.geometry.editor.rect.height).toBeGreaterThan(0)
      expect(phase.geometry.editor.rect.height).toBeCloseTo(phase.geometry.host.rect.height, 1)
    }
    expect(errors).toEqual([])
  } finally {
    const metricsPath = info.outputPath("capture-control.json")
    writeFileSync(metricsPath, JSON.stringify({ source: "old5ab", control: "only body CM tooltip wrapper inline height=0px", phases, screenshots, errors, noFontCauseClaim: true }, null, 2) + "\n")
    await info.attach("capture-control", { path: metricsPath, contentType: "application/json" })
    const tracePath = info.outputPath("capture-control-trace.zip")
    await context.tracing.stop({ path: tracePath })
    await info.attach("capture-control-trace", { path: tracePath, contentType: "application/zip" })
    await cdp.detach()
  }
})
