import { chromium } from "playwright"

export async function launchTestBrowser() {
  const executablePath = process.env.SESERAGI_TEST_BROWSER_PATH
  const browser = await chromium.launch(
    executablePath ? { executablePath } : {}
  )
  console.log(
    `Browser tests: ${executablePath || "Playwright Chromium"} (${browser.version()})`
  )
  return browser
}
