import { expect, test } from "@playwright/test"

test("renders structured Tour inline content safely on mobile", async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await page.goto("/tour/?lesson=primitive-comparison")

  await expect(page.locator("#tour-prerequisite-copy code")).toContainText("+")
  const walkthroughCodes = await page
    .locator(".tour-walkthrough-card > p code")
    .allTextContents()
  expect(walkthroughCodes).toEqual(expect.arrayContaining(["42 > 40", "True"]))

  const inlineAudit = await page.evaluate(async () => {
    const rendererPath = ["", "src", "ui", "guide-markdown.ts"].join("/")
    const { renderGuideInline } = (await import(rendererPath)) as {
      renderGuideInline: (target: HTMLElement, source: string) => void
    }
    const host = document.createElement("p")
    renderGuideInline(
      host,
      "`code` *emphasis* **strong** [docs](https://example.com)"
    )
    const allowedTags = [...host.children].map((child) =>
      child.tagName.toLowerCase()
    )
    const link = host.querySelector("a")

    renderGuideInline(
      host,
      "## heading\n- item <script>boom</script> [bad](javascript:boom)"
    )
    const rejected = {
      block: host.querySelector("h1, h2, h3, ul, ol, pre") !== null,
      html: host.querySelector("script") !== null,
      link: host.querySelector("a") !== null,
      text: host.textContent ?? "",
    }

    renderGuideInline(host, "old `code`")
    renderGuideInline(host, "current")
    const lifecycle = {
      code: host.querySelector("code") !== null,
      text: host.textContent,
    }

    return {
      allowedTags,
      lifecycle,
      link: {
        target: link?.getAttribute("target"),
        rel: link?.getAttribute("rel"),
      },
      rejected,
    }
  })

  expect(inlineAudit.allowedTags).toEqual(["code", "em", "strong", "a"])
  expect(inlineAudit.link).toEqual({
    target: "_blank",
    rel: "noopener noreferrer",
  })
  expect(inlineAudit.rejected.block).toBe(false)
  expect(inlineAudit.rejected.html).toBe(false)
  expect(inlineAudit.rejected.link).toBe(false)
  expect(inlineAudit.rejected.text).toContain("<script>")
  expect(inlineAudit.lifecycle).toEqual({ code: false, text: "current" })

  const overflow = await page
    .locator(".tour-inline-rich-text")
    .evaluateAll(
      (elements) =>
        elements.filter((element) => element.scrollWidth > element.clientWidth)
          .length
    )
  expect(overflow).toBe(0)

  const screenshot = testInfo.outputPath("primitive-comparison-mobile.png")
  await page.screenshot({ path: screenshot, fullPage: true })
  await testInfo.attach("primitive-comparison-mobile", {
    path: screenshot,
    contentType: "image/png",
  })
})
