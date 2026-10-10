import assert from "node:assert/strict"
import { mkdirSync, readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { chromium } from "../../playground/node_modules/@playwright/test"
import { generatorInput } from "../scripts/build"
import {
  articleExecutions,
  typeDiagnostics,
} from "../scripts/published-examples"
import { staticSiteHandler } from "../scripts/static-site-handler"
import { verifyFunctionChapter } from "./function-chapter-browser"

const root = resolve(import.meta.dir, "../../..")
const output = resolve(root, process.env.SITE_OUTPUT ?? "target/site")
const manifest = JSON.parse(
  readFileSync(join(output, "site-manifest.json"), "utf8")
)
const input = generatorInput("https://seseragi.vercel.app/")
const displayed = new Set<string>()
const configuration = JSON.parse(
  readFileSync(join(root, "apps/site/vercel.json"), "utf8")
)
const remote = process.env.SITE_ORIGIN
const server = remote
  ? undefined
  : Bun.serve({
      hostname: "127.0.0.1",
      port: 0,
      fetch: staticSiteHandler(output, configuration),
    })
const origin = remote ?? `http://127.0.0.1:${server?.port}`
const screenshots = resolve(
  root,
  process.env.SITE_SCREENSHOTS ?? "target/site-verification/screenshots"
)
mkdirSync(screenshots, { recursive: true })
const browser = await chromium.launch({ headless: true })
let cases = 0
let chapterWalks = 0
const screenshotNames: Record<string, string> = {
  "/": "home",
  "/docs/composition/apply/": "apply",
  "/docs/effects/errors/": "errors",
  "/docs/types/variants/": "variants",
  "/docs/signals/transactions/": "transactions",
}
try {
  // Every published route and every API declaration is checked in the browser,
  // against compiler data rather than a historical page-count snapshot.
  const context = await browser.newContext()
  const page = await context.newPage()
  const errors: string[] = []
  page.on("pageerror", (error) => errors.push(error.message))
  page.on("response", (response) => {
    if (response.status() >= 400)
      errors.push(`${response.status()} ${response.url()}`)
  })
  for (const route of manifest.pages as string[]) {
    const response = await page.goto(origin + route)
    assert.equal(response?.status(), 200, route)
    assert.equal(await page.locator("h1").count(), 1, route)
    assert.equal(await page.locator("main#content").count(), 1, route)
    const japanese = route.startsWith("/ja/")
    assert.equal(
      await page.locator("html").getAttribute("lang"),
      japanese ? "ja" : "en"
    )
    const path = japanese ? route.slice(3) : route
    for (const source of await page
      .locator(".code-panel:not(.terminal-panel) .seseragi-highlight")
      .allTextContents()) {
      const canonical = input.examples.find(
        (example) => example.source === source
      )
      assert.ok(canonical, `Noncanonical example displayed: ${route}`)
      displayed.add(canonical.id)
    }
    const module = input.referenceModules.find(
      ({ specifier }) => path === `/docs/api/${specifier.slice(4)}/`
    )
    if (module) {
      assert.deepEqual(
        await page.locator(".api-reference pre code").allTextContents(),
        module.items.map(({ signature }) => signature),
        route
      )
      assert.deepEqual(
        await page
          .locator(".api-reference")
          .evaluateAll((elements) => elements.map((element) => element.id)),
        module.items.map(({ anchor }) => anchor),
        route
      )
    }
    for (const example of articleExecutions) {
      if (path !== example.route) continue
      const negatives = typeDiagnostics.filter((item) => item.route === path)
      assert.deepEqual(
        await page.locator(".code-panel .seseragi-highlight").allTextContents(),
        [example, ...negatives].map((item) =>
          readFileSync(join(root, item.sourcePath), "utf8")
        ),
        route
      )
      for (const negative of negatives)
        assert.ok(
          (
            await page.locator(".terminal-panel pre code").allTextContents()
          ).includes(negative.display),
          `${route}: diagnostic differs`
        )
      assert.equal(
        await page.locator(".terminal-panel pre code").first().textContent(),
        example.output.trimEnd(),
        route
      )
    }
    const counterpart = japanese ? path : route === "/" ? "/ja/" : `/ja${route}`
    assert.equal(
      await page.locator(`.language-menu a[href="${counterpart}"]`).count(),
      1,
      route
    )
    assert.ok(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth
      ),
      route
    )
    cases++
  }
  assert.deepEqual(errors, [])
  console.info(
    `Verified ${cases} published routes and every compiler API declaration`
  )
  assert.deepEqual(
    [...displayed].sort(),
    input.examples.map(({ id }) => id).sort(),
    "Every input example must actually be published"
  )
  await context.close()

  for (const javaScriptEnabled of [false, true])
    for (const width of [320, 390, 1280]) {
      const context = await browser.newContext({
        viewport: { width, height: 900 },
        javaScriptEnabled,
      })
      try {
        const page = await context.newPage()
        for (const prefix of ["", "/ja"]) {
          for (const route of [
            "/",
            "/docs/",
            "/docs/composition/",
            "/docs/composition/apply/",
            "/docs/effects/",
            "/docs/effects/errors/",
            "/docs/types/",
            "/docs/types/records/",
            "/docs/types/variants/",
            "/docs/types/generics/",
            "/docs/signals/",
            "/docs/signals/derived/",
            "/docs/signals/subscriptions/",
            "/docs/signals/transactions/",
            "/docs/api/",
            "/docs/api/prelude/",
          ]) {
            await page.goto(origin + prefix + route)
            assert.ok(await page.locator("main").innerText(), route)
            assert.ok(
              await page.evaluate(
                () => document.documentElement.scrollWidth <= innerWidth
              ),
              `${width} ${prefix}${route}`
            )
            if (route === "/docs/api/prelude/") {
              await page.locator(".symbol-index summary").click()
              const link = page.locator(".symbol-index a").first()
              const href = await link.getAttribute("href")
              await link.click()
              assert.equal(new URL(page.url()).hash, href)
              assert.ok(await page.locator(href ?? "missing").isVisible())
            }
            if (screenshotNames[route])
              await page.screenshot({
                path: join(
                  screenshots,
                  `reboot-${prefix ? "ja" : "en"}-${screenshotNames[route]}-${width}-${javaScriptEnabled ? "js" : "no-js"}.png`
                ),
                fullPage: true,
              })
            cases++
          }
          await page.goto(`${origin + prefix}/docs/composition/apply/`)
          const other = prefix ? "" : "/ja"
          await page.locator(".language-menu summary").focus()
          await page.keyboard.press("Enter")
          await Promise.all([
            page.waitForURL(`${origin + other}/docs/composition/apply/`),
            page
              .locator(
                `.language-menu a[href="${other}/docs/composition/apply/"]`
              )
              .click(),
          ])
          assert.equal(
            new URL(page.url()).pathname,
            `${other}/docs/composition/apply/`
          )
          if (width < 768) {
            await page.locator(".site-menu summary").click()
            await Promise.all([
              page.waitForURL(`${origin + other}/docs/api/`),
              page.locator(`.site-menu a[href="${other}/docs/api/"]`).click(),
            ])
            assert.equal(new URL(page.url()).pathname, `${other}/docs/api/`)
          }
          // Read the new chapter through its actual links in both locales,
          // including mobile widths and browsers without JavaScript.
          for (const chapter of [
            [
              "/docs/effects/",
              "/docs/effects/actions/",
              "/docs/effects/errors/",
              "/docs/api/effect/",
            ],
            [
              "/docs/types/",
              "/docs/types/records/",
              "/docs/types/variants/",
              "/docs/types/generics/",
              "/docs/api/prelude/",
            ],
            [
              "/docs/signals/",
              "/docs/signals/derived/",
              "/docs/signals/subscriptions/",
              "/docs/signals/transactions/",
              "/docs/api/signal/",
            ],
          ]) {
            await page.goto(`${origin + prefix}/docs/`)
            for (const route of chapter) {
              const link = page
                .locator(`main a[href="${prefix}${route}"]`)
                .first()
              await Promise.all([
                page.waitForURL(origin + prefix + route),
                link.click(),
              ])
              assert.equal(new URL(page.url()).pathname, prefix + route)
            }
            chapterWalks++
          }
        }
      } finally {
        await context.close()
      }
    }
  await verifyFunctionChapter(browser, origin)
  console.info(
    `Reboot browser: ${cases} route/viewport cases; ${chapterWalks} types/effects/signals chapter walks; all signatures, locale navigation, no-JS, mobile, and first chapter verified`
  )
} finally {
  await browser.close()
  server?.stop(true)
}
