import { expect, test } from "@playwright/test"

test("renders every guide safely at mobile width", async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await page.goto("/")
  await page.locator("#sample-guide-button").click()
  await expect(page.locator("#sample-guide")).toBeVisible()

  const audit = await page.evaluate(async () => {
    const samplesPath = ["", "src", "samples.ts"].join("/")
    const rendererPath = ["", "src", "ui", "guide-markdown.ts"].join("/")
    const { samples } = (await import(samplesPath)) as {
      samples: readonly Readonly<{ id: string; guide: string }>[]
    }
    const { renderGuideMarkdown } = (await import(rendererPath)) as {
      renderGuideMarkdown: (target: HTMLElement, markdown: string) => void
    }
    const body = document.querySelector<HTMLElement>("#sample-guide-body")
    if (body === null) throw new Error("sample guide body is missing")

    const failures: string[] = []
    for (const sample of samples) {
      renderGuideMarkdown(body, sample.guide)
      const rawMarker = [...body.querySelectorAll("p")].some((paragraph) =>
        /^(?:#{2,3}\s|[-+*]\s|\d+[.)]\s|```)/u.test(
          paragraph.textContent?.trimStart() ?? ""
        )
      )
      if (
        body.childElementCount === 0 ||
        rawMarker ||
        body.scrollWidth > body.clientWidth
      ) {
        failures.push(sample.id)
      }
    }

    renderGuideMarkdown(
      body,
      "<script>globalThis.pwned = true</script> [open](javascript:alert(1))"
    )
    const security = {
      script: body.querySelector("script") !== null,
      link: body.querySelector("a") !== null,
      text: body.textContent ?? "",
    }
    renderGuideMarkdown(body, "## Old\n\n- stale")
    renderGuideMarkdown(body, "Current")
    const lifecycle = {
      headings: body.querySelectorAll("h2, h3").length,
      lists: body.querySelectorAll("ul, ol").length,
      paragraphs: body.querySelectorAll("p").length,
    }

    return { failures, lifecycle, security }
  })

  expect(audit.failures).toEqual([])
  expect(audit.security).toEqual({
    script: false,
    link: false,
    text: "<script>globalThis.pwned = true</script> open)",
  })
  expect(audit.lifecycle).toEqual({ headings: 0, lists: 0, paragraphs: 1 })

  await selectDiscoverSample(page, "form-todo")
  await page.locator("#sample-guide-button").click()
  const body = page.locator("#sample-guide-body")
  await expect(
    body.getByRole("heading", { name: "この画面で試すこと" })
  ).toBeVisible()
  await expect(body.locator("ul > li")).toHaveCount(8)

  await selectDiscoverSample(page, "project-flow-app")
  await page.locator("#sample-guide-button").click()
  await expect(body.locator("ol > li")).toHaveCount(5)
  await expect(body.locator("pre > code")).toContainText("Release Room page")
  expect(
    await body.evaluate((element) => element.scrollWidth <= element.clientWidth)
  ).toBe(true)

  const screenshot = testInfo.outputPath("project-flow-guide-mobile.png")
  await page.screenshot({ path: screenshot, fullPage: true })
  await testInfo.attach("project-flow-guide-mobile", {
    path: screenshot,
    contentType: "image/png",
  })

  await page.goto("/tour/?lesson=06-records-and-structs")
  const tourGuide = page.locator("#tour-walkthrough")
  await expect(
    page.getByRole("heading", { name: "codeを上から追う" })
  ).toBeVisible()
  await expect(
    tourGuide.getByText("struct Profile", { exact: true })
  ).toBeVisible()
  expect(
    await tourGuide.evaluate(
      (element) => element.scrollWidth <= element.clientWidth
    )
  ).toBe(true)
})

async function selectDiscoverSample(
  page: import("@playwright/test").Page,
  id: string
): Promise<void> {
  if (await page.locator("#sample-guide").isVisible()) {
    await page.locator("#sample-guide-close").click()
  }
  await page.locator("#sample-browser-button").click()
  await page.locator(`[data-sample-id="${id}"]`).click()
}
