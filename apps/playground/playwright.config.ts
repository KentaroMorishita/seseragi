import { defineConfig } from "@playwright/test"
import process from "node:process"

const baseURL = process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:4173"

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  forbidOnly: process.env.CI === "true",
  retries: process.env.CI === "true" ? 1 : 0,
  workers: 1,
  timeout: 90_000,
  expect: { timeout: 15_000 },
  outputDir: "test-results/web-ui-review/results",
  preserveOutput: "always",
  reporter: [
    ["line"],
    [
      "html",
      { outputFolder: "test-results/web-ui-review/report", open: "never" },
    ],
  ],
  use: {
    baseURL,
    browserName: "chromium",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    video: "retain-on-failure",
  },
  webServer: {
    command: "bun run dev -- --host 127.0.0.1 --port 4173",
    url: baseURL,
    reuseExistingServer: process.env.CI !== "true",
    timeout: 120_000,
  },
})
