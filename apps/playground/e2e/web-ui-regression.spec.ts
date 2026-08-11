import { readFileSync } from "node:fs"
import {
  expect,
  type FrameLocator,
  type Locator,
  type Page,
  type TestInfo,
  test,
} from "@playwright/test"

type MatrixSample = Readonly<{
  readonly id: string
  readonly pickerLabel: string
  readonly heading: string
  readonly architecture: string
  readonly requiredSurfaces: readonly string[]
  readonly requiredStates: readonly string[]
}>

type Matrix = Readonly<{
  readonly viewports: readonly Readonly<{
    readonly id: string
    readonly width: number
    readonly height: number
  }>[]
  readonly samples: readonly MatrixSample[]
}>

const matrix = JSON.parse(
  readFileSync(
    new URL("../tests/fixtures/web-ui-regression.json", import.meta.url),
    "utf8"
  )
) as Matrix

const mockImage = `<svg xmlns="http://www.w3.org/2000/svg" width="960" height="480" viewBox="0 0 960 480"><rect width="960" height="480" fill="#1d4ed8"/><path d="M0 362 224 180l126 96 190-164 420 250v118H0Z" fill="#34d399"/><circle cx="748" cy="118" r="54" fill="#fef3c7"/></svg>`
const mockLogo = readFileSync(
  new URL(
    "../../../assets/brand/source/seseragi-logo-dark.svg",
    import.meta.url
  ),
  "utf8"
)

const visualBaselines = new Set([
  "html-components/desktop/initial",
  "html-components/iphone-390/initial",
  "html-components/iphone-390/image-fallback",
  "html-components/minimum-320/code",
  "form-todo/desktop/initial",
  "form-todo/desktop/invalid-submit",
  "form-todo/desktop/valid-submit",
  "form-todo/desktop/empty",
  "project-flow-app/desktop/initial",
  "project-flow-app/desktop/explorer-code-preview",
  "project-flow-app/desktop/invalid-submit",
  "project-flow-app/desktop/day-studio",
  "project-flow-app/desktop/empty-disabled",
  "seseragi-landing-page/desktop/initial",
  "seseragi-landing-page/desktop/composable",
  "seseragi-landing-page/iphone-390/initial",
  "seseragi-landing-page/android-360/alive",
  "seseragi-landing-page/minimum-320/code",
])

const visualDifferenceRatio = 0.01
const visualColorThreshold = 0.25

function sample(id: string): MatrixSample {
  const entry = matrix.samples.find((candidate) => candidate.id === id)
  if (entry === undefined) throw new Error(`missing regression sample: ${id}`)
  return entry
}

async function routeImages(page: Page, failure = false): Promise<void> {
  await page.route(
    "https://raw.githubusercontent.com/KentaroMorishita/seseragi/main/assets/brand/source/seseragi-logo-dark.svg",
    (route) =>
      route.fulfill({
        contentType: "image/svg+xml",
        body: mockLogo,
      })
  )
  await page.route("https://images.unsplash.com/**", (route) => {
    if (failure) return route.abort("failed")
    return route.fulfill({
      contentType: "image/svg+xml",
      body: mockImage,
    })
  })
}

async function open(page: Page, width: number, height: number): Promise<void> {
  await page.setViewportSize({ width, height })
  await page.goto("/")
  await expect(page.locator("#sample-browser-button")).toBeVisible()
}

async function select(page: Page, entry: MatrixSample): Promise<void> {
  await page.locator("#sample-browser-button").click()
  const dialog = page.locator("#sample-browser-dialog")
  await expect(dialog).toBeVisible()
  await dialog.locator(`[data-sample-id="${entry.id}"]`).click()
  await expect(dialog).toBeHidden()
}

async function run(page: Page, entry: MatrixSample): Promise<FrameLocator> {
  await page.locator("#run-button").click()
  const preview = page.frameLocator("#html-preview")
  await expect(preview.locator("h1")).toContainText(entry.heading)
  await expect(preview.locator("img").first()).toBeVisible()
  return preview
}

