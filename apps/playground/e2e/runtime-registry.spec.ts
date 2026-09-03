import { readFile } from "node:fs/promises"
import { expect, test } from "@playwright/test"

test("runs canonical Random shuffle from the editor using browser providers", async ({
  page,
}) => {
  const fixture = new URL(
    "../../../examples/spec/fixtures/projects/random-shuffle/",
    import.meta.url
  )
  const source = await readFile(new URL("src/main.ssrg", fixture), "utf8")
  const expected = (
    await readFile(new URL("expected.stdout", fixture), "utf8")
  ).trimEnd()
  const errors: string[] = []
  page.on("pageerror", (error) => errors.push(error.message))
  await page.addInitScript(() => {
    globalThis.__SESERAGI_RANDOM_SEED__ = "42"
  })
  for (let run = 0; run < 2; run++) {
    // A fresh browser realm matches the native fixture's fresh process. Within
    // one realm the existing Random provider retains its sequence between Runs.
    await page.goto("/")
    await page
      .getByRole("textbox", { name: "Seseragi source editor" })
      .fill(source)
    await page.locator("#run-button").click()
    await expect(page.locator("#output")).toHaveText(expected)
    await expect(page.locator("#run-button")).toBeEnabled()
  }
  expect(errors).toEqual([])
  await expect(page.locator("vite-error-overlay")).toHaveCount(0)
})
