import assert from "node:assert/strict"
import { mkdirSync, mkdtempSync, readFileSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import {
  chromium,
  type Locator,
} from "../../playground/node_modules/@playwright/test"
import { buildSite } from "../scripts/build"
import { plannedReferenceRoutes } from "../scripts/coverage"
import { verifyLanguageMenu } from "./language-menu"
import { verifyMobileNavigation } from "./mobile-navigation"
import { verifyReferenceNavigation } from "./reference-navigation"

async function codeSurface(locator: Locator) {
  return locator.evaluate((code) => {
    const style = getComputedStyle(code)
    return {
      background: style.backgroundColor,
      border: style.borderTopWidth,
      display: style.display,
      padding: style.paddingTop,
    }
  })
}

async function codeHeaderLayout(locator: Locator) {
  return locator.evaluate((header) => {
    const title = header.querySelector(".code-panel-title")
    const actions = header.querySelector(".code-actions")
    if (!(title instanceof HTMLElement)) return null
    if (!(actions instanceof HTMLElement)) return null
    const titleBox = title.getBoundingClientRect()
    const actionsBox = actions.getBoundingClientRect()
    return {
      titleBottom: titleBox.bottom,
      actionsTop: actionsBox.top,
      actionsRight: actionsBox.right,
      headerRight: header.getBoundingClientRect().right,
    }
  })
}

async function syntaxPresentation(locator: Locator, tokens: string[]) {
  return locator.evaluate((code, tokenNames) => {
    const resolveColor = (variable: string) => {
      const probe = document.createElement("span")
      probe.style.color = `var(${variable})`
      code.append(probe)
      const color = getComputedStyle(probe).color
      probe.remove()
      return color
    }
    const variables: Record<string, string> = {
      keyword: "--seseragi-syntax-keyword",
      typeName: "--seseragi-syntax-type-name",
      standardType: "--seseragi-syntax-standard-type",
      variableName: "--seseragi-syntax-text",
      number: "--seseragi-syntax-type-name",
      bool: "--seseragi-syntax-type-name",
      string: "--seseragi-syntax-string",
      comment: "--seseragi-syntax-muted",
      operator: "--seseragi-syntax-operator",
      punctuation: "--seseragi-syntax-punctuation",
    }
    const colors = Object.fromEntries(
      tokenNames.map((token) => {
        const element = code.querySelector(`.tok-${token}`)
        if (!element) throw new Error(`missing tok-${token}`)
        const variable = variables[token]
        if (!variable)
          throw new Error(`missing palette variable for tok-${token}`)
        return [
          token,
          {
            actual: getComputedStyle(element).color,
            expected: resolveColor(variable),
          },
        ]
      })
    )
    const style = getComputedStyle(code)
    return {
      colors,
      fontSize: style.fontSize,
      lineHeight: style.lineHeight,
    }
  }, tokens)
}

const temporary = mkdtempSync(join(tmpdir(), "seseragi-site-browser-"))
const output = join(temporary, "site")
const screenshots = process.env.SITE_SCREENSHOTS
if (screenshots) mkdirSync(screenshots, { recursive: true })

try {
  buildSite({ output, origin: "https://seseragi.example" })
  const configuredHeaders = JSON.parse(
    readFileSync(resolve(import.meta.dir, "../vercel.json"), "utf8")
  ).headers[0].headers as Array<{ name: string; value: string }>
  const securityHeaders = Object.fromEntries(
    configuredHeaders.map(({ name, value }) => [name, value])
  )
  const server = Bun.serve({
    port: 0,
    hostname: "127.0.0.1",
    fetch(request) {
      const url = new URL(request.url)
      const relative = url.pathname.endsWith("/")
        ? `${url.pathname.slice(1)}index.html`
        : url.pathname.slice(1)
      const path = resolve(output, relative || "index.html")
      if (!path.startsWith(`${output}/`))
        return new Response("Not found", { status: 404 })
      return new Response(Bun.file(path), { headers: securityHeaders })
    },
  })
  try {
    const browser = await chromium.launch()
    try {
      await verifyReferenceNavigation(
        browser,
        `http://127.0.0.1:${server.port}`,
        screenshots
      )
      for (const width of [1280, 390]) {
        const context = await browser.newContext({
          viewport: { width, height: 900 },
          javaScriptEnabled: false,
        })
        const page = await context.newPage()
        const failures: string[] = []
        page.on("pageerror", (error) => failures.push(error.message))
        page.on("response", (response) => {
          if (response.status() >= 400) failures.push(response.url())
        })

        await page.goto(`http://127.0.0.1:${server.port}/`)
        assert.equal(await page.locator("h1").textContent(), "Seseragi")
        assert.equal(await page.locator("html").getAttribute("lang"), "en")
        assert.equal(await page.locator(".seseragi-highlight").count(), 1)
        assert.ok(
          (await page.locator("body").innerText()).includes(
            "pub effect fn main"
          )
        )
        const homeWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          homeWidth <= width,
          `home ${width}px viewport is ${homeWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `home-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/language/syntax/function-application/`
        )
        assert.equal(
          await page.locator("h1").textContent(),
          "Function application"
        )
        assert.equal(await page.locator(".docs-sidebar").count(), 1)
        assert.equal(await page.locator(".on-this-page").count(), 1)
        assert.equal(await page.locator(".mobile-docs-navigation").count(), 1)
        assert.equal(await page.locator(".mobile-on-this-page").count(), 1)
        assert.equal(
          await page.locator(".docs-sidebar > .sidebar-area[open]").count(),
          1
        )
        assert.equal(
          await page
            .locator(".docs-sidebar > .sidebar-area[open] > summary")
            .textContent(),
          "Language Reference"
        )
        assert.equal(
          await page
            .locator(
              ".docs-sidebar > .sidebar-area[open] .sidebar-group[open] > summary"
            )
            .textContent(),
          "Functions and operators"
        )
        assert.equal(
          await page
            .locator(
              ".docs-sidebar > .sidebar-area[open] .sidebar-section:has(.sidebar-link.current) > summary"
            )
            .textContent(),
          "Calling functions"
        )
        assert.equal(
          (await page.locator(".breadcrumbs").innerText()).replace(
            /\s+/gu,
            " "
          ),
          "Language Reference / Functions and operators / Function application"
        )
        assert.equal(await page.locator(".reference-sequence a").count(), 2)
        assert.equal(
          await page.locator(".reference-next strong").textContent(),
          "Method calls"
        )
        if (width === 390) {
          assert.notEqual(
            await page
              .locator(".language-trigger")
              .evaluate((link) => getComputedStyle(link).display),
            "none"
          )
          assert.equal(
            await page
              .locator('.language-option[hreflang="ja"]')
              .getAttribute("href"),
            "/ja/docs/language/syntax/function-application/"
          )
          const headerRows = await page
            .locator(".site-header-inner")
            .evaluate((header) => {
              const brand = header.querySelector(".site-brand")
              const navigation = header.querySelector(".site-nav")
              const tools = header.querySelector(".site-tools")
              if (!(brand instanceof HTMLElement)) return null
              if (!(navigation instanceof HTMLElement)) return null
              if (!(tools instanceof HTMLElement)) return null
              return [brand, navigation, tools].map((item) => {
                const box = item.getBoundingClientRect()
                return box.top + box.height / 2
              })
            })
          assert.ok(headerRows)
          assert.ok(Math.max(...headerRows) - Math.min(...headerRows) <= 1)
        }
        assert.ok(
          (await page.locator("body").innerText()).includes(
            "Application does not infer a different call grammar"
          )
        )
        const functionApplicationSource = await page
          .locator(".seseragi-highlight")
          .first()
          .innerText()
        assert.ok(functionApplicationSource.includes("let addOne = add 1"))
        assert.ok(!functionApplicationSource.includes("Lesson"))
        assert.ok(functionApplicationSource.trim().split("\n").length <= 6)
        const articleWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          articleWidth <= width,
          `article ${width}px viewport is ${articleWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `function-application-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/language/syntax/operator-precedence/`
        )
        assert.equal(
          await page.locator("h1").textContent(),
          "Operator precedence"
        )
        assert.ok(
          (await page.locator("body").innerText()).includes("a < b < c")
        )
        const operatorsWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          operatorsWidth <= width,
          `operator article ${width}px viewport is ${operatorsWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `operator-precedence-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/language/types/type-system/`
        )
        assert.equal(await page.locator("h1").textContent(), "Type system")
        assert.ok(
          (await page.locator("body").innerText()).includes(
            "unresolved meaning becomes a diagnostic"
          )
        )
        const typeSystemCode = await syntaxPresentation(
          page.locator(".seseragi-highlight").first(),
          [
            "keyword",
            "typeName",
            "standardType",
            "variableName",
            "number",
            "string",
            "comment",
            "operator",
            "punctuation",
          ]
        )
        assert.equal(typeSystemCode.fontSize, "14px")
        assert.equal(typeSystemCode.lineHeight, "23.1px")
        assert.equal(
          await page
            .locator(".code-panel-header")
            .first()
            .evaluate((header) => getComputedStyle(header).fontSize),
          "12px"
        )
        for (const [token, colors] of Object.entries(typeSystemCode.colors)) {
          assert.equal(
            colors.actual,
            colors.expected,
            `tok-${token} must use the canonical syntax palette`
          )
        }
        const typeSystemWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          typeSystemWidth <= width,
          `type system article ${width}px viewport is ${typeSystemWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `type-system-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/language/patterns/match/`
        )
        assert.equal(
          await page.locator("h1").textContent(),
          "Match expressions"
        )
        assert.equal(
          await page
            .locator(
              ".docs-sidebar > .sidebar-area[open] .sidebar-group[open] > summary"
            )
            .textContent(),
          "Patterns"
        )
        assert.equal(
          await page
            .locator(
              ".docs-sidebar > .sidebar-area[open] .sidebar-section:has(.sidebar-link.current) > summary"
            )
            .textContent(),
          "Binding and matching"
        )
        assert.equal(
          (await page.locator(".breadcrumbs").innerText()).replace(
            /\s+/gu,
            " "
          ),
          "Language Reference / Patterns / Binding and matching / Match expressions"
        )
        assert.equal(
          await page.locator(".reference-previous strong").textContent(),
          "Irrefutable patterns"
        )
        assert.equal(await page.locator(".seseragi-highlight").count(), 2)
        const matchSource = await page
          .locator(".seseragi-highlight")
          .first()
          .innerText()
        assert.ok(matchSource.includes("Complete count when count > 0"))
        assert.ok(!matchSource.includes("Lesson"))
        const matchWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          matchWidth <= width,
          `match article ${width}px viewport is ${matchWidth}px`
        )

        await page.goto(
          `http://127.0.0.1:${server.port}/ja/docs/language/patterns/match/`
        )
        assert.equal(await page.locator("html").getAttribute("lang"), "ja")
        assert.equal(await page.locator("h1").textContent(), "パターン照合")
        const japaneseMatch = await page.locator("body").innerText()
        assert.ok(
          japaneseMatch.includes("上から順に照合し、最初に一致した分岐を使う")
        )
        assert.ok(
          japaneseMatch.includes("ガード付きの分岐だけでは、取りこぼしが残る")
        )
        assert.ok(!japaneseMatch.match(/#[0-9]+/u))
        const japaneseMatchWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          japaneseMatchWidth <= width,
          `Japanese match article ${width}px viewport is ${japaneseMatchWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `match-ja-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/language/traits/do-notation/`
        )
        assert.equal(await page.locator("h1").textContent(), "Do notation")
        assert.equal(
          await page
            .locator(
              ".docs-sidebar > .sidebar-area[open] .sidebar-group[open] > summary"
            )
            .textContent(),
          "Traits and abstraction"
        )
        assert.equal(
          await page
            .locator(
              ".docs-sidebar > .sidebar-area[open] .sidebar-section:has(.sidebar-link.current) > summary"
            )
            .textContent(),
          "Do notation"
        )
        assert.equal(
          (await page.locator(".breadcrumbs").innerText()).replace(
            /\s+/gu,
            " "
          ),
          "Language Reference / Traits and abstraction / Do notation"
        )
        assert.equal(
          await page.locator(".reference-previous strong").textContent(),
          "Inherent methods versus trait operations"
        )
        assert.equal(
          await page.locator(".reference-next strong").textContent(),
          "How do blocks desugar"
        )
        const doText = await page.locator("body").innerText()
        assert.ok(doText.includes("Do is generic Monad syntax"))
        assert.ok(!doText.includes("Lesson"))
        assert.ok(!doText.match(/#[0-9]+/u))
        assert.equal(await page.locator(".seseragi-highlight").count(), 2)
        const doWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          doWidth <= width,
          `do notation article ${width}px viewport is ${doWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `do-notation-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/ja/docs/language/traits/do-notation/`
        )
        assert.equal(await page.locator("html").getAttribute("lang"), "ja")
        assert.equal(await page.locator("h1").textContent(), "do記法")
        const japaneseDoText = await page.locator("body").innerText()
        assert.ok(
          japaneseDoText.includes("doは、Monadの処理を読みやすく書く構文")
        )
        assert.ok(!japaneseDoText.match(/#[0-9]+/u))
        const japaneseDoWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          japaneseDoWidth <= width,
          `Japanese do notation article ${width}px viewport is ${japaneseDoWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `do-notation-ja-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/language/effects/effect-type/`
        )
        assert.equal(await page.locator("h1").textContent(), "The Effect type")
        assert.equal(
          await page
            .locator(
              ".docs-sidebar > .sidebar-area[open] .sidebar-group[open] > summary"
            )
            .textContent(),
          "Effects and failure"
        )
        assert.equal(
          await page
            .locator(
              ".docs-sidebar > .sidebar-area[open] .sidebar-section:has(.sidebar-link.current) > summary"
            )
            .textContent(),
          "Values and computations"
        )
        assert.equal(
          (await page.locator(".breadcrumbs").innerText()).replace(
            /\s+/gu,
            " "
          ),
          "Language Reference / Effects and failure / Values and computations / The Effect type"
        )
        assert.equal(
          await page.locator(".reference-previous strong").textContent(),
          "Either and typed results"
        )
        assert.equal(
          await page.locator(".reference-next strong").textContent(),
          "Effect functions with an explicit contract"
        )
        const effectText = await page.locator("body").innerText()
        assert.ok(
          effectText.includes("Construction and execution are separate")
        )
        assert.ok(!effectText.includes("Lesson"))
        assert.ok(!effectText.match(/#[0-9]+/u))
        assert.equal(await page.locator(".seseragi-highlight").count(), 2)
        const effectWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          effectWidth <= width,
          `Effect article ${width}px viewport is ${effectWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `effect-type-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/ja/docs/language/effects/effect-type/`
        )
        assert.equal(await page.locator("html").getAttribute("lang"), "ja")
        assert.equal(await page.locator("h1").textContent(), "Effect型")
        const japaneseEffectText = await page.locator("body").innerText()
        assert.ok(japaneseEffectText.includes("構築と実行は別の段階"))
        assert.ok(!japaneseEffectText.match(/#[0-9]+/u))
        const japaneseEffectWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          japaneseEffectWidth <= width,
          `Japanese Effect article ${width}px viewport is ${japaneseEffectWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `effect-type-ja-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/language/effects/signals-and-transactions/`
        )
        assert.equal(
          await page.locator("h1").textContent(),
          "Signals and transactions"
        )
        assert.equal(
          await page
            .locator(
              ".docs-sidebar > .sidebar-area[open] .sidebar-group[open] > summary"
            )
            .textContent(),
          "Signals"
        )
        assert.equal(
          await page
            .locator(
              ".docs-sidebar > .sidebar-area[open] .sidebar-section:has(.sidebar-link.current) > summary"
            )
            .textContent(),
          "Reactive state"
        )
        assert.equal(
          (await page.locator(".breadcrumbs").innerText()).replace(
            /\s+/gu,
            " "
          ),
          "Language Reference / Signals / Signals and transactions"
        )
        assert.equal(
          await page.locator(".reference-previous strong").textContent(),
          "Fiber supervision"
        )
        assert.equal(
          await page.locator(".reference-next strong").textContent(),
          "Derived Signals"
        )
        const signalText = await page.locator("body").innerText()
        assert.ok(
          signalText.includes("Signal state is accessed through Effect")
        )
        assert.ok(!signalText.includes("Lesson"))
        assert.ok(!signalText.match(/#[0-9]+/u))
        assert.equal(await page.locator(".seseragi-highlight").count(), 2)
        const signalWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          signalWidth <= width,
          `Signal article ${width}px viewport is ${signalWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `signal-transactions-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/ja/docs/language/effects/signals-and-transactions/`
        )
        assert.equal(await page.locator("html").getAttribute("lang"), "ja")
        assert.equal(
          await page.locator("h1").textContent(),
          "Signalとトランザクション"
        )
        const japaneseSignalText = await page.locator("body").innerText()
        assert.ok(
          japaneseSignalText.includes(
            "Signalの現在値は、Effectの中で読み書きする"
          )
        )
        assert.ok(!japaneseSignalText.match(/#[0-9]+/u))
        const japaneseSignalWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          japaneseSignalWidth <= width,
          `Japanese Signal article ${width}px viewport is ${japaneseSignalWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `signal-transactions-ja-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/language/types/requirement-merge/`
        )
        assert.equal(
          await page.locator("h1").textContent(),
          "Requirement merge"
        )
        assert.ok(
          (await page.locator("body").innerText()).includes("SES-E0001")
        )
        const highlightedCodeStyle = await codeSurface(
          page.locator(".code-panel pre > code.seseragi-highlight").first()
        )
        assert.deepEqual(highlightedCodeStyle, {
          background: "rgba(0, 0, 0, 0)",
          border: "0px",
          display: "block",
          padding: "0px",
        })
        const inlineCodeStyle = await page
          .locator(".article-content li > code")
          .first()
          .evaluate((code) => {
            const style = getComputedStyle(code)
            return {
              background: style.backgroundColor,
              border: style.borderTopWidth,
              fontSize: style.fontSize,
            }
          })
        assert.notEqual(inlineCodeStyle.background, "rgba(0, 0, 0, 0)")
        assert.equal(inlineCodeStyle.border, "1px")
        assert.equal(inlineCodeStyle.fontSize, "14.4px")
        if (width === 390) {
          const headerLayout = await codeHeaderLayout(
            page.locator(".code-panel-header:has(.code-actions)").first()
          )
          assert.ok(headerLayout)
          assert.ok(headerLayout.titleBottom <= headerLayout.actionsTop)
          assert.ok(headerLayout.actionsRight <= headerLayout.headerRight)
        }
        const requirementWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          requirementWidth <= width,
          `requirement article ${width}px viewport is ${requirementWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `requirement-merge-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/ja/docs/language/types/requirement-merge/`
        )
        assert.equal(await page.locator("html").getAttribute("lang"), "ja")
        assert.ok(!(await page.locator("body").innerText()).match(/#[0-9]+/u))
        if (width === 390) {
          const japaneseHeaderLayout = await codeHeaderLayout(
            page.locator(".code-panel-header:has(.code-actions)").first()
          )
          assert.ok(japaneseHeaderLayout)
          assert.ok(
            japaneseHeaderLayout.titleBottom <= japaneseHeaderLayout.actionsTop
          )
          assert.ok(
            japaneseHeaderLayout.actionsRight <=
              japaneseHeaderLayout.headerRight
          )
        }
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `requirement-merge-ja-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/language/model/non-features/`
        )
        assert.equal(
          await page.locator("h1").textContent(),
          "Features the language does not have"
        )
        assert.ok(
          (await page.locator("body").innerText()).includes(
            "return, break, and continue"
          )
        )
        const modelWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          modelWidth <= width,
          `language model ${width}px viewport is ${modelWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `language-model-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/ja/docs/language/model/non-features/`
        )
        const japaneseModelText = await page.locator("body").innerText()
        assert.ok(japaneseModelText.includes("変数への再代入と可変フィールド"))
        assert.ok(!japaneseModelText.match(/#[0-9]+/u))

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/language/syntax/literals/`
        )
        assert.equal(await page.locator("h1").textContent(), "Literals")
        assert.equal(await page.locator(".seseragi-highlight").count(), 2)
        assert.ok(
          (await page.locator("body").innerText()).includes("SES-P0203")
        )
        const literalsWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          literalsWidth <= width,
          `literals ${width}px viewport is ${literalsWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `literals-${width}.png`),
            fullPage: true,
          })

        await page.goto(`http://127.0.0.1:${server.port}/docs/`)
        assert.equal(
          await page.locator("h1").textContent(),
          "Seseragi Reference"
        )
        assert.equal(await page.locator(".docs-sidebar").count(), 0)
        assert.equal(await page.locator(".mobile-docs-navigation").count(), 0)
        assert.equal(await page.locator(".reference-area-card").count(), 2)
        assert.ok((await page.locator(".reference-topic-link").count()) >= 10)
        const documentationText = await page.locator("body").innerText()
        assert.ok(documentationText.includes("Language Reference"))
        assert.ok(documentationText.includes("Standard Library"))
        assert.ok(documentationText.includes("Functions and operators"))
        assert.ok(documentationText.includes("Collections"))
        assert.ok(documentationText.includes("interactive Tour"))
        assert.ok(!documentationText.includes("Get Started"))
        assert.equal(
          await page
            .locator('a[href="https://seseragi.vercel.app/tour/"]')
            .count(),
          2
        )
        const documentationWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          documentationWidth <= width,
          `documentation ${width}px viewport is ${documentationWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `documentation-${width}.png`),
            fullPage: true,
          })

        await page.goto(`http://127.0.0.1:${server.port}/ja/docs/`)
        assert.equal(await page.locator("html").getAttribute("lang"), "ja")
        assert.equal(
          await page.locator("h1").textContent(),
          "Seseragi リファレンス"
        )
        assert.equal(await page.locator(".docs-sidebar").count(), 0)
        assert.ok(
          (await page.locator("body").innerText()).includes("言語リファレンス")
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `documentation-ja-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/library/array/function/get/`
        )
        assert.equal(await page.locator("h1").textContent(), "get")
        assert.ok(
          (await page.locator(".api-reference").innerText()).includes(
            "std/array::get"
          )
        )
        assert.equal(
          await page
            .locator(".docs-sidebar .sidebar-link.current")
            .evaluate(
              (link) =>
                link.closest("details")?.querySelector(":scope > summary")
                  ?.textContent
            ),
          "std/array"
        )
        assert.equal(
          await page.locator(".docs-sidebar > .sidebar-area[open]").count(),
          1
        )
        assert.equal(
          await page
            .locator(".docs-sidebar > .sidebar-area[open] > summary")
            .textContent(),
          "Standard Library"
        )
        assert.equal(
          await page
            .locator(
              ".docs-sidebar > .sidebar-area[open] .sidebar-group[open] > summary"
            )
            .textContent(),
          "Collections"
        )
        assert.equal(
          await page
            .locator(
              ".docs-sidebar > .sidebar-area[open] .sidebar-section[open]"
            )
            .count(),
          1
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `standard-library-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/language/modules/identity/`
        )
        assert.equal(await page.locator("h1").textContent(), "Module identity")
        const modulePanels = page.locator("main .code-panel")
        assert.equal(await modulePanels.count(), 2)
        assert.equal(
          await modulePanels.nth(0).locator(".code-actions a").count(),
          1
        )
        assert.equal(
          await modulePanels.nth(1).locator(".code-actions a").count(),
          0
        )
        assert.equal(
          await page
            .locator(".docs-sidebar .sidebar-link.current")
            .textContent(),
          "Module identity"
        )
        assert.equal(
          await page
            .locator(".reference-sequence a")
            .last()
            .getAttribute("href"),
          "/docs/language/modules/packages/"
        )
        assert.equal(
          await modulePanels
            .nth(1)
            .locator("pre")
            .evaluate((pre) => getComputedStyle(pre).fontSize),
          "14px"
        )
        assert.ok(
          await page.evaluate(
            () => document.documentElement.scrollWidth <= innerWidth
          )
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `modules-identity-${width}.png`),
            fullPage: true,
          })
        await page.goto(
          `http://127.0.0.1:${server.port}/ja/docs/language/modules/visibility/`
        )
        assert.equal(
          await page.locator("h1").textContent(),
          "公開範囲とopaqueな表現"
        )
        assert.ok(!(await page.locator("main").innerText()).includes("準備中"))
        assert.equal(
          await page
            .locator(".reference-sequence a")
            .last()
            .getAttribute("href"),
          "/ja/docs/language/modules/imports/"
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `modules-visibility-ja-${width}.png`),
            fullPage: true,
          })

        assert.deepEqual(failures, [])
        await context.close()
      }

      for (const width of [1710, 1160, 960, 760]) {
        const context = await browser.newContext({
          viewport: { width, height: 1112 },
          javaScriptEnabled: false,
        })
        const page = await context.newPage()
        const failures: string[] = []
        page.on("pageerror", (error) => failures.push(error.message))
        page.on("response", (response) => {
          if (response.status() >= 400) failures.push(response.url())
        })

        await page.goto(`http://127.0.0.1:${server.port}/`)
        const home = await page.evaluate(() => {
          const hero = document.querySelector(".home-hero")
          const heading = document.querySelector(".home-hero h1")
          const lead = document.querySelector(".home-lead")
          if (!(hero instanceof HTMLElement)) return null
          if (!(heading instanceof HTMLElement)) return null
          if (!(lead instanceof HTMLElement)) return null
          return {
            columns: getComputedStyle(hero).gridTemplateColumns,
            headingFont: Number.parseFloat(getComputedStyle(heading).fontSize),
            headingTracking: Number.parseFloat(
              getComputedStyle(heading).letterSpacing
            ),
            leadFont: Number.parseFloat(getComputedStyle(lead).fontSize),
            overflow: document.documentElement.scrollWidth - innerWidth,
          }
        })
        assert.ok(home)
        assert.ok(home.headingFont <= 68)
        assert.ok(home.headingFont >= 44)
        assert.ok(home.headingTracking >= -3)
        assert.ok(home.leadFont <= 19)
        assert.equal(home.overflow, 0)
        assert.equal(
          home.columns.split(" ").length,
          width <= 960 ? 1 : 2,
          `home ${width}px column count`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `home-responsive-${width}.png`),
            fullPage: true,
          })

        await page.goto(`http://127.0.0.1:${server.port}/docs/`)
        const docsLandingTitle = await page
          .locator(".page-intro h1")
          .evaluate((heading) =>
            Number.parseFloat(getComputedStyle(heading).fontSize)
          )
        assert.ok(docsLandingTitle <= 44)
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `docs-responsive-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/language/syntax/function-application/`
        )
        const article = await page.evaluate(() => {
          const frame = document.querySelector(".docs-frame")
          const header = document.querySelector(".site-header")
          const heading = document.querySelector(".page-intro h1")
          const section = document.querySelector(".article-content h2")
          const body = document.querySelector(".article-content p")
          const sidebar = document.querySelector(".docs-sidebar")
          const mobileNavigation = document.querySelector(
            ".mobile-docs-navigation"
          )
          const onThisPage = document.querySelector(".on-this-page")
          if (!(frame instanceof HTMLElement)) return null
          if (!(header instanceof HTMLElement)) return null
          if (!(heading instanceof HTMLElement)) return null
          if (!(section instanceof HTMLElement)) return null
          if (!(body instanceof HTMLElement)) return null
          if (!(sidebar instanceof HTMLElement)) return null
          if (!(mobileNavigation instanceof HTMLElement)) return null
          if (!(onThisPage instanceof HTMLElement)) return null
          return {
            display: getComputedStyle(frame).display,
            columns: getComputedStyle(frame).gridTemplateColumns,
            headerBottom: header.getBoundingClientRect().bottom,
            headingTop: heading.getBoundingClientRect().top,
            headingFont: Number.parseFloat(getComputedStyle(heading).fontSize),
            headingTracking: Number.parseFloat(
              getComputedStyle(heading).letterSpacing
            ),
            sectionFont: Number.parseFloat(getComputedStyle(section).fontSize),
            bodyFont: Number.parseFloat(getComputedStyle(body).fontSize),
            sidebar: getComputedStyle(sidebar).display,
            mobileNavigation: getComputedStyle(mobileNavigation).display,
            onThisPage: getComputedStyle(onThisPage).display,
            overflow: document.documentElement.scrollWidth - innerWidth,
          }
        })
        assert.ok(article)
        assert.ok(article.headingTop >= article.headerBottom + 32)
        assert.ok(article.headingFont <= 42)
        assert.ok(article.headingTracking >= -1)
        assert.ok(article.sectionFont <= 28)
        assert.equal(article.bodyFont, 16)
        assert.equal(article.overflow, 0)
        if (width > 1160) {
          assert.equal(article.display, "grid")
          assert.equal(article.columns.split(" ").length, 3)
          assert.notEqual(article.sidebar, "none")
          assert.equal(article.mobileNavigation, "none")
          assert.notEqual(article.onThisPage, "none")
        } else if (width > 760) {
          assert.equal(article.display, "grid")
          assert.equal(article.columns.split(" ").length, 2)
          assert.notEqual(article.sidebar, "none")
          assert.equal(article.mobileNavigation, "none")
          assert.equal(article.onThisPage, "none")
        } else {
          assert.equal(article.display, "block")
          assert.equal(article.sidebar, "none")
          assert.notEqual(article.mobileNavigation, "none")
          assert.equal(article.onThisPage, "none")
        }
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `article-responsive-${width}.png`),
            fullPage: true,
          })

        assert.deepEqual(failures, [])
        await context.close()
      }
      await verifyMobileNavigation(
        browser,
        `http://127.0.0.1:${server.port}`,
        screenshots
      )
      await verifyLanguageMenu(
        browser,
        `http://127.0.0.1:${server.port}`,
        screenshots
      )
      let auditedArticles = 0
      for (const width of [320, 1280]) {
        const context = await browser.newContext({
          viewport: { width, height: 900 },
          javaScriptEnabled: false,
        })
        try {
          const page = await context.newPage()
          for (const locale of ["en", "ja"]) {
            const prefix = locale === "ja" ? "/ja" : ""
            await page.goto(
              `http://127.0.0.1:${server.port}${prefix}/docs/language/`
            )
            assert.equal(await page.locator(".directory-group").count(), 12)
            assert.equal(
              await page.evaluate(
                () => document.documentElement.scrollWidth - innerWidth
              ),
              0
            )
            if (screenshots)
              await page.screenshot({
                path: join(
                  screenshots,
                  `language-directory-${locale}-${width}.png`
                ),
                fullPage: true,
              })
            await page.goto(
              `http://127.0.0.1:${server.port}/ja/docs/library/array/function/get/`
            )
            assert.equal(
              await page
                .locator(".reference-description summary")
                .textContent(),
              "APIの詳細説明（英語原文）"
            )
            await page.locator(".reference-description summary").click()
            assert.equal(
              await page
                .locator(".reference-description p")
                .getAttribute("lang"),
              "en"
            )
            assert.equal(
              await page.evaluate(
                () => document.documentElement.scrollWidth - innerWidth
              ),
              0
            )
          }
          for (const route of plannedReferenceRoutes.filter((route) =>
            route.startsWith("/docs/language/")
          )) {
            for (const locale of ["en", "ja"]) {
              const localizedRoute = locale === "ja" ? `/ja${route}` : route
              const response = await page.goto(
                `http://127.0.0.1:${server.port}${localizedRoute}`
              )
              assert.equal(response?.status(), 200, localizedRoute)
              assert.equal(
                await page.locator("html").getAttribute("lang"),
                locale
              )
              assert.equal(await page.locator("main h1").count(), 1)
              assert.ok(
                (await page.locator(".breadcrumbs").innerText()).length > 0
              )
              assert.equal(
                await page
                  .locator(".reference-next, .reference-previous")
                  .count(),
                route.endsWith("/modules/entry-points/") ? 1 : 2
              )
              assert.equal(
                await page.locator('link[hreflang="ja"]').getAttribute("href"),
                `https://seseragi.example/ja${route}`
              )
              const layout = await page.locator("main").evaluate((main) => ({
                overflow: document.documentElement.scrollWidth - innerWidth,
                codeFonts: [...main.querySelectorAll("pre > code")].map(
                  (code) => Number.parseFloat(getComputedStyle(code).fontSize)
                ),
                text: main.textContent ?? "",
              }))
              assert.equal(
                layout.overflow,
                0,
                `${localizedRoute} at ${width}px`
              )
              assert.ok(
                layout.codeFonts.every((font) => font >= 13 && font <= 15),
                `${localizedRoute}: code font ${layout.codeFonts.join(",")}`
              )
              assert.ok(
                !/準備中|整備中|Lesson|#[0-9]+/u.test(layout.text),
                localizedRoute
              )
              if (
                screenshots &&
                /\/(?:generic-functions|variance|re-exports)\/$/u.test(route)
              ) {
                await page.screenshot({
                  path: join(
                    screenshots,
                    `review-${route.split("/").at(-2)}-${locale}-${width}.png`
                  ),
                  fullPage: true,
                })
              }
              auditedArticles += 1
            }
          }
        } finally {
          await context.close()
        }
      }
      console.log(
        `Reviewed ${auditedArticles} language article/locale/viewport combinations without JavaScript`
      )
    } finally {
      await browser.close()
    }
  } finally {
    server.stop(true)
  }
} finally {
  rmSync(temporary, { recursive: true, force: true })
}