async function capture(
  page: Page,
  testInfo: TestInfo,
  entry: MatrixSample,
  viewport: string,
  state: string,
  target?: Locator,
  sensitivityTarget?: Locator
): Promise<void> {
  const baseline = `${entry.id}/${viewport}/${state}`
  if (visualBaselines.has(baseline)) {
    const name = `${entry.id}-${viewport}-${state}.png`
    const surface = sensitivityTarget ?? target
    if (surface === undefined) {
      throw new Error(`${baseline} must declare its sensitivity surface`)
    }
    const options = await visualScreenshotOptions(surface)
    if (target === undefined) {
      await expect(page).toHaveScreenshot(name, { ...options, fullPage: true })
    } else {
      await expect(target).toHaveScreenshot(name, options)
    }
  }

  const path = testInfo.outputPath(
    "web-ui-samples",
    entry.id,
    `${viewport}-${state}.png`
  )
  await page.screenshot({ path, fullPage: true })
  await testInfo.attach(`${entry.id}-${viewport}-${state}`, {
    path,
    contentType: "image/png",
  })
}

async function visualScreenshotOptions(surface: Locator) {
  const bounds = await surface.evaluate((element) => {
    const rect = element.getBoundingClientRect()
    const view = element.ownerDocument.defaultView
    return {
      width: Math.min(rect.width, view?.innerWidth ?? rect.width),
      height: Math.min(rect.height, view?.innerHeight ?? rect.height),
    }
  })
  if (bounds.width <= 0 || bounds.height <= 0) {
    throw new Error("visual baseline sensitivity surface is not visible")
  }
  return {
    animations: "disabled" as const,
    caret: "hide" as const,
    maxDiffPixels: Math.max(
      1,
      Math.floor(bounds.width * bounds.height * visualDifferenceRatio)
    ),
    threshold: visualColorThreshold,
  }
}

async function expectLoadedImage(preview: FrameLocator): Promise<void> {
  await expect
    .poll(async () =>
      preview
        .locator("img")
        .first()
        .evaluate((image) => {
          const element = image as HTMLImageElement
          return {
            complete: element.complete,
            naturalWidth: element.naturalWidth,
          }
        })
    )
    .toEqual({ complete: true, naturalWidth: 960 })
}

async function expectNoHorizontalOverflow(
  page: Page,
  preview: FrameLocator
): Promise<void> {
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth
    )
  ).toBe(true)
  expect(
    await preview
      .locator("html")
      .evaluate((root) => root.scrollWidth <= root.clientWidth)
  ).toBe(true)
}

async function expectKeyboardReachable(preview: FrameLocator): Promise<void> {
  const control = preview
    .locator("a[href], button, input, textarea, select")
    .first()
  await expect(control).toBeVisible()
  await control.focus()
  await expect(control).toBeFocused()
  const bounds = await control.evaluate((element) => {
    const rect = element.getBoundingClientRect()
    return {
      left: rect.left,
      right: rect.right,
      top: rect.top,
      bottom: rect.bottom,
      width: window.innerWidth,
      height: window.innerHeight,
    }
  })
  expect(bounds.left).toBeGreaterThanOrEqual(0)
  expect(bounds.right).toBeLessThanOrEqual(bounds.width)
  expect(bounds.top).toBeGreaterThanOrEqual(0)
  expect(bounds.bottom).toBeLessThanOrEqual(bounds.height)
}

async function expectNoStickyOrFixedSampleControls(
  preview: FrameLocator
): Promise<void> {
  const positioned = await preview
    .locator("h1, h2, button, input, textarea, select")
    .evaluateAll((elements) =>
      elements.flatMap((element) => {
        const position = getComputedStyle(element).position
        return position === "fixed" || position === "sticky"
          ? [
              `${element.tagName.toLowerCase()}:${element.textContent?.trim() ?? ""}`,
            ]
          : []
      })
    )

  expect(positioned).toEqual([])
}

