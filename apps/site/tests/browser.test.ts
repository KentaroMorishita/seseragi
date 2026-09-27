import assert from "node:assert/strict"
import { mkdirSync, mkdtempSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import {
  chromium,
  type Locator,
} from "../../playground/node_modules/@playwright/test"
import { buildSite } from "../scripts/build"

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
      return new Response(Bun.file(path))
    },
  })
  try {
    const browser = await chromium.launch()
    try {
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
        assert.equal(typeSystemCode.fontSize, width === 390 ? "11px" : "12px")
        assert.equal(
          typeSystemCode.lineHeight,
          width === 390 ? "17.6px" : "19.8px"
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
            }
          })
        assert.notEqual(inlineCodeStyle.background, "rgba(0, 0, 0, 0)")
        assert.equal(inlineCodeStyle.border, "1px")
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
        assert.equal(await page.locator("h1").textContent(), "Documentation")
        assert.equal(await page.locator(".docs-sidebar").count(), 1)
        assert.equal(await page.locator(".mobile-docs-navigation").count(), 1)

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/get-started/hello-seseragi/`
        )
        assert.equal(await page.locator("h1").textContent(), "Hello, Seseragi")
        assert.equal(await page.locator(".seseragi-highlight").count(), 1)
        assert.equal(await page.locator(".guide-next").count(), 1)
        assert.ok(
          (await page.locator("body").innerText()).includes(
            'pub effect fn main = println "Hello, Seseragi!"'
          )
        )
        const guideWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          guideWidth <= width,
          `getting started ${width}px viewport is ${guideWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `getting-started-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/get-started/project-layout/`
        )
        const terminalCodeStyle = await codeSurface(
          page.locator(".terminal-panel pre > code").first()
        )
        assert.deepEqual(terminalCodeStyle, {
          background: "rgba(0, 0, 0, 0)",
          border: "0px",
          display: "block",
          padding: "0px",
        })
        const projectLayoutWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          projectLayoutWidth <= width,
          `project layout ${width}px viewport is ${projectLayoutWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `project-layout-${width}.png`),
            fullPage: true,
          })

        await page.goto(`http://127.0.0.1:${server.port}/ja/docs/`)
        assert.equal(await page.locator("html").getAttribute("lang"), "ja")
        assert.equal(await page.locator("h1").textContent(), "ドキュメント")

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
          `http://127.0.0.1:${server.port}/docs/get-started/hello-seseragi/`
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
            path: join(screenshots, `getting-started-responsive-${width}.png`),
            fullPage: true,
          })

        assert.deepEqual(failures, [])
        await context.close()
      }
    } finally {
      await browser.close()
    }
  } finally {
    server.stop(true)
  }
} finally {
  rmSync(temporary, { recursive: true, force: true })
}
