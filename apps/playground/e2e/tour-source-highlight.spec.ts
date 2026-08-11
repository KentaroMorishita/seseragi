import { expect, type Page, test } from "@playwright/test"

test("highlights Tour source excerpts with the shared Seseragi tokens", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 })
  await page.goto("/tour/?lesson=abstraction-signal-contract")

  const code = page.locator(".tour-walkthrough-card code.seseragi-highlight")
  await expect(code).toHaveCount(1)
  const keywordTokens = await code.locator(".tok-keyword").allTextContents()
  expect(keywordTokens).toEqual(
    expect.arrayContaining(["import", "<$>", "<*>", "*"])
  )
  expect(await code.locator(".tok-typeName").allTextContents()).toEqual(
    expect.arrayContaining(["Int"])
  )
  expect(await code.locator(".tok-number").allTextContents()).toEqual(
    expect.arrayContaining(["20", "2"])
  )
  expect(await code.locator(".tok-string").allTextContents()).toEqual(
    expect.arrayContaining(["`Signal: "])
  )
  expect(await code.locator(".tok-punctuation").allTextContents()).toEqual(
    expect.arrayContaining(["{"])
  )
  await expect(code).toContainText("let doubled")
  expect(await code.textContent()).toContain("\n  let doubled")

  await page.goto("/tour/?lesson=abstraction-monad-bind")
  expect(
    await page
      .locator(".tour-walkthrough-card code .tok-keyword")
      .allTextContents()
  ).toContain(">>=")

  await page.goto("/tour/?lesson=comments-and-tools")
  await expect(
    page.locator(".tour-walkthrough-card code .tok-comment")
  ).toContainText("// このcomment")
})

test("preserves excerpt layout and selection on desktop and mobile", async ({
  page,
}) => {
  for (const viewport of [
    { width: 1440, height: 900 },
    { width: 390, height: 844 },
  ]) {
    await page.setViewportSize(viewport)
    await page.goto("/tour/?lesson=abstraction-signal-contract")
    const layout = await sourceLayout(page)

    expect(layout.codeText).toContain("\n  let doubled")
    expect(layout.overflowX).toBe("auto")
    expect(layout.userSelect).not.toBe("none")
    expect(layout.preLeft).toBeGreaterThanOrEqual(layout.cardLeft)
    expect(layout.preRight).toBeLessThanOrEqual(layout.cardRight + 1)
  }
})

async function sourceLayout(page: Page) {
  return page.locator(".tour-walkthrough-card").evaluate((card) => {
    const pre = card.querySelector("pre")
    const code = pre?.querySelector("code.seseragi-highlight")
    if (!(pre instanceof HTMLElement) || !(code instanceof HTMLElement)) {
      throw new Error("highlighted walkthrough source is missing")
    }
    const cardRect = card.getBoundingClientRect()
    const preRect = pre.getBoundingClientRect()
    const preStyle = getComputedStyle(pre)
    const codeStyle = getComputedStyle(code)
    return {
      cardLeft: cardRect.left,
      cardRight: cardRect.right,
      codeText: code.textContent ?? "",
      overflowX: preStyle.overflowX,
      preLeft: preRect.left,
      preRight: preRect.right,
      userSelect: codeStyle.userSelect,
    }
  })
}