async function expectReadableContrast(
  preview: FrameLocator,
  sampleId: string
): Promise<void> {
  const entries = await preview
    .locator("h1, h2, p, button, a, label")
    .evaluateAll((elements) => {
      const parse = (
        value: string
      ): readonly [number, number, number, number] | undefined => {
        const match = value.match(
          /rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)/u
        )
        return match === null
          ? undefined
          : [
              Number(match[1]),
              Number(match[2]),
              Number(match[3]),
              Number(match[4] ?? "1"),
            ]
      }
      const luminance = (color: readonly [number, number, number]): number => {
        const channel = (value: number): number => {
          const normalized = value / 255
          return normalized <= 0.03928
            ? normalized / 12.92
            : ((normalized + 0.055) / 1.055) ** 2.4
        }
        return (
          0.2126 * channel(color[0]) +
          0.7152 * channel(color[1]) +
          0.0722 * channel(color[2])
        )
      }
      const background = (
        element: Element
      ): readonly [number, number, number] => {
        let current: Element | null = element
        while (current !== null) {
          const value = parse(getComputedStyle(current).backgroundColor)
          if (value !== undefined && value[3] > 0)
            return [value[0], value[1], value[2]]
          current = current.parentElement
        }
        return [255, 255, 255]
      }
      return elements.flatMap((element) => {
        const rect = element.getBoundingClientRect()
        const text = element.textContent?.trim() ?? ""
        if (text === "" || rect.width === 0 || rect.height === 0) return []
        const foreground = parse(getComputedStyle(element).color)
        if (foreground === undefined) return []
        const foregroundColor: readonly [number, number, number] = [
          foreground[0],
          foreground[1],
          foreground[2],
        ]
        const ratio =
          (Math.max(
            luminance(foregroundColor),
            luminance(background(element))
          ) +
            0.05) /
          (Math.min(
            luminance(foregroundColor),
            luminance(background(element))
          ) +
            0.05)
        return [
          {
            tag: element.tagName.toLowerCase(),
            text,
            ratio,
            large: Number.parseFloat(getComputedStyle(element).fontSize) >= 24,
          },
        ]
      })
    })

  expect(entries.length, sampleId).toBeGreaterThan(0)
  for (const entry of entries) {
    expect(
      entry.ratio,
      `${sampleId}: ${entry.tag} ${entry.text}`
    ).toBeGreaterThanOrEqual(entry.large ? 3 : 4.5)
  }
}

