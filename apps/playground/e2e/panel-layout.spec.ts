import { expect, type Locator, type Page, test } from "@playwright/test"
import { startQuietly } from "./quiet-start-support"

// 初めて開いたときの案内（最初の例の自動実行など）は first-example.spec.ts で確かめる
test.beforeEach(async ({ page }) => {
  await startQuietly(page)
})

const workspaceKey = "seseragi.playground.workspace-ratio"
const ioKey = "seseragi.playground.io-ratio"

async function resize(page: Page, width: number, height = 1000): Promise<void> {
  await page.setViewportSize({ width, height })
  await page.evaluate(
    () =>
      new Promise<void>((resolve) =>
        requestAnimationFrame(() => requestAnimationFrame(() => resolve()))
      )
  )
}

async function expectRatio(
  container: Locator,
  property: string,
  value: string
): Promise<void> {
  await expect
    .poll(() =>
      container.evaluate(
        (node, cssProperty) =>
          (node as HTMLElement).style.getPropertyValue(cssProperty),
        property
      )
    )
    .toBe(value)
}

test("restores default desktop panel ratios after narrow and short viewports", async ({
  page,
}) => {
  await resize(page, 1440)
  await page.goto("/")
  const workspace = page.locator(".workspace")
  const io = page.locator("#io-panel")
  await expectRatio(workspace, "--editor-panel-ratio", "55.00%")
  await expectRatio(io, "--stdin-panel-ratio", "34.00%")

  await resize(page, 1425)
  await expectRatio(workspace, "--editor-panel-ratio", "55.00%")
  await resize(page, 640)
  await expectRatio(workspace, "--editor-panel-ratio", "50.00%")
  await resize(page, 1440, 300)
  await expectRatio(io, "--stdin-panel-ratio", "50.00%")
  await resize(page, 1440)
  await expectRatio(workspace, "--editor-panel-ratio", "55.00%")
  await expectRatio(io, "--stdin-panel-ratio", "34.00%")
  expect(
    await page.evaluate(
      ({ workspace, io }) => [
        localStorage.getItem(workspace),
        localStorage.getItem(io),
      ],
      { workspace: workspaceKey, io: ioKey }
    )
  ).toEqual([null, null])
})

test("preserves user panel sizes across mobile reload and later user resizing", async ({
  page,
}) => {
  await resize(page, 1440)
  await page.goto("/")
  const workspace = page.locator(".workspace")
  const io = page.locator("#io-panel")
  const workspaceResizer = page.locator("#workspace-resizer")
  const ioResizer = page.locator("#io-resizer")
  await expect(workspaceResizer).toBeVisible()
  await workspaceResizer.press("ArrowLeft")
  await page.locator("#stdin-toggle-button").click()
  await ioResizer.press("ArrowDown")
  await expectRatio(workspace, "--editor-panel-ratio", "52.50%")
  await expectRatio(io, "--stdin-panel-ratio", "36.50%")

  await resize(page, 640)
  await page.reload()
  await expectRatio(workspace, "--editor-panel-ratio", "50.00%")
  await resize(page, 1440)
  await expectRatio(workspace, "--editor-panel-ratio", "52.50%")
  await expectRatio(io, "--stdin-panel-ratio", "36.50%")
  const persisted = await page.evaluate(
    ({ workspace, io }) => ({
      workspace: Number(localStorage.getItem(workspace)),
      io: Number(localStorage.getItem(io)),
    }),
    { workspace: workspaceKey, io: ioKey }
  )
  expect(persisted.workspace).toBeCloseTo(0.525)
  expect(persisted.io).toBeCloseTo(0.365)

  await workspaceResizer.press("End")
  const clamped = await page.evaluate(
    (key) => Number(localStorage.getItem(key)),
    workspaceKey
  )
  expect(clamped).toBeCloseTo(1 - 300 / 1440)
  await resize(page, 1000)
  await expectRatio(workspace, "--editor-panel-ratio", "70.00%")
  await workspaceResizer.press("ArrowLeft")
  await expectRatio(workspace, "--editor-panel-ratio", "67.50%")
  await resize(page, 1440)
  await expectRatio(workspace, "--editor-panel-ratio", "67.50%")
  const box = await workspaceResizer.boundingBox()
  if (box === null) throw new Error("workspace separator is not visible")
  // Grab the flow line above the Run button that sits on its middle.
  const grabY = box.y + 24
  await page.mouse.move(box.x + box.width / 2, grabY)
  await page.mouse.down()
  await page.mouse.move(800, grabY, { steps: 4 })
  await page.mouse.up()
  const dragged = await page.evaluate(
    (key) => Number(localStorage.getItem(key)),
    workspaceKey
  )
  expect(dragged).toBeCloseTo(800 / 1440)
  await resize(page, 640)
  await resize(page, 1440)
  await expectRatio(
    workspace,
    "--editor-panel-ratio",
    `${(dragged * 100).toFixed(2)}%`
  )
  await page.reload()
  await expectRatio(
    workspace,
    "--editor-panel-ratio",
    `${(dragged * 100).toFixed(2)}%`
  )
})