test.describe("canonical Web UI browser regression", () => {
  for (const viewport of matrix.viewports) {
    test(`${viewport.id} renders every HTML sample without horizontal overflow`, async ({
      page,
    }, testInfo) => {
      await routeImages(page)
      await open(page, viewport.width, viewport.height)

      for (const entry of matrix.samples) {
        await select(page, entry)
        if (viewport.id === "minimum-320") {
          await page.locator('[data-panel-target="code"]').click()
          const editor = page.locator(".cm-scroller")
          await expect(editor).toBeVisible()
          expect(
            await editor.evaluate(
              (element) => element.scrollWidth <= element.clientWidth
            )
          ).toBe(true)
          await capture(
            page,
            testInfo,
            entry,
            viewport.id,
            "code",
            undefined,
            page.locator("#editor-panel")
          )
        }

        const preview = await run(page, entry)
        await expectLoadedImage(preview)
        await expectNoHorizontalOverflow(page, preview)
        await expectKeyboardReachable(preview)
        await expectNoStickyOrFixedSampleControls(preview)
        await expectReadableContrast(preview, entry.id)
        await capture(
          page,
          testInfo,
          entry,
          viewport.id,
          "initial",
          undefined,
          preview.locator("body")
        )
      }
    })
  }

  test("records interaction, empty, disabled, and Explorer states", async ({
    page,
  }, testInfo) => {
    await routeImages(page)

    for (const id of ["interactive-app", "signal-run-route"] as const) {
      const entry = sample(id)
      await open(page, 1440, 1000)
      await select(page, entry)
      const preview = await run(page, entry)
      await preview.getByRole("button", { name: "川辺" }).click()
      await expect(preview.locator("h1")).toContainText("川辺をゆっくり歩く")
      await capture(page, testInfo, entry, "desktop", "riverside-route")
    }

    const feature = sample("feature-composition")
    await open(page, 1440, 1000)
    await select(page, feature)
    const featurePreview = await run(page, feature)
    await featurePreview
      .getByRole("button", { name: "Hide / show focus" })
      .click()
    await expect(
      featurePreview.getByText("First feature is outside the HTML tree.")
    ).toBeVisible()
    await capture(page, testInfo, feature, "desktop", "hidden-feature")

    const form = sample("form-todo")
    await open(page, 1440, 1000)
    await select(page, form)
    const formPreview = await run(page, form)
    const planTitle = formPreview.locator("#plan-title")
    const planDetails = formPreview.locator("#plan-details")
    const addToLaunchLoop = formPreview.getByRole("button", {
      name: "Add to launch loop",
    })
    await planTitle.scrollIntoViewIfNeeded()
    await planTitle.fill("Review the launch loop")
    await addToLaunchLoop.scrollIntoViewIfNeeded()
    await addToLaunchLoop.click()
    const formAlert = formPreview.getByRole("alert")
    await expect(formAlert).toContainText("clear purpose")
    await formAlert.evaluate((element) =>
      element.scrollIntoView({ block: "center" })
    )
    await capture(
      page,
      testInfo,
      form,
      "desktop",
      "invalid-submit",
      formPreview.locator("body")
    )
    await planDetails.scrollIntoViewIfNeeded()
    await planDetails.fill("Walk through every control once.")
    await addToLaunchLoop.scrollIntoViewIfNeeded()
    await addToLaunchLoop.click()
    const addedPlan = formPreview.locator('input[id="4"]')
    await expect(addedPlan).toHaveValue("Review the launch loop")
    await addedPlan.evaluate((element) =>
      element.scrollIntoView({ block: "center" })
    )
    await capture(
      page,
      testInfo,
      form,
      "desktop",
      "valid-submit",
      formPreview.locator("body")
    )
    while (await formPreview.getByRole("button", { name: "Remove" }).count()) {
      const remove = formPreview.getByRole("button", { name: "Remove" }).first()
      await remove.scrollIntoViewIfNeeded()
      await remove.click()
    }
    const emptyHeading = formPreview.getByRole("heading", {
      name: "Your launch loop is clear.",
    })
    await expect(emptyHeading).toBeVisible()
    await emptyHeading.evaluate((element) =>
      element.scrollIntoView({ block: "center" })
    )
    await capture(
      page,
      testInfo,
      form,
      "desktop",
      "empty",
      formPreview.locator("body")
    )

    const project = sample("project-flow-app")
    await open(page, 1440, 1000)
    await select(page, project)
    const projectPreview = await run(page, project)
    const explorerToggle = page.locator("#explorer-toggle-button")
    if ((await explorerToggle.getAttribute("aria-pressed")) !== "true") {
      await explorerToggle.click()
    }
    await expect(page.locator("#explorer-tree")).toBeVisible()
    await expect(page.locator("#workspace-tabs")).toBeVisible()
    await capture(
      page,
      testInfo,
      project,
      "desktop",
      "explorer-code-preview",
      undefined,
      page.locator('[data-testid="workspace-shell"]')
    )
    await projectPreview
      .getByRole("button", { name: "Add a story card" })
      .click()
    const projectAlert = projectPreview.getByRole("alert")
    await expect(projectAlert).toContainText("Give this card a title")
    await projectAlert.evaluate((element) =>
      element.scrollIntoView({ block: "center" })
    )
    await capture(
      page,
      testInfo,
      project,
      "desktop",
      "invalid-submit",
      projectPreview.locator("body")
    )
    await projectPreview.getByRole("button", { name: "Use day studio" }).click()
    const nightStudio = projectPreview.getByRole("button", {
      name: "Use night studio",
    })
    await expect(nightStudio).toBeVisible()
    await nightStudio.evaluate((element) =>
      element.scrollIntoView({ block: "center" })
    )
    await capture(
      page,
      testInfo,
      project,
      "desktop",
      "day-studio",
      projectPreview.locator("body")
    )
    const clearDeck = projectPreview.getByRole("button", { name: "Clear deck" })
    await clearDeck.click()
    const clearHeading = projectPreview.getByRole("heading", {
      name: "The deck is clear.",
    })
    await expect(clearHeading).toBeVisible()
    await expect(clearDeck).toBeDisabled()
    await clearHeading.evaluate((element) =>
      element.scrollIntoView({ block: "center" })
    )
    await capture(
      page,
      testInfo,
      project,
      "desktop",
      "empty-disabled",
      projectPreview.locator("body")
    )

    const landing = sample("seseragi-landing-page")
    await open(page, 1440, 1000)
    await select(page, landing)
    const landingPreview = await run(page, landing)
    const playgroundLink = landingPreview.getByRole("link", {
      name: "Playgroundで試す",
    })
    await expect(playgroundLink).toHaveAttribute(
      "href",
      "https://seseragi.vercel.app/"
    )
    await expect(playgroundLink).toHaveAttribute("target", "_blank")
    await expect(playgroundLink).toHaveAttribute("rel", "noopener noreferrer")
    await landingPreview.getByRole("button", { name: "Composable" }).click()
    await expect(
      landingPreview.getByRole("heading", {
        name: "別々の力を、ひとつの流れへ。",
      })
    ).toBeVisible()
    await capture(
      page,
      testInfo,
      landing,
      "desktop",
      "composable",
      undefined,
      landingPreview.locator("body")
    )

    await open(page, 360, 800)
    await select(page, landing)
    const landingMobilePreview = await run(page, landing)
    const aliveTab = landingMobilePreview.getByRole("button", { name: "Alive" })
    await aliveTab.focus()
    await expect(aliveTab).toBeFocused()
    await aliveTab.press("Enter")
    await expect(
      landingMobilePreview.getByRole("heading", {
        name: "言語は、動いた瞬間に生き始める。",
      })
    ).toBeVisible()
    await capture(
      page,
      testInfo,
      landing,
      "android-360",
      "alive",
      undefined,
      landingMobilePreview.locator("body")
    )
  })

  test("detects localized spacing, typography, and alignment regressions", async ({
    page,
  }, testInfo) => {
    test.skip(
      testInfo.config.updateSnapshots === "all" ||
        testInfo.config.updateSnapshots === "changed",
      "sensitivity checks must not rewrite reviewed baselines"
    )
    await routeImages(page)
    const entry = sample("html-components")
    await open(page, 1440, 1000)
    await select(page, entry)
    const preview = await run(page, entry)
    const surface = preview.locator("body")
    const name = "html-components-desktop-initial.png"
    const options = await visualScreenshotOptions(surface)

    await expect(page).toHaveScreenshot(name, { ...options, fullPage: true })
    for (const regression of [
      {
        id: "spacing",
        css: "main { padding-top: 8px !important; }",
      },
      {
        id: "typography",
        css: "h1, h2, p { letter-spacing: 1px !important; }",
      },
      {
        id: "alignment",
        css: "main { transform: translateX(8px) !important; }",
      },
    ]) {
      await expectLocalizedVisualMismatch(
        page,
        preview,
        name,
        options,
        regression
      )
    }
  })

  test("keeps descriptive image fallback layout for every HTML sample", async ({
    page,
  }, testInfo) => {
    await routeImages(page, true)
    await open(page, 390, 844)

    for (const entry of matrix.samples) {
      await select(page, entry)
      const preview = await run(page, entry)
      const image = preview.locator("img").first()
      await expect(image).toHaveAttribute("alt", /.+/u)
      await expect
        .poll(async () =>
          image.evaluate((element) => {
            const imageElement = element as HTMLImageElement
            const rect = imageElement.getBoundingClientRect()
            return {
              complete: imageElement.complete,
              naturalWidth: imageElement.naturalWidth,
              width: rect.width,
              height: rect.height,
            }
          })
        )
        .toMatchObject({
          complete: true,
          naturalWidth: 0,
          width: expect.any(Number),
          height: expect.any(Number),
        })
      const metrics = await image.evaluate((element) => {
        const rect = element.getBoundingClientRect()
        return { width: rect.width, height: rect.height }
      })
      expect(metrics.width, entry.id).toBeGreaterThan(0)
      expect(metrics.height, entry.id).toBeGreaterThan(0)
      await expectNoHorizontalOverflow(page, preview)
      await capture(
        page,
        testInfo,
        entry,
        "iphone-390",
        "image-fallback",
        undefined,
        preview.locator("body")
      )
    }
  })
})

async function expectLocalizedVisualMismatch(
  page: Page,
  preview: FrameLocator,
  name: string,
  options: Awaited<ReturnType<typeof visualScreenshotOptions>>,
  regression: Readonly<{ id: string; css: string }>
): Promise<void> {
  const styleId = `visual-regression-${regression.id}`
  await preview.locator("head").evaluate(
    (head, value) => {
      const style = head.ownerDocument.createElement("style")
      style.id = value.id
      style.textContent = value.css
      head.append(style)
    },
    { id: styleId, css: regression.css }
  )

  let mismatch: unknown
  try {
    await expect(page).toHaveScreenshot(name, {
      ...options,
      fullPage: true,
      timeout: 1_500,
    })
  } catch (error) {
    mismatch = error
  } finally {
    await preview.locator(`#${styleId}`).evaluate((style) => style.remove())
  }
  expect(
    mismatch,
    `${regression.id} regression must exceed the local surface budget`
  ).toBeDefined()
}
